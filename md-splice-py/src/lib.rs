use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use markdown_ppp::ast::{Block, Document, HeadingKind, SetextHeading};
use markdown_ppp::printer::render_markdown;
use md_splice_lib::{
    default_printer_config,
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
    conversion::IntoPyObjectExt,
    create_exception,
    exceptions::{PyException, PyTypeError, PyValueError},
    prelude::*,
    types::{PyAny, PyAnyMethods, PyDict, PyList, PyModule, PyString, PyTuple, PyType},
    Bound,
};
use regex::{Regex, RegexBuilder};
use serde_json;
use serde_yaml::{Mapping as YamlMapping, Number as YamlNumber, Value as YamlValue};
use similar::TextDiff;
use tempfile::Builder as TempFileBuilder;

create_exception!(_native, MdSpliceError, PyException);

/// AST-backed Markdown document that mirrors the `md-splice` Rust core.
///
/// Instances of this class expose semantic selectors, transactional
/// operations, and atomic write helpers exactly as documented in
/// `goal-Python-library/Specification.md`.
#[pyclass(name = "MarkdownDocument", module = "md_splice")]
pub struct PyMarkdownDocument {
    inner: CoreMarkdownDocument,
    source_path: Option<PathBuf>,
}

#[pymethods]
impl PyMarkdownDocument {
    /// Parse Markdown from an in-memory string and return a new document.
    ///
    /// Use this constructor when you already hold the Markdown source. The
    /// resulting document can be queried with selectors, mutated via
    /// operations, and rendered back to Markdown with :meth:`render`.
    #[classmethod]
    pub fn from_string(_cls: &Bound<'_, PyType>, markdown: &str) -> PyResult<Self> {
        let document = CoreMarkdownDocument::from_str(markdown).map_err(map_splice_error)?;
        Ok(Self {
            inner: document,
            source_path: None,
        })
    }

    /// Load Markdown from ``path`` and associate the document with that file.
    ///
    /// Subsequent calls to :meth:`write_in_place` will persist changes back to
    /// this path using the atomic semantics required by the specification.
    #[classmethod]
    pub fn from_file(_cls: &Bound<'_, PyType>, path: &Bound<'_, PyAny>) -> PyResult<Self> {
        let path_buf: PathBuf = path.extract()?;
        let content = fs::read_to_string(&path_buf).map_err(|err| map_io_error(err))?;
        let document = CoreMarkdownDocument::from_str(&content).map_err(map_splice_error)?;

        Ok(Self {
            inner: document,
            source_path: Some(path_buf),
        })
    }

    /// Render the current Markdown document to a string.
    ///
    /// The output reflects all in-memory mutations performed through
    /// :meth:`apply` without writing them to disk.
    pub fn render(&self) -> PyResult<String> {
        Ok(self.inner.render())
    }

    /// Atomically write the document back to its source path.
    ///
    /// When ``backup`` is ``True`` the current on-disk file is first copied to
    /// a ``path~`` sibling before the atomic replace occurs. This mirrors the
    /// CLI's safety guarantees described in the specification.
    #[pyo3(signature = (*, backup=false))]
    pub fn write_in_place(&self, backup: bool) -> PyResult<()> {
        let Some(path) = &self.source_path else {
            return Err(map_splice_error(SpliceError::Io(
                "Document has no associated path; call write_to() instead.".to_string(),
            )));
        };

        if backup {
            create_backup(path.as_path())?;
        }

        let rendered = self.inner.render();
        write_atomic(path.as_path(), rendered.as_str())?;
        Ok(())
    }

    /// Render the document and write it to ``path`` atomically.
    ///
    /// Unlike :meth:`write_in_place`, this method always targets the provided
    /// location and does not require the document to originate from disk.
    pub fn write_to(&self, path: &Bound<'_, PyAny>) -> PyResult<()> {
        let path_buf: PathBuf = path.extract()?;
        write_atomic(path_buf.as_path(), &self.inner.render())
    }

    /// Apply a list of operations transactionally to the document.
    ///
    /// The operations mirror the CLI schema. All edits either succeed as a
    /// unit or the document remains unchanged. When ``warn_on_ambiguity`` is
    /// ``True`` a :class:`UserWarning` is emitted if any selector matches more
    /// than one node, matching the behavior mandated in the specification.
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

    /// Preview a list of operations without mutating the original document.
    ///
    /// The operations run against a clone and the rendered Markdown is
    /// returned. Ambiguity warnings follow the same rules as :meth:`apply`.
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

