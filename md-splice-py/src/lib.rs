use std::str::FromStr;

use markdown_ppp::ast::{Block, Document, HeadingKind, SetextHeading};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use md_splice_lib::{
    error::SpliceError,
    frontmatter::FrontmatterFormat,
    locator::{locate, locate_all, FoundNode, Selector as LocatorSelector},
    MarkdownDocument as CoreMarkdownDocument,
};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyModule, PyString, PyType},
    Bound,
};
use regex::Regex;
use serde_yaml::Value as YamlValue;

create_exception!(_native, MdSpliceError, PyException);

#[pyclass(name = "MarkdownDocument", module = "md_splice")]
pub struct PyMarkdownDocument {
    inner: CoreMarkdownDocument,
}

#[pymethods]
impl PyMarkdownDocument {
    #[classmethod]
    pub fn from_string(_cls: &Bound<'_, PyType>, markdown: &str) -> PyResult<Self> {
        let document = CoreMarkdownDocument::from_str(markdown).map_err(map_splice_error)?;
        Ok(Self { inner: document })
    }

    pub fn render(&self) -> PyResult<String> {
        Ok(self.inner.render())
    }

    #[pyo3(signature = (selector, *, select_all=false, section=false, until=None))]
    pub fn get(
        &self,
        py: Python<'_>,
        selector: &Bound<'_, PyAny>,
        select_all: bool,
        section: bool,
        until: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyObject> {
        let locator_selector = py_selector_to_locator(py, selector)?;
        let blocks = self.inner.blocks();

        if select_all {
            if until.is_some() {
                return Err(PyValueError::new_err(
                    "until selector cannot be used when select_all=True",
                ));
            }

            let matches = locate_all(blocks, &locator_selector).map_err(map_splice_error)?;
            let py_list = PyList::empty_bound(py);

            for found in &matches {
                let rendered = if section {
                    render_heading_section(blocks, found)?
                } else {
                    render_found_node(blocks, found)?
                };
                py_list.append(PyString::new_bound(py, &rendered))?;
            }

            return Ok(py_list.into_py(py));
        }

        let (found_node, _) = locate(blocks, &locator_selector).map_err(map_splice_error)?;

        if let Some(until_selector) = until {
            let until_selector = py_selector_to_locator(py, until_selector)?;
            match &found_node {
                FoundNode::Block { index, .. } => {
                    let end_index = compute_range_end(blocks, *index, &until_selector)?;
                    let rendered = render_blocks(&blocks[*index..end_index]);
                    return Ok(PyString::new_bound(py, &rendered).into_py(py));
                }
                FoundNode::ListItem { .. } => {
                    return Err(map_splice_error(SpliceError::RangeRequiresBlock));
                }
            }
        }

        let rendered = if section {
            render_heading_section(blocks, &found_node)?
        } else {
            render_found_node(blocks, &found_node)?
        };

        Ok(PyString::new_bound(py, &rendered).into_py(py))
    }

    pub fn frontmatter(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        match self.inner.frontmatter() {
            Some(value) => yaml_value_to_py(py, value).map(Some),
            None => Ok(None),
        }
    }

    pub fn frontmatter_format(&self, py: Python<'_>) -> PyResult<Option<PyObject>> {
        let Some(format) = self.inner.frontmatter_format() else {
            return Ok(None);
        };

        let types_module = PyModule::import_bound(py, "md_splice.types")?;
        let enum_class = types_module.getattr("FrontmatterFormat")?;

        let variant_name = match format {
            FrontmatterFormat::Yaml => "YAML",
            FrontmatterFormat::Toml => "TOML",
        };

        let variant = enum_class.getattr(variant_name)?;
        Ok(Some(variant.into_py(py)))
    }
}

#[pymodule]
fn _native(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyMarkdownDocument>()?;
    module.add("MdSpliceError", py.get_type_bound::<MdSpliceError>())?;
    Ok(())
}

fn map_splice_error(err: SpliceError) -> PyErr {
    Python::with_gil(|py| match map_splice_error_inner(py, &err) {
        Ok(py_err) => py_err,
        Err(_) => MdSpliceError::new_err(err.to_string()),
    })
}

fn map_splice_error_inner(py: Python<'_>, err: &SpliceError) -> PyResult<PyErr> {
    let errors_module = PyModule::import_bound(py, "md_splice.errors")?;
    let (class_name, message) = match err {
        SpliceError::NodeNotFound => ("NodeNotFoundError", err.to_string()),
        SpliceError::InvalidChildInsertion(_) => ("InvalidChildInsertionError", err.to_string()),
        SpliceError::AmbiguousContentSource => ("AmbiguousContentSourceError", err.to_string()),
        SpliceError::NoContent => ("NoContentError", err.to_string()),
        SpliceError::InvalidListItemContent => ("InvalidListItemContentError", err.to_string()),
        SpliceError::AmbiguousStdinSource => ("AmbiguousStdinSourceError", err.to_string()),
        SpliceError::InvalidSectionDelete => ("InvalidSectionDeleteError", err.to_string()),
        SpliceError::SectionRequiresHeading => ("SectionRequiresHeadingError", err.to_string()),
        SpliceError::ConflictingScopeModifiers => ("ConflictingScopeError", err.to_string()),
        SpliceError::RangeRequiresBlock => ("RangeRequiresBlockError", err.to_string()),
        SpliceError::FrontmatterMissing => ("FrontmatterMissingError", err.to_string()),
        SpliceError::FrontmatterKeyNotFound(_) => ("FrontmatterKeyNotFoundError", err.to_string()),
        SpliceError::FrontmatterParse(_) => ("FrontmatterParseError", err.to_string()),
        SpliceError::FrontmatterSerialize(_) => ("FrontmatterSerializeError", err.to_string()),
        SpliceError::MarkdownParse(_) => ("MarkdownParseError", err.to_string()),
        SpliceError::OperationParse(_) => ("OperationParseError", err.to_string()),
        SpliceError::OperationFailed(_) => ("OperationFailedError", err.to_string()),
        SpliceError::Io(_) => ("IoError", err.to_string()),
    };

    let error_type = errors_module
        .getattr(class_name)?
        .downcast_into::<PyType>()?;
    Ok(PyErr::from_type_bound(error_type, (message,)))
}

fn yaml_value_to_py(py: Python<'_>, value: &YamlValue) -> PyResult<PyObject> {
    Ok(match value {
        YamlValue::Null => py.None().into_py(py),
        YamlValue::Bool(value) => (*value).into_py(py),
        YamlValue::Number(number) => {
            if let Some(int_value) = number.as_i64() {
                int_value.into_py(py)
            } else if let Some(uint_value) = number.as_u64() {
                uint_value.into_py(py)
            } else if let Some(float_value) = number.as_f64() {
                float_value.into_py(py)
            } else {
                return Err(PyErr::new::<PyException, _>(
                    "Unsupported YAML number representation",
                ));
            }
        }
        YamlValue::String(value) => value.clone().into_py(py),
        YamlValue::Sequence(items) => {
            let list = PyList::empty_bound(py);
            for item in items {
                list.append(yaml_value_to_py(py, item)?)?;
            }
            list.into_py(py)
        }
        YamlValue::Mapping(mapping) => {
            let dict = PyDict::new_bound(py);
            for (key, value) in mapping {
                let key_obj = yaml_value_to_py(py, key)?;
                let value_obj = yaml_value_to_py(py, value)?;
                dict.set_item(key_obj, value_obj)?;
            }
            dict.into_py(py)
        }
        YamlValue::Tagged(tagged) => yaml_value_to_py(py, &tagged.value)?,
    })
}

fn py_selector_to_locator(
    py: Python<'_>,
    selector: &Bound<'_, PyAny>,
) -> PyResult<LocatorSelector> {
    let select_type = selector
        .getattr("select_type")?
        .extract::<Option<String>>()?;
    let select_contains = selector
        .getattr("select_contains")?
        .extract::<Option<String>>()?;
    let select_regex_obj = selector.getattr("select_regex")?;
    let select_regex = if select_regex_obj.is_none() {
        None
    } else {
        Some(python_regex_to_rust(py, &select_regex_obj)?)
    };
    let select_ordinal = selector.getattr("select_ordinal")?.extract::<usize>()?;
    let after_obj = selector.getattr("after")?;
    let after = if after_obj.is_none() {
        None
    } else {
        Some(Box::new(py_selector_to_locator(py, &after_obj)?))
    };
    let within_obj = selector.getattr("within")?;
    let within = if within_obj.is_none() {
        None
    } else {
        Some(Box::new(py_selector_to_locator(py, &within_obj)?))
    };

    Ok(LocatorSelector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after,
        within,
    })
}

