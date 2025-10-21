use std::str::FromStr;

use md_splice_lib::{error::SpliceError, MarkdownDocument as CoreMarkdownDocument};
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyModule, PyType},
    Bound,
};

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

    pub fn frontmatter(&self) -> PyResult<Option<PyObject>> {
        Ok(None)
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
