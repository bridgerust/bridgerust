#![allow(deprecated)]
#[allow(unused_imports)]
use bridgerust::{export, new, staticmethod};

#[cfg(feature = "python")]
extern crate pyo3_crate as pyo3;
use bridge_embex::types::CollectionSchema;
use bridge_embex::types::Point as RustPoint;
use bridge_embex::EmbexClient as RustClient;
use bridge_embex_infrastructure::config::EmbexConfig;
use serde_json::Value as SerdeValue;
use std::collections::HashMap;

#[cfg(feature = "python")]
use pyo3_crate::prelude::*;
#[cfg(feature = "python")]
use pyo3_crate::types::{PyDict, PyList};

#[cfg(feature = "nodejs")]
use napi::bindgen_prelude::*;

#[derive(Debug)]
pub struct BridgeError(String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BridgeError {}

impl AsRef<str> for BridgeError {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for BridgeError {
    fn from(s: String) -> Self {
        BridgeError(s)
    }
}

#[cfg(feature = "python")]
impl From<BridgeError> for PyErr {
    fn from(e: BridgeError) -> Self {
        use pyo3_crate::exceptions::PyRuntimeError;
        PyRuntimeError::new_err(e.0)
    }
}

#[cfg(feature = "nodejs")]
impl From<BridgeError> for napi::Error {
    fn from(e: BridgeError) -> Self {
        napi::Error::from_reason(e.0)
    }
}

impl BridgeError {
    #[cfg(all(feature = "python", not(feature = "nodejs")))]
    pub fn into_platform(self) -> PyErr {
        self.into()
    }

    #[cfg(all(feature = "nodejs", not(feature = "python")))]
    pub fn into_platform(self) -> napi::Error {
        self.into()
    }

    #[cfg(any(
        all(feature = "python", feature = "nodejs"),
        not(any(feature = "python", feature = "nodejs"))
    ))]
    pub fn into_platform(self) -> BridgeError {
        self
    }
}

// Helper removed (unused)

// Result alias for cross-platform support
#[cfg(all(feature = "python", not(feature = "nodejs")))]
pub type BridgeResult<T> = PyResult<T>;

#[cfg(all(feature = "nodejs", not(feature = "python")))]
pub type BridgeResult<T> = napi::Result<T>;

#[cfg(any(
    all(feature = "python", feature = "nodejs"),
    not(any(feature = "python", feature = "nodejs"))
))]
pub type BridgeResult<T> = std::result::Result<T, BridgeError>;

#[derive(Clone, Debug)]
pub struct JsonValue(pub SerdeValue);

#[cfg(feature = "python")]
fn py_to_json<'py>(ob: &Bound<'py, PyAny>) -> PyResult<SerdeValue> {
    if ob.is_none() {
        return Ok(SerdeValue::Null);
    }

    if let Ok(b) = ob.extract::<bool>() {
        return Ok(SerdeValue::Bool(b));
    }

    if let Ok(i) = ob.extract::<i64>() {
        return Ok(SerdeValue::Number(serde_json::Number::from(i)));
    }

    if let Ok(f) = ob.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(SerdeValue::Number(n));
        }
    }

    if let Ok(s) = ob.extract::<String>() {
        return Ok(SerdeValue::String(s));
    }

    // Using downcast (deprecated but works) or cast implies trait availability
    if let Ok(list) = ob.downcast::<PyList>() {
        let mut arr = Vec::new();
        for item in list.iter() {
            arr.push(py_to_json(&item)?);
        }
        return Ok(SerdeValue::Array(arr));
    }

    if let Ok(dict) = ob.downcast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict.iter() {
            let key = k.extract::<String>()?;
            map.insert(key, py_to_json(&v)?);
        }
        return Ok(SerdeValue::Object(map));
    }

    // Fallback: try string conversion
    Ok(SerdeValue::String(ob.to_string()))
}

#[cfg(feature = "python")]
fn json_to_py<'py>(py: Python<'py>, v: &SerdeValue) -> PyResult<Bound<'py, PyAny>> {
    use pyo3_crate::types::{PyFloat, PyString};
    match v {
        SerdeValue::Null => Ok(py.None().into_bound(py)),
        SerdeValue::Bool(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
        SerdeValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.into_any())
            } else if let Some(f) = n.as_f64() {
                Ok(PyFloat::new(py, f).into_any())
            } else {
                Ok(PyString::new(py, &n.to_string()).into_any())
            }
        }
        SerdeValue::String(s) => Ok(PyString::new(py, s.as_str()).into_any()),
        SerdeValue::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        SerdeValue::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

