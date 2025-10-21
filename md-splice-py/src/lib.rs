use std::str::FromStr;

use markdown_ppp::ast::{Block, Document, HeadingKind, SetextHeading};
use markdown_ppp::printer::{config::Config as PrinterConfig, render_markdown};
use md_splice_lib::{
    error::SpliceError,
    frontmatter::FrontmatterFormat,
    locator::{locate, locate_all, FoundNode, Selector as LocatorSelector},
    transaction::{
        DeleteFrontmatterOperation as TxDeleteFrontmatterOperation,
        DeleteOperation as TxDeleteOperation, InsertOperation as TxInsertOperation,
        InsertPosition as TxInsertPosition, Operation as TxOperation,
        ReplaceFrontmatterOperation as TxReplaceFrontmatterOperation,
        ReplaceOperation as TxReplaceOperation, Selector as TxSelector,
        SetFrontmatterOperation as TxSetFrontmatterOperation,
    },
    ApplyOutcome, MarkdownDocument as CoreMarkdownDocument,
};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyTypeError, PyValueError},
    prelude::*,
    types::{PyAny, PyDict, PyList, PyModule, PyString, PyTuple, PyType},
    Bound,
};
use regex::Regex;
use serde_yaml::{Mapping as YamlMapping, Number as YamlNumber, Value as YamlValue};
use similar::TextDiff;

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

    #[pyo3(signature = (ops, *, warn_on_ambiguity=true))]
    pub fn apply(
        &mut self,
        py: Python<'_>,
        ops: &Bound<'_, PyAny>,
        warn_on_ambiguity: bool,
    ) -> PyResult<()> {
        let operations = py_operations_to_rust(py, ops)?;
        let outcome = self
            .inner
            .apply_with_ambiguity(operations)
            .map_err(map_splice_error)?;
        maybe_emit_ambiguity_warning(py, warn_on_ambiguity, outcome)?;
        Ok(())
    }

    #[pyo3(signature = (ops, *, warn_on_ambiguity=true))]
    pub fn preview(
        &self,
        py: Python<'_>,
        ops: &Bound<'_, PyAny>,
        warn_on_ambiguity: bool,
    ) -> PyResult<String> {
        let operations = py_operations_to_rust(py, ops)?;
        let mut clone = self.inner.clone();
        let outcome = clone
            .apply_with_ambiguity(operations)
            .map_err(map_splice_error)?;
        maybe_emit_ambiguity_warning(py, warn_on_ambiguity, outcome)?;
        Ok(clone.render())
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

    pub fn clone(&self) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.clone(),
        })
    }
}