fn python_regex_to_rust(py: Python<'_>, pattern_obj: &Bound<'_, PyAny>) -> PyResult<Regex> {
    let pattern = if let Ok(pattern_attr) = pattern_obj.getattr("pattern") {
        pattern_attr.extract::<String>()?
    } else {
        pattern_obj.extract::<String>()?
    };

    Regex::new(&pattern).map_err(|err| invalid_regex_pyerr(py, err.to_string()))
}

fn invalid_regex_pyerr(py: Python<'_>, message: String) -> PyErr {
    if let Ok(errors_module) = PyModule::import_bound(py, "md_splice.errors") {
        if let Ok(obj) = errors_module.getattr("InvalidRegexError") {
            if let Ok(error_type) = obj.downcast_into::<PyType>() {
                return PyErr::from_type_bound(error_type, (message,));
            }
        }
    }

    PyException::new_err(message)
}

fn compute_range_end(
    blocks: &[Block],
    start_index: usize,
    until_selector: &LocatorSelector,
) -> PyResult<usize> {
    if start_index + 1 >= blocks.len() {
        return Ok(blocks.len());
    }

    match locate(&blocks[start_index + 1..], until_selector) {
        Ok((FoundNode::Block { index, .. }, _)) => Ok(start_index + 1 + index),
        Ok((FoundNode::ListItem { .. }, _)) => {
            Err(map_splice_error(SpliceError::RangeRequiresBlock))
        }
        Err(SpliceError::NodeNotFound) => Ok(blocks.len()),
        Err(other) => Err(map_splice_error(other)),
    }
}

