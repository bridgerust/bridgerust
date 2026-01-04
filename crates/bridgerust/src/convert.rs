//! Type conversion helpers for BridgeRust
//!
//! This module provides utilities for converting between Rust types and
//! Python/Node.js types, making it easier to work with complex data structures.

#[cfg(feature = "python")]
pub mod python {
    //! Python-specific type conversion helpers

    use crate::pyo3::prelude::*;
    use crate::pyo3::{Py, PyAny, PyResult, Python};

    /// Convert a Rust Vec to a Python list
    ///
    /// # Example
    /// ```rust,ignore
    /// use bridgerust::convert::python::vec_to_py_list;
    ///
    /// #[export]
    /// pub fn get_numbers(py: Python) -> PyResult<Py<PyAny>> {
    ///     let numbers = vec![1, 2, 3, 4, 5];
    ///     vec_to_py_list(py, numbers)
    /// }
    /// ```
    pub fn vec_to_py_list<T: Into<Py<PyAny>>>(py: Python<'_>, vec: Vec<T>) -> PyResult<Py<PyAny>> {
        use crate::pyo3::types::PyList;
        let list = PyList::empty(py);
        for item in vec {
            list.append(item.into())?;
        }
        Ok(list.into())
    }

    /// Convert a Python list to a Rust Vec
    ///
    /// # Example
    /// ```rust,ignore
    /// use bridgerust::convert::python::py_list_to_vec;
    ///
    /// #[export]
    /// pub fn sum_numbers(py: Python, numbers: Py<PyAny>) -> PyResult<i32> {
    ///     let vec: Vec<i32> = py_list_to_vec(py, numbers)?;
    ///     Ok(vec.iter().sum())
    /// }
    /// ```
    pub fn py_list_to_vec<'py, T>(py: Python<'py>, obj: Py<PyAny>) -> PyResult<Vec<T>>
    where
        T: for<'a> FromPyObject<'a, 'py>,
        for<'a> <T as FromPyObject<'a, 'py>>::Error: Into<PyErr>,
    {
        use crate::pyo3::types::PyList;
        let bound = obj.bind(py);
        let list = bound.cast::<PyList>()?;
        let mut vec = Vec::with_capacity(list.len());
        for i in 0..list.len() {
            let item = list.get_item(i)?;
            vec.push(item.extract::<T>().map_err(Into::into)?);
        }
        Ok(vec)
    }

    /// Convert a Rust Option to Python (None becomes None, Some becomes the value)
    ///
    /// This is already handled automatically by PyO3, but this helper provides
    /// explicit conversion when needed.
    pub fn option_to_py<T: Into<Py<PyAny>>>(py: Python<'_>, opt: Option<T>) -> Py<PyAny> {
        match opt {
            Some(value) => value.into(),
            None => py.None(),
        }
    }

    /// Convert Python None/Some to Rust Option
    ///
    /// This is already handled automatically by PyO3, but this helper provides
    /// explicit conversion when needed.
    pub fn py_to_option<'py, T>(py: Python<'py>, obj: Py<PyAny>) -> PyResult<Option<T>>
    where
        T: for<'a> FromPyObject<'a, 'py>,
        for<'a> <T as FromPyObject<'a, 'py>>::Error: Into<PyErr>,
    {
        let bound = obj.bind(py);
        if bound.is_none() {
            Ok(None)
        } else {
            Ok(Some(bound.extract::<T>().map_err(Into::into)?))
        }
    }
}

#[cfg(feature = "nodejs")]
pub mod nodejs {
    //! Node.js-specific type conversion helpers
    //!
    //! Note: napi-rs automatically handles most type conversions (Vec<T>, Option<T>, etc.)
    //! These helpers are for advanced use cases or when you need explicit control.

    use crate::napi::Env;

    /// Helper to convert complex nested structures
    ///
    /// For most cases, napi-rs handles Vec<T> and Option<T> automatically.
    /// Use this helper when you need custom conversion logic.
    ///
    /// # Example
    /// ```rust,ignore
    /// use bridgerust::convert::nodejs::convert_vec;
    ///
    /// #[export]
    /// pub fn process_items(env: Env, items: Vec<MyType>) -> Result<Vec<ProcessedType>> {
    ///     // napi-rs automatically converts Vec<MyType> from JS array
    ///     // and Vec<ProcessedType> to JS array
    ///     Ok(items.into_iter().map(process).collect())
    /// }
    /// ```
    ///
    /// This is mainly for documentation - napi-rs handles conversions automatically.
    pub fn convert_vec<T, U>(_env: Env, items: Vec<T>, f: impl Fn(T) -> U) -> Vec<U> {
        items.into_iter().map(f).collect()
    }
}

/// Re-export conversion helpers based on enabled features
#[cfg(feature = "python")]
pub use python::*;

#[cfg(feature = "nodejs")]
pub use nodejs::*;
