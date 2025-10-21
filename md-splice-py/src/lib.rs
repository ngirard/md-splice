use std::str::FromStr;

use md_splice_lib::{
    error::SpliceError, frontmatter::FrontmatterFormat, MarkdownDocument as CoreMarkdownDocument,
};
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyList, PyModule, PyType},
    Bound,
};
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
    MdSpliceError::new_err(err.to_string())
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