fn render_heading_section(blocks: &[Block], found: &FoundNode) -> PyResult<String> {
    if let FoundNode::Block { index, block } = found {
        if let Some(level) = get_heading_level(block) {
            let end_index = find_heading_section_end(blocks, *index, level);
            return Ok(render_blocks(&blocks[*index..end_index]));
        }
    }

    Err(map_splice_error(SpliceError::SectionRequiresHeading))
}

fn render_found_node(blocks: &[Block], found: &FoundNode) -> PyResult<String> {
    match found {
        FoundNode::Block { block, .. } => Ok(render_blocks(std::slice::from_ref(block))),
        FoundNode::ListItem {
            block_index, item, ..
        } => match blocks.get(*block_index) {
            Some(Block::List(list)) => {
                let mut single_list = list.clone();
                single_list.items = vec![(*item).clone()];
                Ok(render_blocks(std::slice::from_ref(&Block::List(
                    single_list,
                ))))
            }
            _ => Err(PyException::new_err(format!(
                "Internal error: block at index {} is not a list",
                block_index
            ))),
        },
    }
}

fn render_blocks(blocks: &[Block]) -> String {
    let temp_doc = Document {
        blocks: blocks.to_vec(),
    };
    let mut rendered = render_markdown(&temp_doc, PrinterConfig::default());
    if !rendered.is_empty() && !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered
}

fn get_heading_level(block: &Block) -> Option<u8> {
    match block {
        Block::Heading(heading) => match heading.kind {
            HeadingKind::Atx(level) => Some(level),
            HeadingKind::Setext(SetextHeading::Level1) => Some(1),
            HeadingKind::Setext(SetextHeading::Level2) => Some(2),
        },
        _ => None,
    }
}

fn find_heading_section_end(blocks: &[Block], heading_index: usize, target_level: u8) -> usize {
    let mut end = blocks.len();
    for (idx, block) in blocks.iter().enumerate().skip(heading_index + 1) {
        if let Some(level) = get_heading_level(block) {
            if level <= target_level {
                end = idx;
                break;
            }
        }
    }
    end
}
