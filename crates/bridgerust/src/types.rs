use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonValue(pub serde_json::Value);

#[cfg(feature = "python")]
use crate::pyo3::prelude::*;
#[cfg(feature = "python")]
use crate::pyo3::types::{PyDict, PyFloat, PyList, PyString};

#[cfg(feature = "python")]
impl<'a, 'py> FromPyObject<'a, 'py> for JsonValue {
    type Error = PyErr;
    fn extract(ob: crate::pyo3::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Borrowed<T> derefs to Bound<T>
        let bound: &Bound<'py, PyAny> = &ob;
        let v = py_to_json(bound)?;
        Ok(JsonValue(v))
    }
}

#[cfg(feature = "python")]
impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> std::result::Result<Self::Output, Self::Error> {
        json_to_py(py, &self.0)
    }
}

#[cfg(feature = "nodejs")]
use crate::napi::bindgen_prelude::*;

#[cfg(feature = "nodejs")]
impl TypeName for JsonValue {
    fn type_name() -> &'static str {
        "any"
    }
    fn value_type() -> ValueType {
        ValueType::Object
    }
}

#[cfg(feature = "nodejs")]
impl FromNapiValue for JsonValue {
    unsafe fn from_napi_value(
        env: crate::napi::sys::napi_env,
        napi_val: crate::napi::sys::napi_value,
    ) -> crate::napi::Result<Self> {
        let val: serde_json::Value = unsafe { FromNapiValue::from_napi_value(env, napi_val)? };
        Ok(JsonValue(val))
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for JsonValue {
    unsafe fn to_napi_value(
        env: crate::napi::sys::napi_env,
        val: Self,
    ) -> crate::napi::Result<crate::napi::sys::napi_value> {
        unsafe { ToNapiValue::to_napi_value(env, val.0) }
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for &JsonValue {
    unsafe fn to_napi_value(
        env: crate::napi::sys::napi_env,
        val: Self,
    ) -> crate::napi::Result<crate::napi::sys::napi_value> {
        // serde_json::Value implements ToNapiValue, but &serde_json::Value might not.
        // We clone it to pass ownership to to_napi_value.
        unsafe { ToNapiValue::to_napi_value(env, val.0.clone()) }
    }
}

#[cfg(feature = "python")]
fn py_to_json(ob: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if ob.is_none() {
        return Ok(serde_json::Value::Null);
    }

    if let Ok(b) = ob.extract::<bool>() {
        return Ok(serde_json::Value::Bool(b));
    }

    if let Ok(i) = ob.extract::<i64>() {
        return Ok(serde_json::Value::Number(serde_json::Number::from(i)));
    }

    if let Ok(f) = ob.extract::<f64>()
        && let Some(n) = serde_json::Number::from_f64(f)
    {
        return Ok(serde_json::Value::Number(n));
    }

    if let Ok(s) = ob.extract::<String>() {
        return Ok(serde_json::Value::String(s));
    }

    if let Ok(list) = ob.cast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json(&item)?);
        }
        return Ok(serde_json::Value::Array(arr));
    }

    if let Ok(dict) = ob.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key = k.extract::<String>()?;
            map.insert(key, py_to_json(&v)?);
        }
        return Ok(serde_json::Value::Object(map));
    }

    // Fallback: try string conversion
    Ok(serde_json::Value::String(ob.to_string()))
}

#[cfg(feature = "python")]
fn json_to_py<'py>(py: Python<'py>, v: &serde_json::Value) -> PyResult<Bound<'py, PyAny>> {
    match v {
        serde_json::Value::Null => Ok(py.None().into_bound(py)),
        serde_json::Value::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any())
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new(py, f).into_any())
            } else {
                Ok(PyString::new(py, &n.to_string()).into_any())
            }
        }
        serde_json::Value::String(s) => Ok(PyString::new(py, s.as_str()).into_any()),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into_any())
        }
    }
}