    /// Retrieve Markdown matching ``selector`` with optional range controls.
    ///
    /// When ``select_all`` is ``False`` the first match is returned. Setting
    /// ``section`` renders an entire heading section, while ``until`` defines a
    /// range ending before the provided selector. When ``select_all`` is
    /// ``True`` the return value is a list of rendered snippets for every
    /// match, and ``until`` must be omitted.
    #[pyo3(signature = (selector, *, select_all=false, section=false, until=None))]
    pub fn get(
        &self,
        py: Python<'_>,
        selector: &Bound<'_, PyAny>,
        select_all: bool,
        section: bool,
        until: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let locator_selector = py_selector_to_locator(py, selector)?;
        let blocks = self.inner.blocks();

        if select_all {
            if until.is_some() {
                return Err(PyValueError::new_err(
                    "until selector cannot be used when select_all=True",
                ));
            }

            let matches = locate_all(blocks, &locator_selector).map_err(map_splice_error)?;
            let py_list = PyList::empty(py);

            for found in &matches {
                let rendered = if section {
                    render_heading_section(blocks, found)?
                } else {
                    render_found_node(blocks, found)?
                };
                py_list.append(PyString::new(py, &rendered))?;
            }

            return Ok(py_list.into_any().unbind());
        }

        let (found_node, _) = locate(blocks, &locator_selector).map_err(map_splice_error)?;

        if let Some(until_selector) = until {
            let until_selector = py_selector_to_locator(py, until_selector)?;
            match &found_node {
                FoundNode::Block { index, .. } => {
                    let end_index = compute_range_end(blocks, *index, &until_selector)?;
                    let rendered = render_blocks(&blocks[*index..end_index]);
                    return Ok(PyString::new(py, &rendered).into_any().unbind());
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

        Ok(PyString::new(py, &rendered).into_any().unbind())
    }

    /// Return the frontmatter as native Python data or ``None``.
    ///
    /// The value mirrors the YAML/TOML content as described in the
    /// specification and round-trips through :class:`yaml` compatible types.
    pub fn frontmatter(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        match self.inner.frontmatter() {
            Some(value) => yaml_value_to_py(py, value).map(Some),
            None => Ok(None),
        }
    }

    /// Return the detected frontmatter format enum or ``None`` when absent.
    pub fn frontmatter_format(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let Some(format) = self.inner.frontmatter_format() else {
            return Ok(None);
        };

        let types_module = py.import("md_splice.types")?;
        let enum_class = types_module.getattr("FrontmatterFormat")?;

        let variant_name = match format {
            FrontmatterFormat::Yaml => "YAML",
            FrontmatterFormat::Toml => "TOML",
        };

        let variant = enum_class.getattr(variant_name)?;
        Ok(Some(variant.into_any().unbind()))
    }

    /// Create a deep copy of the document, including pending mutations.
    pub fn clone(&self) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.clone(),
            source_path: self.source_path.clone(),
        })
    }
}

#[pymodule]
fn _native(py: Python, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_class::<PyMarkdownDocument>()?;
    module.add("MdSpliceError", py.get_type::<MdSpliceError>())?;
    module.add_function(pyo3::wrap_pyfunction!(diff_unified, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(loads_operations, module)?)?;
    module.add_function(pyo3::wrap_pyfunction!(dumps_operations, module)?)?;
    Ok(())
}

fn map_splice_error(err: SpliceError) -> PyErr {
    Python::attach(|py| match map_splice_error_inner(py, &err) {
        Ok(py_err) => py_err,
        Err(_) => MdSpliceError::new_err(err.to_string()),
    })
}

fn map_splice_error_inner(py: Python<'_>, err: &SpliceError) -> PyResult<PyErr> {
    let errors_module = py.import("md_splice.errors")?;
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
        SpliceError::SelectorAliasNotDefined(_) => {
            ("SelectorAliasNotDefinedError", err.to_string())
        }
        SpliceError::SelectorAliasAlreadyDefined(_) => {
            ("SelectorAliasAlreadyDefinedError", err.to_string())
        }
        SpliceError::AmbiguousSelectorSource(_) => {
            ("AmbiguousSelectorSourceError", err.to_string())
        }
        SpliceError::AmbiguousNestedSelectorSource(_) => {
            ("AmbiguousNestedSelectorSourceError", err.to_string())
        }
        SpliceError::FrontmatterMissing => ("FrontmatterMissingError", err.to_string()),
        SpliceError::FrontmatterKeyNotFound(_) => ("FrontmatterKeyNotFoundError", err.to_string()),
        SpliceError::FrontmatterParse(_) => ("FrontmatterParseError", err.to_string()),
        SpliceError::FrontmatterSerialize(_) => ("FrontmatterSerializeError", err.to_string()),
        SpliceError::MarkdownParse(_) => ("MarkdownParseError", err.to_string()),
        SpliceError::OperationParse(_) => ("OperationParseError", err.to_string()),
        SpliceError::OperationFailed(_) => ("OperationFailedError", err.to_string()),
        SpliceError::Io(_) => ("IoError", err.to_string()),
    };

    let error_type = errors_module.getattr(class_name)?.cast_into::<PyType>()?;
    Ok(PyErr::from_type(error_type, (message,)))
}