#[cfg(feature = "python")]
impl<'a, 'py> FromPyObject<'a, 'py> for JsonValue {
    type Error = PyErr;
    fn extract(ob: pyo3_crate::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        let bound = ob.as_borrowed().to_owned();
        let v = py_to_json(&bound)?;
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
impl TypeName for JsonValue {
    fn type_name() -> &'static str {
        "any"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}

#[cfg(feature = "nodejs")]
impl FromNapiValue for JsonValue {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let val: SerdeValue = FromNapiValue::from_napi_value(env, napi_val)?;
        Ok(JsonValue(val))
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for JsonValue {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        ToNapiValue::to_napi_value(env, val.0)
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for &JsonValue {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        ToNapiValue::to_napi_value(env, &val.0)
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for &mut JsonValue {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        ToNapiValue::to_napi_value(env, &val.0)
    }
}

// Implement From/Into for Rust <-> Bridge conversions
impl From<SerdeValue> for JsonValue {
    fn from(v: SerdeValue) -> Self {
        JsonValue(v)
    }
}

impl From<JsonValue> for SerdeValue {
    fn from(val: JsonValue) -> Self {
        val.0
    }
}

#[export]
pub struct EmbexClient {
    inner: RustClient,
}

#[export]
impl EmbexClient {
    #[constructor]
    pub fn new(provider: String, url: String, api_key: Option<String>) -> Self {
        let config = EmbexConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
            idle_timeout_secs: 90,
            pool_size: 10,
        };
        // Using expect() to bypass Napi fallible constructor limitation
        let client = RustClient::new(config)
            .expect("Failed to initialize EmbexClient: Invalid configuration");
        Self { inner: client }
    }

    pub async fn new_async(
        provider: String,
        url: String,
        api_key: Option<String>,
    ) -> BridgeResult<Self> {
        let config = EmbexConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
            idle_timeout_secs: 90,
            pool_size: 10,
        };
        let client = RustClient::new_async(config)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(Self { inner: client })
    }

    pub fn collection(&self, name: String) -> Collection {
        Collection {
            inner: self.inner.collection(&name),
        }
    }

    pub async fn delete_collection(&self, name: String) -> BridgeResult<()> {
        let col = self.inner.collection(&name);
        col.delete_collection()
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    #[cfg(feature = "python")]
    pub fn run_migrations(&self, _py: Python, migrations: Vec<Py<PyAny>>) -> BridgeResult<()> {
        use async_trait::async_trait;
        use bridge_embex::{Migration, MigrationManager};
        use std::sync::Arc;

        struct PyMigrationAdapter {
            inner: Py<PyAny>,
        }

        #[async_trait]
        impl Migration for PyMigrationAdapter {
            fn version(&self) -> String {
                Python::with_gil(|py| {
                    self.inner
                        .call_method0(py, "version")
                        .expect("Migration must have version() method")
                        .extract(py)
                        .expect("version() must return string")
                })
            }

            async fn up(
                &self,
                db: Arc<dyn bridge_embex::VectorDatabase>,
            ) -> bridge_embex::Result<()> {
                let rust_client = bridge_embex::EmbexClient::from_db(db);
                let bridge_client = EmbexClient { inner: rust_client };

                let py_future = Python::with_gil(|py| {
                    let py_client =
                        Py::new(py, bridge_client).expect("Failed to create python client wrapper");
                    self.inner.call_method1(py, "up", (py_client,))
                });

                match py_future {
                    Ok(awaitable) => {
                        let fut = Python::with_gil(|py| {
                            bridgerust::pyo3_async_runtimes::tokio::into_future(
                                awaitable.into_bound(py),
                            )
                        })
                        .map_err(|e| {
                            bridge_embex::EmbexError::Other(anyhow::anyhow!(
                                "Python migration failed: {}",
                                e
                            ))
                        })?;

                        fut.await.map_err(|e| {
                            bridge_embex::EmbexError::Other(anyhow::anyhow!(
                                "Python migration failed: {}",
                                e
                            ))
                        })?;
                        Ok(())
                    }
                    Err(e) => Err(bridge_embex::EmbexError::Other(anyhow::anyhow!(
                        "Python error: {}",
                        e
                    ))),
                }
            }

            async fn down(
                &self,
                db: Arc<dyn bridge_embex::VectorDatabase>,
            ) -> bridge_embex::Result<()> {
                let rust_client = bridge_embex::EmbexClient::from_db(db);
                let bridge_client = EmbexClient { inner: rust_client };

                let py_future = Python::with_gil(|py| {
                    let py_client =
                        Py::new(py, bridge_client).expect("Failed to create python client wrapper");
                    self.inner.call_method1(py, "down", (py_client,))
                });

                match py_future {
                    Ok(awaitable) => {
                        let fut = Python::with_gil(|py| {
                            bridgerust::pyo3_async_runtimes::tokio::into_future(
                                awaitable.into_bound(py),
                            )
                        })
                        .map_err(|e| {
                            bridge_embex::EmbexError::Other(anyhow::anyhow!(
                                "Python migration failed: {}",
                                e
                            ))
                        })?;

                        fut.await.map_err(|e| {
                            bridge_embex::EmbexError::Other(anyhow::anyhow!(
                                "Python migration failed: {}",
                                e
                            ))
                        })?;
                        Ok(())
                    }
                    Err(e) => Err(bridge_embex::EmbexError::Other(anyhow::anyhow!(
                        "Python error: {}",
                        e
                    ))),
                }
            }
        }

        let adapters: Vec<Box<dyn Migration>> = migrations
            .into_iter()
            .map(|m| Box::new(PyMigrationAdapter { inner: m }) as Box<dyn Migration>)
            .collect();

        let manager = MigrationManager::new(self.inner.db());

        let fut = async move {
            manager
                .run_migrations(adapters)
                .await
                .map_err(|e| BridgeError(e.to_string()).into_platform())
        };

        bridgerust::pyo3_async_runtimes::tokio::get_runtime().block_on(fut)
    }
}

#[export]
pub struct Collection {
    inner: bridge_embex::client::Collection,
}

#[export]
impl Collection {
    pub async fn create(&self, dimension: usize, distance: String) -> BridgeResult<()> {
        let metric = match distance.as_str() {
            "cosine" => bridge_embex::types::DistanceMetric::Cosine,
            "euclidean" => bridge_embex::types::DistanceMetric::Euclidean,
            "dot" => bridge_embex::types::DistanceMetric::Dot,
            _ => return Err(BridgeError("Invalid distance metric".to_string()).into_platform()),
        };

        let schema = CollectionSchema {
            name: self.inner.name().to_string(),
            dimension,
            metric,
        };

        self.inner
            .create(schema)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn create_auto(
        &self,
        dimension: Option<usize>,
        distance: Option<String>,
    ) -> BridgeResult<()> {
        let metric_str = distance.as_deref().unwrap_or("cosine");
        let metric = match metric_str {
            "cosine" => bridge_embex::types::DistanceMetric::Cosine,
            "euclidean" => bridge_embex::types::DistanceMetric::Euclidean,
            "dot" => bridge_embex::types::DistanceMetric::Dot,
            _ => return Err(BridgeError("Invalid distance metric".to_string()).into_platform()),
        };

        self.inner
            .create_auto(dimension, metric)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn insert(&self, points: Vec<Point>) -> BridgeResult<()> {
        let rust_points: Vec<RustPoint> = points.into_iter().map(|p| p.into_rust()).collect();
        self.inner
            .insert(rust_points)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    #[cfg(feature = "python")]
    pub fn insert_stream(
        &self,
        _py: Python,
        stream: Py<PyAny>,
        batch_size: Option<usize>,
        parallel: Option<usize>,
    ) -> BridgeResult<()> {
        use bridgerust::stream::python::PyStreamAdapter;
        use futures::StreamExt;

        let adapter = PyStreamAdapter::<Py<Point>>::new(stream);

        let stream = adapter.map(|res| match res {
            Ok(py_point) => Python::with_gil(|py| {
                let point = py_point.borrow(py);
                Ok(point.clone().into_rust())
            }),
            Err(e) => Err(bridge_embex::EmbexError::Other(anyhow::anyhow!(
                "Stream error: {}",
                e
            ))),
        });

        let rust_client = self.inner.clone();

        let fut = async move {
            rust_client
                .insert_stream(stream, batch_size.unwrap_or(1000), parallel)
                .await
                .map_err(|e| BridgeError(e.to_string()).into_platform())
        };

        bridgerust::pyo3_async_runtimes::tokio::get_runtime().block_on(fut)
    }

    pub async fn insert_batch(
        &self,
        points: Vec<Point>,
        batch_size: Option<usize>,
        parallel: Option<usize>,
    ) -> BridgeResult<()> {
        let rust_points: Vec<RustPoint> = points.into_iter().map(|p| p.into_rust()).collect();
        self.inner
            .insert_batch(rust_points, batch_size.unwrap_or(1000), parallel)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn delete(&self, ids: Vec<String>) -> BridgeResult<()> {
        self.inner
            .delete(ids)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn delete_collection(&self) -> BridgeResult<()> {
        self.inner
            .delete_collection()
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn search(
        &self,
        vector: Vec<f64>,
        top_k: usize,
        filter: Option<JsonValue>,
        include_metadata: Option<bool>,
        include_vector: Option<bool>,
    ) -> BridgeResult<SearchResponse> {
        let rust_vector: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();
        let mut builder = self.inner.search(rust_vector).limit(top_k);
        if let Some(inc_meta) = include_metadata {
            builder = builder.include_metadata(inc_meta);
        } else {
            builder = builder.include_metadata(true);
        }

        if let Some(inc_vec) = include_vector {
            builder = builder.include_vector(inc_vec);
        } else {
            builder = builder.include_vector(false);
        }

        if let Some(f) = filter {
            let rust_filter: bridge_embex::types::Filter = serde_json::from_value(f.0)
                .map_err(|e| BridgeError(format!("Invalid filter: {}", e)).into_platform())?;
            builder = builder.filter(rust_filter);
        }

        let result = builder
            .execute()
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(SearchResponse::from_rust(result))
    }

    pub fn build_search(&self, vector: Vec<f64>) -> SearchBuilder {
        let rust_vector: Vec<f32> = vector.into_iter().map(|v| v as f32).collect();
        SearchBuilder {
            inner: Some(self.inner.search(rust_vector)),
        }
    }
}

#[export(object)]
#[derive(Clone)]
pub struct Point {
    pub id: String,
    pub vector: Vec<f64>,
    pub metadata: Option<JsonValue>,
}

impl Point {
    fn into_rust(self) -> RustPoint {
        let metadata = self.metadata.map(|m| match m.0 {
            serde_json::Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        });

        RustPoint {
            id: self.id,
            vector: self.vector.into_iter().map(|v| v as f32).collect(),
            metadata,
        }
    }
}

#[export(object)]
#[derive(Clone)]
pub struct SearchResponse {
    #[readonly]
    pub results: Vec<SearchResult>,
    #[readonly]
    pub aggregations: Option<JsonValue>,
}

impl SearchResponse {
    fn from_rust(r: bridge_embex::types::SearchResponse) -> Self {
        let aggregations = if r.aggregations.is_empty() {
            None
        } else {
            Some(JsonValue(serde_json::Value::Object(
                r.aggregations.into_iter().collect(),
            )))
        };
        Self {
            results: r.results.into_iter().map(SearchResult::from_rust).collect(),
            aggregations,
        }
    }
}

#[export(object)]
#[derive(Clone)]
pub struct SearchResult {
    #[readonly]
    pub id: String,
    #[readonly]
    pub score: f64,
    #[readonly]
    pub vector: Option<Vec<f64>>,
    #[readonly]
    pub metadata: Option<JsonValue>,
}

impl SearchResult {
    fn from_rust(r: bridge_embex::types::SearchResult) -> Self {
        let metadata = r
            .metadata
            .map(|m| JsonValue(serde_json::Value::Object(m.into_iter().collect())));

        Self {
            id: r.id,
            score: r.score as f64,
            vector: r.vector.map(|v| v.into_iter().map(|x| x as f64).collect()),
            metadata,
        }
    }
}

#[export]
pub struct SearchBuilder {
    inner: Option<bridge_embex::client::SearchBuilder>,
}

#[export]
impl SearchBuilder {
    pub fn limit(&mut self, limit: usize) {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.limit(limit));
        }
    }

    pub fn offset(&mut self, offset: usize) {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.offset(offset));
        }
    }

    pub fn include_vector(&mut self, include: bool) {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.include_vector(include));
        }
    }

    pub fn include_metadata(&mut self, include: bool) {
        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.include_metadata(include));
        }
    }

    pub async fn execute(&mut self) -> BridgeResult<SearchResponse> {
        let inner = self
            .inner
            .take()
            .ok_or(BridgeError("Search already executed".to_string()).into_platform())?;
        let result = inner
            .execute()
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(SearchResponse::from_rust(result))
    }
}

#[cfg(feature = "python")]
#[pyo3::pymodule]
fn embex(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
    m.add_class::<EmbexClient>()?;
    m.add_class::<Collection>()?;
    m.add_class::<Point>()?;
    m.add_class::<SearchResponse>()?;
    m.add_class::<SearchResult>()?;
    m.add_class::<SearchBuilder>()?;
    Ok(())
}