#[pymodule]
fn _native(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyMarkdownDocument>()?;
    module.add("MdSpliceError", py.get_type_bound::<MdSpliceError>())?;
    module.add_function(pyo3::wrap_pyfunction!(diff_unified, module)?)?;
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

fn maybe_emit_ambiguity_warning(
    py: Python<'_>,
    warn_on_ambiguity: bool,
    outcome: ApplyOutcome,
) -> PyResult<()> {
    if warn_on_ambiguity && outcome.ambiguity_detected {
        let warnings = PyModule::import_bound(py, "warnings")?;
        let builtins = PyModule::import_bound(py, "builtins")?;
        let warning_type = builtins.getattr("UserWarning")?;
        warnings.call_method1(
            "warn",
            (
                "Selector matched multiple nodes; first match used.",
                warning_type,
            ),
        )?;
    }

    Ok(())
}

fn py_operations_to_rust(
    py: Python<'_>,
    operations: &Bound<'_, PyAny>,
) -> PyResult<Vec<TxOperation>> {
    let iterator = operations.iter()?;
    let mut converted = Vec::new();
    for item in iterator {
        let bound = item?;
        converted.push(py_operation_to_rust(py, &bound)?);
    }
    Ok(converted)
}

fn py_operation_to_rust(py: Python<'_>, operation: &Bound<'_, PyAny>) -> PyResult<TxOperation> {
    let class = operation.getattr("__class__")?;
    let name: String = class.getattr("__name__")?.extract()?;

    match name.as_str() {
        "InsertOperation" => {
            let selector_obj = operation.getattr("selector")?;
            let selector = py_selector_to_transaction(py, &selector_obj)?;
            let content = operation.getattr("content")?.extract::<Option<String>>()?;
            let position_obj = operation.getattr("position")?;
            let position = py_insert_position_to_rust(&position_obj)?;
            Ok(TxOperation::Insert(TxInsertOperation {
                selector,
                comment: None,
                content,
                content_file: None,
                position,
            }))
        }
        "ReplaceOperation" => {
            let selector_obj = operation.getattr("selector")?;
            let selector = py_selector_to_transaction(py, &selector_obj)?;
            let content = operation.getattr("content")?.extract::<Option<String>>()?;
            let until_obj = operation.getattr("until")?;
            let until = if until_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &until_obj)?)
            };
            Ok(TxOperation::Replace(TxReplaceOperation {
                selector,
                comment: None,
                content,
                content_file: None,
                until,
            }))
        }
        "DeleteOperation" => {
            let selector_obj = operation.getattr("selector")?;
            let selector = py_selector_to_transaction(py, &selector_obj)?;
            let section = operation.getattr("section")?.extract::<bool>()?;
            let until_obj = operation.getattr("until")?;
            let until = if until_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &until_obj)?)
            };
            Ok(TxOperation::Delete(TxDeleteOperation {
                selector,
                comment: None,
                section,
                until,
            }))
        }
        "SetFrontmatterOperation" => {
            let key: String = operation.getattr("key")?.extract()?;
            let value_obj = operation.getattr("value")?;
            let value = Some(py_to_yaml_value(py, &value_obj)?);
            let format_obj = operation.getattr("format")?;
            let format = if format_obj.is_none() {
                None
            } else {
                Some(py_frontmatter_format_to_rust(&format_obj)?)
            };
            Ok(TxOperation::SetFrontmatter(TxSetFrontmatterOperation {
                key,
                comment: None,
                value,
                value_file: None,
                format,
            }))
        }
        "DeleteFrontmatterOperation" => {
            let key: String = operation.getattr("key")?.extract()?;
            Ok(TxOperation::DeleteFrontmatter(
                TxDeleteFrontmatterOperation { key, comment: None },
            ))
        }
        "ReplaceFrontmatterOperation" => {
            let content_obj = operation.getattr("content")?;
            let content = Some(py_to_yaml_value(py, &content_obj)?);
            let format_obj = operation.getattr("format")?;
            let format = if format_obj.is_none() {
                None
            } else {
                Some(py_frontmatter_format_to_rust(&format_obj)?)
            };
            Ok(TxOperation::ReplaceFrontmatter(
                TxReplaceFrontmatterOperation {
                    comment: None,
                    content,
                    content_file: None,
                    format,
                },
            ))
        }
        other => Err(PyValueError::new_err(format!(
            "Unsupported operation type: {other}"
        ))),
    }
}

fn py_selector_to_transaction(py: Python<'_>, selector: &Bound<'_, PyAny>) -> PyResult<TxSelector> {
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
        Some(extract_regex_pattern(&select_regex_obj)?)
    };
    let select_ordinal = selector.getattr("select_ordinal")?.extract::<usize>()?;
    let after_obj = selector.getattr("after")?;
    let after = if after_obj.is_none() {
        None
    } else {
        Some(Box::new(py_selector_to_transaction(py, &after_obj)?))
    };
    let within_obj = selector.getattr("within")?;
    let within = if within_obj.is_none() {
        None
    } else {
        Some(Box::new(py_selector_to_transaction(py, &within_obj)?))
    };

    Ok(TxSelector {
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after,
        within,
    })
}