fn maybe_emit_ambiguity_warning(
    py: Python<'_>,
    warn_on_ambiguity: bool,
    outcome: ApplyOutcome,
) -> PyResult<()> {
    if warn_on_ambiguity && outcome.ambiguity_detected {
        let warnings = py.import("warnings")?;
        let builtins = py.import("builtins")?;
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
    let iterator = operations.try_iter()?;
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
            let selector = if selector_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &selector_obj)?)
            };
            let selector_ref = operation
                .getattr("selector_ref")?
                .extract::<Option<String>>()?;
            let content = operation.getattr("content")?.extract::<Option<String>>()?;
            let position_obj = operation.getattr("position")?;
            let position = py_insert_position_to_rust(&position_obj)?;
            Ok(TxOperation::Insert(TxInsertOperation {
                selector,
                selector_ref,
                comment: None,
                content,
                content_file: None,
                position,
            }))
        }
        "ReplaceOperation" => {
            let selector_obj = operation.getattr("selector")?;
            let selector = if selector_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &selector_obj)?)
            };
            let selector_ref = operation
                .getattr("selector_ref")?
                .extract::<Option<String>>()?;
            let content = operation.getattr("content")?.extract::<Option<String>>()?;
            let until_obj = operation.getattr("until")?;
            let until = if until_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &until_obj)?)
            };
            let until_ref = operation
                .getattr("until_ref")?
                .extract::<Option<String>>()?;
            Ok(TxOperation::Replace(TxReplaceOperation {
                selector,
                selector_ref,
                comment: None,
                content,
                content_file: None,
                until,
                until_ref,
            }))
        }
        "DeleteOperation" => {
            let selector_obj = operation.getattr("selector")?;
            let selector = if selector_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &selector_obj)?)
            };
            let selector_ref = operation
                .getattr("selector_ref")?
                .extract::<Option<String>>()?;
            let section = operation.getattr("section")?.extract::<bool>()?;
            let until_obj = operation.getattr("until")?;
            let until = if until_obj.is_none() {
                None
            } else {
                Some(py_selector_to_transaction(py, &until_obj)?)
            };
            let until_ref = operation
                .getattr("until_ref")?
                .extract::<Option<String>>()?;
            Ok(TxOperation::Delete(TxDeleteOperation {
                selector,
                selector_ref,
                comment: None,
                section,
                until,
                until_ref,
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
    let alias = selector.getattr("alias")?.extract::<Option<String>>()?;
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
    let after_ref = selector.getattr("after_ref")?.extract::<Option<String>>()?;
    let within_obj = selector.getattr("within")?;
    let within = if within_obj.is_none() {
        None
    } else {
        Some(Box::new(py_selector_to_transaction(py, &within_obj)?))
    };
    let within_ref = selector
        .getattr("within_ref")?
        .extract::<Option<String>>()?;

    Ok(TxSelector {
        alias,
        select_type,
        select_contains,
        select_regex,
        select_ordinal,
        after,
        after_ref,
        within,
        within_ref,
    })
}

fn py_insert_position_to_rust(position: &Bound<'_, PyAny>) -> PyResult<TxInsertPosition> {
    let value: String = if let Ok(val) = position.getattr("value") {
        val.extract()?
    } else {
        position.extract()?
    };

    let normalized = value.replace('-', "_");

    match normalized.as_str() {
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

    if let Ok(list) = obj.cast::<PyList>() {
        let mut seq = Vec::with_capacity(list.len());
        for item in list {
            seq.push(py_to_yaml_value(py, &item)?);
        }
        return Ok(YamlValue::Sequence(seq));
    }

    if let Ok(tuple) = obj.cast::<PyTuple>() {
        let mut seq = Vec::with_capacity(tuple.len());
        for item in tuple {
            seq.push(py_to_yaml_value(py, &item)?);
        }
        return Ok(YamlValue::Sequence(seq));
    }

    if let Ok(dict) = obj.cast::<PyDict>() {
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

fn yaml_value_to_py(py: Python<'_>, value: &YamlValue) -> PyResult<Py<PyAny>> {
    match value {
        YamlValue::Null => Ok(py.None()),
        YamlValue::Bool(value) => (*value).into_py_any(py),
        YamlValue::Number(number) => {
            if let Some(int_value) = number.as_i64() {
                int_value.into_py_any(py)
            } else if let Some(uint_value) = number.as_u64() {
                uint_value.into_py_any(py)
            } else if let Some(float_value) = number.as_f64() {
                float_value.into_py_any(py)
            } else {
                Err(PyErr::new::<PyException, _>(
                    "Unsupported YAML number representation",
                ))
            }
        }
        YamlValue::String(value) => value.clone().into_py_any(py),
        YamlValue::Sequence(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(yaml_value_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        YamlValue::Mapping(mapping) => {
            let dict = PyDict::new(py);
            for (key, value) in mapping {
                let key_obj = yaml_value_to_py(py, key)?;
                let value_obj = yaml_value_to_py(py, value)?;
                dict.set_item(key_obj, value_obj)?;
            }
            Ok(dict.into_any().unbind())
        }
        YamlValue::Tagged(tagged) => yaml_value_to_py(py, &tagged.value),
    }
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
    let flags = extract_regex_flags(py, pattern_obj)?;

    let mut builder = RegexBuilder::new(&pattern);
    builder.case_insensitive(flags.ignore_case);
    builder.multi_line(flags.multi_line);
    builder.dot_matches_new_line(flags.dot_all);
    builder.unicode(true);

    builder
        .build()
        .map_err(|err| invalid_regex_pyerr(py, err.to_string()))
}

fn invalid_regex_pyerr(py: Python<'_>, message: String) -> PyErr {
    if let Ok(errors_module) = py.import("md_splice.errors") {
        if let Ok(obj) = errors_module.getattr("InvalidRegexError") {
            if let Ok(error_type) = obj.cast_into::<PyType>() {
                return PyErr::from_type(error_type, (message,));
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

#[derive(Default)]
struct RegexFlags {
    ignore_case: bool,
    multi_line: bool,
    dot_all: bool,
}

fn extract_regex_flags(py: Python<'_>, pattern_obj: &Bound<'_, PyAny>) -> PyResult<RegexFlags> {
    if !pattern_obj.hasattr("flags")? {
        return Ok(RegexFlags::default());
    }

    let flags_value = pattern_obj.getattr("flags")?.extract::<u32>()?;
    if flags_value == 0 {
        return Ok(RegexFlags::default());
    }

    let re_module = py.import("re")?;
    let flag_ignorecase = re_module.getattr("IGNORECASE")?.extract::<u32>()?;
    let flag_multiline = re_module.getattr("MULTILINE")?.extract::<u32>()?;
    let flag_dotall = re_module.getattr("DOTALL")?.extract::<u32>()?;
    let flag_unicode = re_module.getattr("UNICODE")?.extract::<u32>()?;

    let supported_mask = flag_ignorecase | flag_multiline | flag_dotall | flag_unicode;

    let known_unsupported = [
        ("VERBOSE", re_module.getattr("VERBOSE")?.extract::<u32>()?),
        ("ASCII", re_module.getattr("ASCII")?.extract::<u32>()?),
        ("LOCALE", re_module.getattr("LOCALE")?.extract::<u32>()?),
        ("DEBUG", re_module.getattr("DEBUG")?.extract::<u32>()?),
    ];

    let mut unsupported: Vec<String> = Vec::new();
    let mut known_mask = 0u32;
    for (name, value) in &known_unsupported {
        known_mask |= *value;
        if flags_value & value != 0 {
            unsupported.push(name.to_string());
        }
    }

    let leftover = flags_value & !(supported_mask | known_mask);
    if leftover != 0 {
        unsupported.push(format!("0x{leftover:x}"));
    }

    if !unsupported.is_empty() {
        return Err(invalid_regex_pyerr(
            py,
            format!("Unsupported regex flag(s): {}", unsupported.join(", ")),
        ));
    }

    Ok(RegexFlags {
        ignore_case: flags_value & flag_ignorecase != 0,
        multi_line: flags_value & flag_multiline != 0,
        dot_all: flags_value & flag_dotall != 0,
    })
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
    let mut rendered = render_markdown(&temp_doc, default_printer_config());
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

/// Produce a unified diff between two Markdown strings.
///
/// The optional ``fromfile`` and ``tofile`` labels appear in the diff header.
#[pyfunction]
#[pyo3(signature = (before, after, *, fromfile="original", tofile="modified"))]
fn diff_unified(before: &str, after: &str, fromfile: &str, tofile: &str) -> PyResult<String> {
    let diff = TextDiff::from_lines(before, after)
        .unified_diff()
        .header(fromfile, tofile)
        .to_string();
    Ok(diff)
}

/// Parse YAML or JSON operation definitions into Python dataclasses.
///
/// The ``format`` parameter can force ``"yaml"`` or ``"json"``. When omitted
/// the loader first attempts YAML then falls back to JSON, matching the CLI.
#[pyfunction]
#[pyo3(signature = (text, *, format=None))]
fn loads_operations(py: Python<'_>, text: &str, format: Option<&str>) -> PyResult<Py<PyAny>> {
    let operations = parse_operations(text, format).map_err(map_splice_error)?;
    let types_module = py.import("md_splice.types")?;
    let py_list = PyList::empty(py);

    for operation in &operations {
        let py_op = tx_operation_to_py(py, &types_module, operation)?;
        py_list.append(py_op)?;
    }

    Ok(py_list.into_any().unbind())
}

/// Serialize Python operation dataclasses to YAML or JSON.
///
/// ``format`` defaults to ``"yaml"``; specifying ``"json"`` returns formatted
/// JSON compatible with the CLI tooling.
#[pyfunction]
#[pyo3(signature = (operations, *, format="yaml"))]
fn dumps_operations(
    py: Python<'_>,
    operations: &Bound<'_, PyAny>,
    format: &str,
) -> PyResult<String> {
    let tx_operations = py_operations_to_rust(py, operations)?;
    let yaml_operations = tx_operations
        .iter()
        .map(tx_operation_to_yaml_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(map_splice_error)?;
    let normalized = format.to_ascii_lowercase();

    match normalized.as_str() {
        "yaml" => serde_yaml::to_string(&yaml_operations)
            .map_err(|err| map_splice_error(SpliceError::OperationParse(err.to_string()))),
        "json" => serde_json::to_string_pretty(&yaml_operations)
            .map_err(|err| map_splice_error(SpliceError::OperationParse(err.to_string()))),
        other => Err(PyValueError::new_err(format!(
            "Unsupported operations format: {other}"
        ))),
    }
}

fn parse_operations(text: &str, format: Option<&str>) -> Result<Vec<TxOperation>, SpliceError> {
    let normalized = format.map(|value| value.to_ascii_lowercase());
    match normalized.as_deref() {
        Some("yaml") => serde_yaml::from_str(text)
            .map_err(|err| SpliceError::OperationParse(err.to_string())),
        Some("json") => serde_json::from_str(text)
            .map_err(|err| SpliceError::OperationParse(err.to_string())),
        Some(other) => Err(SpliceError::OperationParse(format!(
            "Unsupported operations format: {other}"
        ))),
        None => match serde_yaml::from_str(text) {
            Ok(value) => Ok(value),
            Err(yaml_err) => serde_json::from_str(text).map_err(|json_err| {
                SpliceError::OperationParse(format!(
                    "Failed to parse operations as YAML ({yaml_err}). Attempt to parse as JSON also failed ({json_err})."
                ))
            }),
        },
    }
}

fn tx_operation_to_py(
    py: Python<'_>,
    types_module: &Bound<'_, PyModule>,
    operation: &TxOperation,
) -> PyResult<Py<PyAny>> {
    match operation {
        TxOperation::Insert(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("InsertOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            if let Some(selector) = &op.selector {
                let selector_value = tx_selector_to_py(py, types_module, selector)?;
                kwargs.set_item("selector", selector_value)?;
            }
            if let Some(selector_ref) = &op.selector_ref {
                kwargs.set_item("selector_ref", selector_ref)?;
            }
            if let Some(content) = &op.content {
                kwargs.set_item("content", content)?;
            }
            let position = insert_position_to_py(py, types_module, op.position)?;
            kwargs.set_item("position", position)?;
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
        TxOperation::Replace(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("ReplaceOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            if let Some(selector) = &op.selector {
                let selector_value = tx_selector_to_py(py, types_module, selector)?;
                kwargs.set_item("selector", selector_value)?;
            }
            if let Some(selector_ref) = &op.selector_ref {
                kwargs.set_item("selector_ref", selector_ref)?;
            }
            if let Some(content) = &op.content {
                kwargs.set_item("content", content)?;
            }
            if let Some(until) = &op.until {
                let until_selector = tx_selector_to_py(py, types_module, until)?;
                kwargs.set_item("until", until_selector)?;
            }
            if let Some(until_ref) = &op.until_ref {
                kwargs.set_item("until_ref", until_ref)?;
            }
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
        TxOperation::Delete(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("DeleteOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            if let Some(selector) = &op.selector {
                let selector_value = tx_selector_to_py(py, types_module, selector)?;
                kwargs.set_item("selector", selector_value)?;
            }
            if let Some(selector_ref) = &op.selector_ref {
                kwargs.set_item("selector_ref", selector_ref)?;
            }
            kwargs.set_item("section", op.section)?;
            if let Some(until) = &op.until {
                let until_selector = tx_selector_to_py(py, types_module, until)?;
                kwargs.set_item("until", until_selector)?;
            }
            if let Some(until_ref) = &op.until_ref {
                kwargs.set_item("until_ref", until_ref)?;
            }
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
        TxOperation::SetFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;
            ensure_operation_field_absent(op.value_file.as_ref(), "value_file")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("SetFrontmatterOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("key", &op.key)?;
            let value = match &op.value {
                Some(value) => yaml_value_to_py(py, value)?,
                None => py.None(),
            };
            kwargs.set_item("value", value)?;
            if let Some(format) = op.format {
                let format_value = frontmatter_format_to_py(py, types_module, format)?;
                kwargs.set_item("format", format_value)?;
            }
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
        TxOperation::DeleteFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("DeleteFrontmatterOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("key", &op.key)?;
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
        TxOperation::ReplaceFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")
                .map_err(map_splice_error)?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")
                .map_err(map_splice_error)?;

            let class = types_module
                .getattr("ReplaceFrontmatterOperation")?
                .cast_into::<PyType>()?;
            let kwargs = PyDict::new(py);
            let content = match &op.content {
                Some(value) => yaml_value_to_py(py, value)?,
                None => py.None(),
            };
            kwargs.set_item("content", content)?;
            if let Some(format) = op.format {
                let format_value = frontmatter_format_to_py(py, types_module, format)?;
                kwargs.set_item("format", format_value)?;
            }
            let instance = class.call((), Some(&kwargs))?;
            Ok(instance.into_any().unbind())
        }
    }
}

fn tx_operation_to_yaml_value(operation: &TxOperation) -> Result<YamlValue, SpliceError> {
    let mut mapping = YamlMapping::new();

    match operation {
        TxOperation::Insert(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("insert".to_string()),
            );
            if let Some(selector) = &op.selector {
                mapping.insert(
                    YamlValue::String("selector".to_string()),
                    tx_selector_to_yaml_value(selector),
                );
            }
            if let Some(selector_ref) = &op.selector_ref {
                mapping.insert(
                    YamlValue::String("selector_ref".to_string()),
                    YamlValue::String(selector_ref.clone()),
                );
            }
            if let Some(content) = &op.content {
                mapping.insert(
                    YamlValue::String("content".to_string()),
                    YamlValue::String(content.clone()),
                );
            }
            if op.position != TxInsertPosition::After {
                mapping.insert(
                    YamlValue::String("position".to_string()),
                    YamlValue::String(insert_position_to_str(op.position).to_string()),
                );
            }
        }
        TxOperation::Replace(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("replace".to_string()),
            );
            if let Some(selector) = &op.selector {
                mapping.insert(
                    YamlValue::String("selector".to_string()),
                    tx_selector_to_yaml_value(selector),
                );
            }
            if let Some(selector_ref) = &op.selector_ref {
                mapping.insert(
                    YamlValue::String("selector_ref".to_string()),
                    YamlValue::String(selector_ref.clone()),
                );
            }
            if let Some(content) = &op.content {
                mapping.insert(
                    YamlValue::String("content".to_string()),
                    YamlValue::String(content.clone()),
                );
            }
            if let Some(until) = &op.until {
                mapping.insert(
                    YamlValue::String("until".to_string()),
                    tx_selector_to_yaml_value(until),
                );
            }
            if let Some(until_ref) = &op.until_ref {
                mapping.insert(
                    YamlValue::String("until_ref".to_string()),
                    YamlValue::String(until_ref.clone()),
                );
            }
        }
        TxOperation::Delete(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("delete".to_string()),
            );
            if let Some(selector) = &op.selector {
                mapping.insert(
                    YamlValue::String("selector".to_string()),
                    tx_selector_to_yaml_value(selector),
                );
            }
            if let Some(selector_ref) = &op.selector_ref {
                mapping.insert(
                    YamlValue::String("selector_ref".to_string()),
                    YamlValue::String(selector_ref.clone()),
                );
            }
            if op.section {
                mapping.insert(
                    YamlValue::String("section".to_string()),
                    YamlValue::Bool(true),
                );
            }
            if let Some(until) = &op.until {
                mapping.insert(
                    YamlValue::String("until".to_string()),
                    tx_selector_to_yaml_value(until),
                );
            }
            if let Some(until_ref) = &op.until_ref {
                mapping.insert(
                    YamlValue::String("until_ref".to_string()),
                    YamlValue::String(until_ref.clone()),
                );
            }
        }
        TxOperation::SetFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;
            ensure_operation_field_absent(op.value_file.as_ref(), "value_file")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("set_frontmatter".to_string()),
            );
            mapping.insert(
                YamlValue::String("key".to_string()),
                YamlValue::String(op.key.clone()),
            );
            let value = op.value.as_ref().cloned().unwrap_or(YamlValue::Null);
            mapping.insert(YamlValue::String("value".to_string()), value);
            if let Some(format) = op.format {
                mapping.insert(
                    YamlValue::String("format".to_string()),
                    YamlValue::String(frontmatter_format_to_str(format).to_string()),
                );
            }
        }
        TxOperation::DeleteFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("delete_frontmatter".to_string()),
            );
            mapping.insert(
                YamlValue::String("key".to_string()),
                YamlValue::String(op.key.clone()),
            );
        }
        TxOperation::ReplaceFrontmatter(op) => {
            ensure_operation_field_absent(op.comment.as_ref(), "comment")?;
            ensure_operation_field_absent(op.content_file.as_ref(), "content_file")?;

            mapping.insert(
                YamlValue::String("op".to_string()),
                YamlValue::String("replace_frontmatter".to_string()),
            );
            let content = op.content.as_ref().cloned().unwrap_or(YamlValue::Null);
            mapping.insert(YamlValue::String("content".to_string()), content);
            if let Some(format) = op.format {
                mapping.insert(
                    YamlValue::String("format".to_string()),
                    YamlValue::String(frontmatter_format_to_str(format).to_string()),
                );
            }
        }
    }

    Ok(YamlValue::Mapping(mapping))
}

fn tx_selector_to_yaml_value(selector: &TxSelector) -> YamlValue {
    let mut mapping = YamlMapping::new();

    if let Some(alias) = &selector.alias {
        mapping.insert(
            YamlValue::String("alias".to_string()),
            YamlValue::String(alias.clone()),
        );
    }
    if let Some(select_type) = &selector.select_type {
        mapping.insert(
            YamlValue::String("select_type".to_string()),
            YamlValue::String(select_type.clone()),
        );
    }
    if let Some(select_contains) = &selector.select_contains {
        mapping.insert(
            YamlValue::String("select_contains".to_string()),
            YamlValue::String(select_contains.clone()),
        );
    }
    if let Some(select_regex) = &selector.select_regex {
        mapping.insert(
            YamlValue::String("select_regex".to_string()),
            YamlValue::String(select_regex.clone()),
        );
    }
    if selector.select_ordinal != 1 {
        mapping.insert(
            YamlValue::String("select_ordinal".to_string()),
            YamlValue::Number(YamlNumber::from(selector.select_ordinal as i64)),
        );
    }
    if let Some(after) = &selector.after {
        mapping.insert(
            YamlValue::String("after".to_string()),
            tx_selector_to_yaml_value(after),
        );
    }
    if let Some(after_ref) = &selector.after_ref {
        mapping.insert(
            YamlValue::String("after_ref".to_string()),
            YamlValue::String(after_ref.clone()),
        );
    }
    if let Some(within) = &selector.within {
        mapping.insert(
            YamlValue::String("within".to_string()),
            tx_selector_to_yaml_value(within),
        );
    }
    if let Some(within_ref) = &selector.within_ref {
        mapping.insert(
            YamlValue::String("within_ref".to_string()),
            YamlValue::String(within_ref.clone()),
        );
    }

    YamlValue::Mapping(mapping)
}

fn tx_selector_to_py(
    py: Python<'_>,
    types_module: &Bound<'_, PyModule>,
    selector: &TxSelector,
) -> PyResult<Py<PyAny>> {
    let class = types_module.getattr("Selector")?.cast_into::<PyType>()?;
    let kwargs = PyDict::new(py);

    if let Some(alias) = &selector.alias {
        kwargs.set_item("alias", alias)?;
    }
    if let Some(select_type) = &selector.select_type {
        kwargs.set_item("select_type", select_type)?;
    }
    if let Some(select_contains) = &selector.select_contains {
        kwargs.set_item("select_contains", select_contains)?;
    }
    if let Some(select_regex) = &selector.select_regex {
        kwargs.set_item("select_regex", select_regex)?;
    }
    if selector.select_ordinal != 1 {
        kwargs.set_item("select_ordinal", selector.select_ordinal)?;
    }
    if let Some(after) = &selector.after {
        let nested = tx_selector_to_py(py, types_module, after)?;
        kwargs.set_item("after", nested)?;
    }
    if let Some(after_ref) = &selector.after_ref {
        kwargs.set_item("after_ref", after_ref)?;
    }
    if let Some(within) = &selector.within {
        let nested = tx_selector_to_py(py, types_module, within)?;
        kwargs.set_item("within", nested)?;
    }
    if let Some(within_ref) = &selector.within_ref {
        kwargs.set_item("within_ref", within_ref)?;
    }

    let instance = class.call((), Some(&kwargs))?;
    Ok(instance.into_any().unbind())
}

fn insert_position_to_py(
    _py: Python<'_>,
    types_module: &Bound<'_, PyModule>,
    position: TxInsertPosition,
) -> PyResult<Py<PyAny>> {
    let enum_class = types_module.getattr("InsertPosition")?;
    let variant_name = match position {
        TxInsertPosition::Before => "BEFORE",
        TxInsertPosition::After => "AFTER",
        TxInsertPosition::PrependChild => "PREPEND_CHILD",
        TxInsertPosition::AppendChild => "APPEND_CHILD",
    };
    Ok(enum_class.getattr(variant_name)?.into_any().unbind())
}

fn insert_position_to_str(position: TxInsertPosition) -> &'static str {
    match position {
        TxInsertPosition::Before => "before",
        TxInsertPosition::After => "after",
        TxInsertPosition::PrependChild => "prepend_child",
        TxInsertPosition::AppendChild => "append_child",
    }
}

fn frontmatter_format_to_py(
    _py: Python<'_>,
    types_module: &Bound<'_, PyModule>,
    format: FrontmatterFormat,
) -> PyResult<Py<PyAny>> {
    let enum_class = types_module.getattr("FrontmatterFormat")?;
    let variant_name = match format {
        FrontmatterFormat::Yaml => "YAML",
        FrontmatterFormat::Toml => "TOML",
    };
    Ok(enum_class.getattr(variant_name)?.into_any().unbind())
}

fn frontmatter_format_to_str(format: FrontmatterFormat) -> &'static str {
    match format {
        FrontmatterFormat::Yaml => "yaml",
        FrontmatterFormat::Toml => "toml",
    }
}

fn ensure_operation_field_absent<T>(
    field: Option<&T>,
    field_name: &str,
) -> Result<(), SpliceError> {
    if field.is_some() {
        Err(unsupported_operation_field(field_name))
    } else {
        Ok(())
    }
}

fn unsupported_operation_field(field: &str) -> SpliceError {
    SpliceError::OperationParse(format!(
        "Operations containing `{}` are not supported by the Python API.",
        field
    ))
}

fn create_backup(path: &Path) -> PyResult<PathBuf> {
    if !path.exists() {
        return Err(map_splice_error(SpliceError::Io(format!(
            "Cannot create backup; file does not exist: {}",
            path.display()
        ))));
    }

    let mut backup_name = path.as_os_str().to_os_string();
    backup_name.push("~");
    let backup_path = PathBuf::from(backup_name);

    fs::copy(path, &backup_path).map_err(|err| map_io_error(err))?;
    Ok(backup_path)
}

fn write_atomic(path: &Path, content: &str) -> PyResult<()> {
    let parent = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        Some(_) | None => Path::new("."),
    };

    let mut temp_file = TempFileBuilder::new()
        .prefix(".md-splice-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|err| map_io_error(io::Error::new(io::ErrorKind::Other, err.to_string())))?;

    temp_file
        .write_all(content.as_bytes())
        .map_err(|err| map_io_error(err))?;
    temp_file.flush().map_err(|err| map_io_error(err))?;
    temp_file
        .persist(path)
        .map_err(|err| map_io_error(err.error))?;
    Ok(())
}

fn map_io_error(err: io::Error) -> PyErr {
    map_splice_error(SpliceError::Io(err.to_string()))
}
