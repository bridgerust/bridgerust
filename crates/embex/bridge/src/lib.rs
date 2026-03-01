#![allow(deprecated)]
#[allow(unused_imports)]
use bridgerust::{export, new, staticmethod, BridgeError, JsonValue, Result as BridgeResult};

#[cfg(feature = "python")]
extern crate pyo3_crate as pyo3;

#[cfg(feature = "python")]
use bridgerust::pyo3::prelude::*;

#[cfg(feature = "nodejs")]
use bridgerust::napi;
#[cfg(feature = "nodejs")]
use bridgerust::napi::bindgen_prelude::*;
#[cfg(feature = "nodejs")]
use bridgerust::napi_derive::napi;

use bridge_embex::types::CollectionSchema;
use bridge_embex::types::Point as RustPoint;
use bridge_embex::EmbexClient as RustClient;
use bridge_embex_infrastructure::config::EmbexConfig;
use std::collections::HashMap;

pub enum MigrationItem {
    #[cfg(feature = "python")]
    Python(Py<PyAny>),
    #[cfg(feature = "nodejs")]
    Node(MigrationInput),
}

impl Clone for MigrationItem {
    fn clone(&self) -> Self {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "python")]
            MigrationItem::Python(p) => {
                // Clone reference with GIL
                MigrationItem::Python(Python::with_gil(|py| p.clone_ref(py)))
            }
            #[cfg(feature = "nodejs")]
            MigrationItem::Node(n) => MigrationItem::Node(n.clone()),
            _ => unreachable!(),
        }
    }
}

#[cfg(feature = "python")]
impl<'a, 'py> FromPyObject<'a, 'py> for MigrationItem {
    type Error = PyErr;
    fn extract(ob: pyo3::Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
        // Borrowed derefs to Bound. Bound is reference counted in Python, so clone it to get owned Bound, then unbind to get Py<PyAny>.
        let bound: &Bound<'py, PyAny> = &ob;
        Ok(MigrationItem::Python(bound.clone().unbind()))
    }
}

#[cfg(feature = "nodejs")]
impl FromNapiValue for MigrationItem {
    unsafe fn from_napi_value(
        env: napi::sys::napi_env,
        napi_val: napi::sys::napi_value,
    ) -> napi::Result<Self> {
        let input = MigrationInput::from_napi_value(env, napi_val)?;
        Ok(MigrationItem::Node(input))
    }
}

#[cfg(feature = "nodejs")]
impl TypeName for MigrationItem {
    fn type_name() -> &'static str {
        "MigrationItem"
    }
    fn value_type() -> napi::ValueType {
        napi::ValueType::Object
    }
}

#[cfg(feature = "nodejs")]
impl ToNapiValue for MigrationItem {
    unsafe fn to_napi_value(
        env: napi::sys::napi_env,
        val: Self,
    ) -> napi::Result<napi::sys::napi_value> {
        match val {
            #[cfg(feature = "python")]
            MigrationItem::Python(_) => Err(napi::Error::from_reason(
                "Cannot convert Python migration to Node",
            )),
            MigrationItem::Node(input) => MigrationInput::to_napi_value(env, input),
        }
    }
}

#[export]
pub struct EmbexClient {
    inner: Option<RustClient>,
    init_error: Option<String>,
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
        match RustClient::new(config) {
            Ok(client) => Self {
                inner: Some(client),
                init_error: None,
            },
            Err(e) => Self {
                inner: None,
                init_error: Some(e.to_string()),
            },
        }
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
        Ok(Self {
            inner: Some(client),
            init_error: None,
        })
    }

    pub fn collection(&self, name: String) -> BridgeResult<Collection> {
        let client = self.client()?;
        Ok(Collection {
            inner: client.collection(&name),
        })
    }

    pub async fn delete_collection(&self, name: String) -> BridgeResult<()> {
        let client = self.client()?;
        let col = client.collection(&name);
        col.delete_collection()
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;
        Ok(())
    }

    pub async fn run_migrations(&self, migrations: Vec<MigrationItem>) -> BridgeResult<()> {
        use bridge_embex::{Migration, MigrationManager};

        let adapters: Vec<Box<dyn Migration>> = migrations
            .into_iter()
            .map(|item| -> BridgeResult<Box<dyn Migration>> {
                match item {
                    #[cfg(feature = "python")]
                    MigrationItem::Python(py_migration) => Ok(Box::new(PyMigrationAdapter {
                        inner: py_migration,
                    })),
                    #[cfg(feature = "nodejs")]
                    MigrationItem::Node(m) => {
                        let up_ops: Vec<MigrationOp> = serde_json::from_value(
                            m.operations.map(|v| v.0).unwrap_or(serde_json::Value::Null),
                        )
                        .map_err(|e| {
                            BridgeError(format!("Invalid operations: {}", e)).into_platform()
                        })?;

                        let down_ops: Vec<MigrationOp> = serde_json::from_value(
                            m.down_operations
                                .map(|v| v.0)
                                .unwrap_or(serde_json::Value::Null),
                        )
                        .map_err(|e| {
                            BridgeError(format!("Invalid downOperations: {}", e)).into_platform()
                        })?;

                        Ok(Box::new(DeclarativeMigrationAdapter {
                            version: m.version,
                            up_ops,
                            down_ops,
                        }))
                    }
                }
            })
            .collect::<BridgeResult<Vec<_>>>()?;

        let client = self.client()?;
        let manager = MigrationManager::new(client.db());

        let fut = async move { manager.run_migrations(adapters).await };

        #[cfg(feature = "python")]
        if tokio::runtime::Handle::try_current().is_ok() {
            return fut
                .await
                .map_err(|e| BridgeError(e.to_string()).into_platform());
        } else {
            bridgerust::pyo3_async_runtimes::tokio::get_runtime()
                .block_on(fut)
                .map_err(|e| BridgeError(e.to_string()).into_platform())
        }

        #[cfg(not(feature = "python"))]
        {
            fut.await
                .map_err(|e| BridgeError(e.to_string()).into_platform())
        }
    }
}