fn py_insert_position_to_rust(position: &Bound<'_, PyAny>) -> PyResult<TxInsertPosition> {
    let value: String = if let Ok(val) = position.getattr("value") {
        val.extract()?
    } else {
        position.extract()?
    };

    match value.as_str() {
        "before" => Ok(TxInsertPosition::Before),
        "after" => Ok(TxInsertPosition::After),
        "prepend_child" => Ok(TxInsertPosition::PrependChild),
        "append_child" => Ok(TxInsertPosition::AppendChild),
        _ => Err(PyValueError::new_err(format!(
            "Unsupported insert position: {value}"
        ))),
    }
}

fn py_frontmatter_format_to_rust(format_obj: &Bound<'_, PyAny>) -> PyResult<FrontmatterFormat> {
    let value: String = if let Ok(val) = format_obj.getattr("name") {
        val.extract()?
    } else if let Ok(val) = format_obj.getattr("value") {
        val.extract::<String>()?
    } else {
        format_obj.extract()?
    };

    match value.as_str() {
        "YAML" | "yaml" => Ok(FrontmatterFormat::Yaml),
        "TOML" | "toml" => Ok(FrontmatterFormat::Toml),
        other => Err(PyValueError::new_err(format!(
            "Unsupported frontmatter format: {other}"
        ))),
    }
}

fn py_to_yaml_value(py: Python<'_>, obj: &Bound<'_, PyAny>) -> PyResult<YamlValue> {
    if obj.is_none() {
        return Ok(YamlValue::Null);
    }

    if let Ok(value) = obj.extract::<bool>() {
        return Ok(YamlValue::Bool(value));
    }

    if let Ok(value) = obj.extract::<i64>() {
        return Ok(YamlValue::Number(YamlNumber::from(value)));
    }

    if let Ok(value) = obj.extract::<u64>() {
        return Ok(YamlValue::Number(YamlNumber::from(value)));
    }

    if let Ok(value) = obj.extract::<f64>() {
        if value.is_finite() {
            return Ok(YamlValue::from(value));
        } else {
            return Err(PyValueError::new_err(
                "Float value is not representable in YAML",
            ));
        }
    }

    if let Ok(value) = obj.extract::<String>() {
        return Ok(YamlValue::String(value));
    }

    if let Ok(list) = obj.downcast::<PyList>() {
        let mut seq = Vec::with_capacity(list.len());
        for item in list {
            seq.push(py_to_yaml_value(py, &item)?);
        }
        return Ok(YamlValue::Sequence(seq));
    }

    if let Ok(tuple) = obj.downcast::<PyTuple>() {
        let mut seq = Vec::with_capacity(tuple.len());
        for item in tuple {
            seq.push(py_to_yaml_value(py, &item)?);
        }
        return Ok(YamlValue::Sequence(seq));
    }

    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut mapping = YamlMapping::new();
        for (key, value) in dict.iter() {
            let key_value = py_to_yaml_value(py, &key)?;
            let value_value = py_to_yaml_value(py, &value)?;
            mapping.insert(key_value, value_value);
        }
        return Ok(YamlValue::Mapping(mapping));
    }

    Err(PyTypeError::new_err(format!(
        "Unsupported value type for YAML conversion: {}",
        obj.get_type().name()?
    )))
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
    let pattern = extract_regex_pattern(pattern_obj)?;
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

fn extract_regex_pattern(pattern_obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(pattern_attr) = pattern_obj.getattr("pattern") {
        pattern_attr.extract::<String>()
    } else {
        pattern_obj.extract::<String>()
    }
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

#[pyfunction]
#[pyo3(signature = (before, after, *, fromfile="original", tofile="modified"))]
fn diff_unified(before: &str, after: &str, fromfile: &str, tofile: &str) -> PyResult<String> {
    let diff = TextDiff::from_lines(before, after)
        .unified_diff()
        .header(fromfile, tofile)
        .to_string();
    Ok(diff)
}
