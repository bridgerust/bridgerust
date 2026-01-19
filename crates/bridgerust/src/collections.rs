use std::collections::HashMap;
#[cfg(feature = "python")]
use std::hash::Hash;

/// A wrapper around `std::collections::HashMap` for cross-language compatibility.
pub struct HashMapWrapper<K, V>(pub HashMap<K, V>);

impl<K, V> From<HashMap<K, V>> for HashMapWrapper<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Self(map)
    }
}

impl<K, V> From<HashMapWrapper<K, V>> for HashMap<K, V> {
    fn from(wrapper: HashMapWrapper<K, V>) -> Self {
        wrapper.0
    }
}

// Deref to access underlying map methods easily
impl<K, V> std::ops::Deref for HashMapWrapper<K, V> {
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> std::ops::DerefMut for HashMapWrapper<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "python")]
impl<'py, K, V> pyo3::IntoPyObject<'py> for HashMapWrapper<K, V>
where
    K: pyo3::IntoPyObject<'py> + Eq + Hash,
    V: pyo3::IntoPyObject<'py>,
{
    type Target = pyo3::types::PyDict;
    type Output = pyo3::Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: pyo3::Python<'py>) -> Result<Self::Output, Self::Error> {
        use pyo3::types::PyDictMethods; // Import trait for set_item
        let dict = pyo3::types::PyDict::new(py);
        for (k, v) in self.0 {
            dict.set_item(k, v)?;
        }
        Ok(dict)
    }
}

/* FromPyObject temporarily disabled due to PyO3 0.27 lifetime complexities.
   Use Vec<(K,V)> for input arguments if needed.
#[cfg(feature = "python")]
impl<'source, 'py, K, V> pyo3::FromPyObject<'source, 'py> for HashMapWrapper<K, V>
where
    K: pyo3::FromPyObject<'source, 'py> + Eq + Hash,
    V: pyo3::FromPyObject<'source, 'py>,
    <K as pyo3::FromPyObject<'source, 'py>>::Error: Into<pyo3::PyErr>,
    <V as pyo3::FromPyObject<'source, 'py>>::Error: Into<pyo3::PyErr>,
{
    type Error = pyo3::PyErr;

    fn extract(ob: pyo3::Borrowed<'source, 'py, pyo3::PyAny>) -> Result<Self, Self::Error> {
        use pyo3::types::{PyAnyMethods, PyDictMethods};

        let dict = ob.cast::<pyo3::types::PyDict>()?;
        let mut map = HashMap::with_capacity(dict.len());

        for (k, v) in dict.iter() {
            let key = k.extract::<K>().map_err(Into::into)?;
            let val = v.extract::<V>().map_err(Into::into)?;
            map.insert(key, val);
        }
        Ok(HashMapWrapper(map))
    }
}
*/

#[cfg(feature = "nodejs")]
impl<V> napi::bindgen_prelude::ToNapiValue for HashMapWrapper<String, V>
where
    V: napi::bindgen_prelude::ToNapiValue,
{
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        unsafe { napi::bindgen_prelude::ToNapiValue::to_napi_value(env, val.0) }
    }
}

#[cfg(feature = "nodejs")]
impl<V> napi::bindgen_prelude::FromNapiValue for HashMapWrapper<String, V>
where
    V: napi::bindgen_prelude::FromNapiValue,
{
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let map =
            unsafe { std::collections::HashMap::<String, V>::from_napi_value(env, napi_val)? };
        Ok(HashMapWrapper(map))
    }
}