impl EmbexClient {
    fn client(&self) -> BridgeResult<&RustClient> {
        self.inner.as_ref().ok_or_else(|| {
            BridgeError(
                self.init_error
                    .clone()
                    .unwrap_or_else(|| "EmbexClient is not initialized".to_string()),
            )
            .into_platform()
        })
    }
}

#[cfg(feature = "python")]
struct PyMigrationAdapter {
    inner: Py<PyAny>,
}

#[cfg(feature = "python")]
#[async_trait::async_trait]
impl bridge_embex::Migration for PyMigrationAdapter {
    fn version(&self) -> String {
        Python::with_gil(|py| {
            let fallback = format!(
                "invalid_python_migration_{:x}",
                self.inner.bind(py).as_ptr() as usize
            );
            self.inner
                .getattr(py, "version")
                .and_then(|value| value.extract(py))
                .unwrap_or(fallback)
        })
    }

    async fn up(
        &self,
        db: std::sync::Arc<dyn bridge_embex::VectorDatabase>,
    ) -> bridge_embex::Result<()> {
        let rust_client = bridge_embex::EmbexClient::from_db(db);
        let bridge_client = EmbexClient {
            inner: Some(rust_client),
            init_error: None,
        };

        let py_future = Python::with_gil(|py| {
            let py_client = Py::new(py, bridge_client).map_err(|e| {
                bridge_embex::EmbexError::Other(anyhow::anyhow!(
                    "Failed to create python client wrapper: {}",
                    e
                ))
            })?;
            self.inner
                .call_method1(py, "up", (py_client,))
                .map_err(|e| {
                    bridge_embex::EmbexError::Other(anyhow::anyhow!("Failed to call up(): {}", e))
                })
        });

        match py_future {
            Ok(awaitable) => {
                let fut = Python::with_gil(|py| {
                    bridgerust::pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py))
                });

                match fut {
                    Ok(f) => f.await.map(|_| ()).map_err(|e| {
                        bridge_embex::EmbexError::Other(anyhow::anyhow!("Python error: {}", e))
                    }),
                    Err(e) => Err(bridge_embex::EmbexError::Other(anyhow::anyhow!(
                        "Failed to convert python future: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn down(
        &self,
        db: std::sync::Arc<dyn bridge_embex::VectorDatabase>,
    ) -> bridge_embex::Result<()> {
        let rust_client = bridge_embex::EmbexClient::from_db(db);
        let bridge_client = EmbexClient {
            inner: Some(rust_client),
            init_error: None,
        };

        let py_future = Python::with_gil(|py| {
            let py_client = Py::new(py, bridge_client).map_err(|e| {
                bridge_embex::EmbexError::Other(anyhow::anyhow!(
                    "Failed to create python client wrapper: {}",
                    e
                ))
            })?;
            self.inner
                .call_method1(py, "down", (py_client,))
                .map_err(|e| {
                    bridge_embex::EmbexError::Other(anyhow::anyhow!("Failed to call down(): {}", e))
                })
        });

        match py_future {
            Ok(awaitable) => {
                let fut = Python::with_gil(|py| {
                    bridgerust::pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py))
                });

                match fut {
                    Ok(f) => f.await.map(|_| ()).map_err(|e| {
                        bridge_embex::EmbexError::Other(anyhow::anyhow!("Python error: {}", e))
                    }),
                    Err(e) => Err(bridge_embex::EmbexError::Other(anyhow::anyhow!(
                        "Failed to convert python future: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(e),
        }
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

    pub fn build_query(&self) -> QueryBuilder {
        QueryBuilder {
            collection: self.inner.clone(),
            inner: Some(self.inner.build_query()),
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

#[export]
pub struct QueryBuilder {
    collection: bridge_embex::client::Collection,
    inner: Option<bridge_embex::QueryBuilder>,
}

#[export]
impl QueryBuilder {
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

    pub fn filter(&mut self, filter: JsonValue) -> BridgeResult<()> {
        let rust_filter: bridge_embex::types::Filter = serde_json::from_value(filter.0)
            .map_err(|e| BridgeError(format!("Invalid filter: {}", e)).into_platform())?;

        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.filter(rust_filter));
        }
        Ok(())
    }

    pub fn aggregation(&mut self, agg_type: String) -> BridgeResult<()> {
        let agg = match agg_type.as_str() {
            "count" => bridge_embex::types::Aggregation::Count,
            _ => return Err(BridgeError("Invalid aggregation type".to_string()).into_platform()),
        };

        if let Some(inner) = self.inner.take() {
            self.inner = Some(inner.aggregate(agg));
        }
        Ok(())
    }

    pub async fn execute(&mut self) -> BridgeResult<SearchResponse> {
        let inner = self
            .inner
            .take()
            .ok_or(BridgeError("Query already executed".to_string()).into_platform())?;

        let result = self
            .collection
            .query(inner)
            .await
            .map_err(|e| BridgeError(e.to_string()).into_platform())?;

        Ok(SearchResponse::from_rust(result))
    }
}

#[cfg(feature = "nodejs")]
#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MigrationOp {
    CreateCollection {
        schema: bridge_embex::types::CollectionSchema,
    },
    DeleteCollection {
        name: String,
    },
}

#[cfg(feature = "nodejs")]
#[export(object)]
#[derive(Clone)]
pub struct MigrationInput {
    pub version: String,
    pub operations: Option<JsonValue>,
    pub down_operations: Option<JsonValue>,
}

#[cfg(feature = "nodejs")]
#[derive(Clone)]
pub struct DeclarativeMigrationAdapter {
    version: String,
    up_ops: Vec<MigrationOp>,
    down_ops: Vec<MigrationOp>,
}

#[cfg(feature = "nodejs")]
#[async_trait::async_trait]
impl bridge_embex::Migration for DeclarativeMigrationAdapter {
    fn version(&self) -> String {
        self.version.clone()
    }

    async fn up(
        &self,
        db: std::sync::Arc<dyn bridge_embex::VectorDatabase>,
    ) -> bridge_embex::Result<()> {
        for op in &self.up_ops {
            match op {
                MigrationOp::CreateCollection { schema } => {
                    db.create_collection(schema).await?;
                }
                MigrationOp::DeleteCollection { name } => {
                    db.delete_collection(name).await?;
                }
            }
        }
        Ok(())
    }

    async fn down(
        &self,
        db: std::sync::Arc<dyn bridge_embex::VectorDatabase>,
    ) -> bridge_embex::Result<()> {
        for op in &self.down_ops {
            match op {
                MigrationOp::CreateCollection { schema } => {
                    db.create_collection(schema).await?;
                }
                MigrationOp::DeleteCollection { name } => {
                    db.delete_collection(name).await?;
                }
            }
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Separate Impl Blocks for Feature-Specific Methods
// -----------------------------------------------------------------------------

// Methods moved to main impl block

// -----------------------------------------------------------------------------
// CLI Entry Points
// -----------------------------------------------------------------------------

#[cfg(any(feature = "python", feature = "nodejs"))]
fn normalize_cli_args(mut args: Vec<String>) -> Vec<String> {
    if args.is_empty() {
        args.push("embex".to_string());
        return args;
    }

    if args[0] != "embex" {
        args.insert(0, "embex".to_string());
    }

    args
}

#[cfg(feature = "python")]
#[pyfunction(name = "cli")]
pub fn cli_main<'p>(py: Python<'p>, args: Vec<String>) -> PyResult<Bound<'p, PyAny>> {
    let cli_args = normalize_cli_args(args);
    bridgerust::pyo3_async_runtimes::tokio::future_into_py::<_, ()>(py, async move {
        embex_cli::run(cli_args)
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        Ok(())
    })
}

#[cfg(feature = "nodejs")]
#[napi]
pub async fn cli(args: Vec<String>) -> napi::Result<()> {
    let cli_args = normalize_cli_args(args);
    embex_cli::run(cli_args)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[cfg(feature = "python")]
#[bridgerust::pyo3::pymodule]
fn embex(
    m: &bridgerust::pyo3::Bound<'_, bridgerust::pyo3::types::PyModule>,
) -> bridgerust::pyo3::PyResult<()> {
    m.add_class::<EmbexClient>()?;
    m.add_class::<Collection>()?;
    m.add_class::<Point>()?;
    m.add_class::<SearchResponse>()?;
    m.add_class::<SearchResult>()?;
    m.add_class::<SearchBuilder>()?;
    m.add_class::<QueryBuilder>()?;
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    Ok(())
}
