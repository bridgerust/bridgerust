use bridge_embex::EmbexClient as RustClient;
use bridge_embex::Migration;
use bridge_embex::MigrationManager;
use bridge_embex::VectorDatabase;
use bridge_embex::config::EmbexConfig;
use bridge_embex::types::{Filter, Point, SearchResponse as RustSearchResponse};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PyList};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use pyo3::create_exception;

create_exception!(embex, EmbexError, pyo3::exceptions::PyException);
create_exception!(embex, EmbexConfigError, EmbexError);
create_exception!(embex, EmbexDatabaseError, EmbexError);
create_exception!(embex, EmbexSerializationError, EmbexError);
create_exception!(embex, EmbexValidationError, EmbexError);

fn to_py_err(err: bridge_embex::error::EmbexError) -> PyErr {
    use bridge_embex::error::EmbexError::*;
    match err {
        Config(e) => EmbexConfigError::new_err(e.to_string()),
        Database(e) => EmbexDatabaseError::new_err(e),
        Serialization(e) => EmbexSerializationError::new_err(e.to_string()),
        Validation(e) => EmbexValidationError::new_err(e),
        other => EmbexError::new_err(other.to_string()),
    }
}

struct PyMigrationAdapter {
    inner: Py<PyAny>,
}

#[async_trait::async_trait]
impl Migration for PyMigrationAdapter {
    fn version(&self) -> String {
        Python::attach(|py| {
            self.inner
                .getattr(py, "version")
                .expect("Migration must have version")
                .extract(py)
                .expect("Version must be string")
        })
    }

    async fn up(&self, db: Arc<dyn VectorDatabase>) -> bridge_embex::error::Result<()> {
        let client = EmbexClient {
            inner: RustClient::from_db(db),
        };

        let fut = Python::attach(|py| {
            let py_client = Py::new(py, client).expect("Failed to create python client");
            let awaitable = self
                .inner
                .call_method1(py, "up", (py_client,))
                .expect("Failed to call up");
            pyo3_async_runtimes::tokio::into_future(awaitable.bind(py).clone())
                .expect("Failed to convert future")
        });

        fut.await
            .map_err(|e| bridge_embex::error::EmbexError::Validation(e.to_string()))?;
        Ok(())
    }

    async fn down(&self, db: Arc<dyn VectorDatabase>) -> bridge_embex::error::Result<()> {
        let client = EmbexClient {
            inner: RustClient::from_db(db),
        };

        let fut = Python::attach(|py| {
            let py_client = Py::new(py, client).expect("Failed to create python client");
            let awaitable = self
                .inner
                .call_method1(py, "down", (py_client,))
                .expect("Failed to call down");
            pyo3_async_runtimes::tokio::into_future(awaitable.bind(py).clone())
                .expect("Failed to convert future")
        });

        fut.await
            .map_err(|e| bridge_embex::error::EmbexError::Validation(e.to_string()))?;
        Ok(())
    }
}

/// Main client for interacting with the Embex vector database.
#[pyclass]
struct EmbexClient {
    inner: RustClient,
}

#[pymethods]
impl EmbexClient {
    /// Create a new Embex client.
    ///
    /// Args:
    ///     provider: The database provider (e.g., "qdrant", "pinecone").
    ///     url: The connection URL.
    ///     api_key: Optional API key.
    #[new]
    #[pyo3(signature = (provider, url, api_key=None))]
    fn new(provider: String, url: String, api_key: Option<String>) -> PyResult<Self> {
        let config = EmbexConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
            idle_timeout_secs: 90,
            pool_size: 10,
        };

        let client = RustClient::new(config).map_err(to_py_err)?;

        Ok(Self { inner: client })
    }

    /// Get a collection by name.
    fn collection(&self, name: String) -> Collection {
        Collection {
            inner: self.inner.collection(&name),
        }
    }

    /// Create a new Embex client asynchronously.
    ///
    /// Required for providers that need async initialization (milvus, pgvector, lancedb).
    ///
    /// Args:
    ///     provider: The database provider.
    ///     url: The connection URL.
    ///     api_key: Optional API key.
    #[staticmethod]
    #[pyo3(signature = (provider, url, api_key=None))]
    fn new_async<'p>(
        py: Python<'p>,
        provider: String,
        url: String,
        api_key: Option<String>,
    ) -> PyResult<Bound<'p, PyAny>> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let config = EmbexConfig {
                provider,
                url,
                api_key,
                timeout_ms: None,
                options: Default::default(),
                idle_timeout_secs: 90,
                pool_size: 10,
            };

            let client = RustClient::new_async(config).await.map_err(to_py_err)?;

            Ok(EmbexClient { inner: client })
        })
    }

    /// Run database migrations.
    ///
    /// Args:
    ///     migrations: A list of migration objects.
    #[pyo3(signature = (migrations))]
    fn run_migrations<'p>(
        &self,
        py: Python<'p>,
        migrations: Vec<Py<PyAny>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        // MigrationManager takes Arc<dyn VectorDatabase>.
        // Let's assume I can get db from RustClient via a new method `db()` I should add.
        // Or I can add `db()` to `EmbexClient` in client.rs.

        // Actually, let's look at `EmbexClient::collection`. It creates `Collection` which holds `db`.
        // `EmbexClient` has `db` field.
        // I'll add `pub fn db(&self) -> Arc<dyn VectorDatabase>` to `RustClient`.

        let db = self.inner.db();
        let manager = MigrationManager::new(db);

        let rust_migrations: Vec<Box<dyn Migration>> = migrations
            .into_iter()
            .map(|m| Box::new(PyMigrationAdapter { inner: m }) as Box<dyn Migration>)
            .collect();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            manager
                .run_migrations(rust_migrations)
                .await
                .map_err(to_py_err)
        })
    }
}

/// A collection in the vector database.
#[pyclass]
struct Collection {
    inner: bridge_embex::client::Collection,
}

#[pyclass(name = "Point")]
struct EmbexPoint {
    #[pyo3(get, set)]
    id: String,
    #[pyo3(get, set)]
    vector: Vec<f32>,
    #[pyo3(get, set)]
    metadata: Option<HashMap<String, Py<PyAny>>>,
}

#[pymethods]
impl EmbexPoint {
    #[new]
    fn new(id: String, vector: Vec<f32>, metadata: Option<HashMap<String, Py<PyAny>>>) -> Self {
        Self {
            id,
            vector,
            metadata,
        }
    }

    fn dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("vector", &self.vector)?;
        dict.set_item("metadata", &self.metadata)?;
        Ok(dict.into())
    }

    fn model_dump(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.dict(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "Point(id='{}', vector=[...], metadata={})",
            self.id,
            if self.metadata.is_some() { "yes" } else { "no" }
        )
    }
}

#[pyclass]
struct SearchResult {
    #[pyo3(get)]
    id: String,
    #[pyo3(get)]
    score: f32,
    #[pyo3(get)]
    vector: Option<Vec<f32>>,
    #[pyo3(get)]
    metadata: Option<HashMap<String, Py<PyAny>>>,
}

#[pymethods]
impl SearchResult {
    fn dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("id", &self.id)?;
        dict.set_item("score", self.score)?;
        dict.set_item("vector", &self.vector)?;
        dict.set_item("metadata", &self.metadata)?;
        Ok(dict.into())
    }

    fn model_dump(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.dict(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "SearchResult(id='{}', score={:.4}, metadata={})",
            self.id,
            self.score,
            if self.metadata.is_some() { "yes" } else { "no" }
        )
    }
}

impl Clone for SearchResult {
    fn clone(&self) -> Self {
        Python::attach(|py| Self {
            id: self.id.clone(),
            score: self.score,
            vector: self.vector.clone(),
            metadata: self.metadata.as_ref().map(|m| {
                let mut new_map = HashMap::with_capacity(m.len());
                for (k, v) in m {
                    new_map.insert(k.clone(), v.clone_ref(py));
                }
                new_map
            }),
        })
    }
}

#[pyclass]
struct SearchResponse {
    #[pyo3(get)]
    results: Vec<SearchResult>,
    #[pyo3(get)]
    aggregations: HashMap<String, Py<PyAny>>,
}

#[pymethods]
impl SearchResponse {
    fn dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        let results_list = PyList::new(py, self.results.iter().map(|r| r.dict(py).unwrap()))?;
        dict.set_item("results", results_list)?;
        dict.set_item("aggregations", &self.aggregations)?;
        Ok(dict.into())
    }

    fn model_dump(&self, py: Python) -> PyResult<Py<PyAny>> {
        self.dict(py)
    }

    fn __repr__(&self) -> String {
        format!(
            "SearchResponse(results={}, aggregations={})",
            self.results.len(),
            self.aggregations.len()
        )
    }

    fn __len__(&self) -> usize {
        self.results.len()
    }
}

#[pyclass]
struct SearchBuilder {
    inner: Option<bridge_embex::client::SearchBuilder>,
}

#[pymethods]
impl SearchBuilder {
    fn limit(slf: PyRefMut<'_, Self>, limit: usize) -> PyRefMut<'_, Self> {
        let mut slf = slf;
        if let Some(inner) = slf.inner.take() {
            slf.inner = Some(inner.limit(limit));
        }
        slf
    }

    fn offset(slf: PyRefMut<'_, Self>, offset: usize) -> PyRefMut<'_, Self> {
        let mut slf = slf;
        if let Some(inner) = slf.inner.take() {
            slf.inner = Some(inner.offset(offset));
        }
        slf
    }

    fn include_vector(slf: PyRefMut<'_, Self>, include: bool) -> PyRefMut<'_, Self> {
        let mut slf = slf;
        if let Some(inner) = slf.inner.take() {
            slf.inner = Some(inner.include_vector(include));
        }
        slf
    }

    fn include_metadata(slf: PyRefMut<'_, Self>, include: bool) -> PyRefMut<'_, Self> {
        let mut slf = slf;
        if let Some(inner) = slf.inner.take() {
            slf.inner = Some(inner.include_metadata(include));
        }
        slf
    }

    fn execute<'p>(&mut self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Search already executed")
        })?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = inner.execute().await.map_err(to_py_err)?;

            Python::attach(|py| convert_search_response(py, res))
        })
    }
}

#[pymethods]
impl Collection {
    /// Insert points into the collection.
    ///
    /// Args:
    ///     points: A list of Point objects.
    fn insert<'p>(
        &self,
        py: Python<'p>,
        points: Vec<PyRef<'p, EmbexPoint>>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();

        let mut rust_points = Vec::with_capacity(points.len());
        for p in points {
            let mut metadata = None;
            if let Some(py_meta) = &p.metadata {
                let mut meta_map = HashMap::new();
                for (k, v) in py_meta {
                    meta_map.insert(k.clone(), py_to_json(py, v)?);
                }
                metadata = Some(meta_map);
            }

            let point = Point {
                id: p.id.clone(),
                vector: p.vector.clone(),
                metadata,
            };
            rust_points.push(point);
        }

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.insert(rust_points).await.map_err(to_py_err)
        })
    }

    /// Insert points in parallel batches.
    ///
    /// Args:
    ///     points: List of points to insert.
    ///     batch_size: Number of points per batch (default: 1000).
    ///     parallel: Max concurrent requests (default: 1).
    #[pyo3(signature = (points, batch_size=None, parallel=None))]
    fn insert_batch<'p>(
        &self,
        py: Python<'p>,
        points: Vec<PyRef<'p, EmbexPoint>>,
        batch_size: Option<usize>,
        parallel: Option<usize>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let size = batch_size.unwrap_or(1000);

        let mut rust_points = Vec::with_capacity(points.len());
        for p in points {
            let mut metadata = None;
            if let Some(py_meta) = &p.metadata {
                let mut meta_map = HashMap::new();
                for (k, v) in py_meta {
                    meta_map.insert(k.clone(), py_to_json(py, v)?);
                }
                metadata = Some(meta_map);
            }

            let point = Point {
                id: p.id.clone(),
                vector: p.vector.clone(),
                metadata,
            };
            rust_points.push(point);
        }

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .insert_batch(rust_points, size, parallel)
                .await
                .map_err(to_py_err)
        })
    }

    /// Search for similar vectors.
    ///
    ///Args:
    ///    vector: Query vector.
    ///    top_k: Number of results to return.
    ///    filter: Optional metadata filter.
    #[pyo3(signature = (vector, top_k=10, filter=None, include_metadata=true, include_vector=false))]
    fn search<'p>(
        &self,
        py: Python<'p>,
        vector: Vec<f32>,
        top_k: usize,
        filter: Option<Bound<'p, PyAny>>,
        include_metadata: bool,
        include_vector: bool,
    ) -> PyResult<Bound<'p, PyAny>> {
        let mut builder = self
            .inner
            .search(vector)
            .limit(top_k)
            .include_metadata(include_metadata)
            .include_vector(include_vector);

        if let Some(f) = filter {
            let json_val = py_to_json(py, &f.unbind())?;
            let rust_filter: Filter = serde_json::from_value(json_val).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid filter: {}", e))
            })?;
            builder = builder.filter(rust_filter);
        }

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = builder.execute().await.map_err(to_py_err)?;

            Python::attach(|py| convert_search_response(py, res))
        })
    }

    fn build_search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder {
            inner: Some(self.inner.search(vector)),
        }
    }

    fn delete<'p>(&self, py: Python<'p>, ids: Vec<String>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.delete(ids).await.map_err(to_py_err)
        })
    }

    fn delete_collection<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.delete_collection().await.map_err(to_py_err)
        })
    }

    /// Create the collection in the database.
    fn create<'p>(
        &self,
        py: Python<'p>,
        dimension: usize,
        distance: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let name_str = inner.name().to_string();

        let schema = bridge_embex::types::CollectionSchema {
            name: name_str,
            dimension,
            metric: match distance.as_str() {
                "cosine" => bridge_embex::types::DistanceMetric::Cosine,
                "euclidean" => bridge_embex::types::DistanceMetric::Euclidean,
                "dot" => bridge_embex::types::DistanceMetric::Dot,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Invalid distance metric",
                    ));
                }
            },
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.create(schema).await.map_err(to_py_err)
        })
    }

    /// Create the collection with optional dimension.
    ///
    /// For providers like Chroma that infer dimension from the first insert,
    /// you can pass `None` for dimension. For other providers, dimension is required.
    ///
    /// Args:
    ///     dimension: Optional dimension. Use `None` for Chroma (infers from first insert).
    ///     distance: Distance metric ("cosine", "euclidean", or "dot").
    #[pyo3(signature = (dimension=None, distance=None))]
    fn create_auto<'p>(
        &self,
        py: Python<'p>,
        dimension: Option<usize>,
        distance: Option<String>,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();

        let metric = match distance.as_deref().unwrap_or("cosine") {
            "cosine" => bridge_embex::types::DistanceMetric::Cosine,
            "euclidean" => bridge_embex::types::DistanceMetric::Euclidean,
            "dot" => bridge_embex::types::DistanceMetric::Dot,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Invalid distance metric",
                ));
            }
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.create_auto(dimension, metric).await.map_err(to_py_err)
        })
    }

    /// Insert points from a streaming iterator.
    fn insert_stream<'p>(
        &self,
        py: Python<'p>,
        points: Bound<'p, PyAny>,
        batch_size: usize,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let iterator = points.try_iter()?;
        let py_iter: Py<PyIterator> = iterator.unbind();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                let mut batch: Vec<Point> = Vec::with_capacity(batch_size);
                let mut done = false;

                Python::attach(|py| {
                    let mut iter = py_iter.bind(py).clone();

                    for _ in 0..batch_size {
                        let next_item = iter.next();
                        match next_item {
                            Some(Ok(item)) => {
                                if let Ok(p) = item.extract::<PyRef<EmbexPoint>>() {
                                    let mut metadata = None;
                                    if let Some(py_meta) = &p.metadata {
                                        let mut meta_map = HashMap::new();
                                        for (k, v) in py_meta {
                                            if let Ok(val) = py_to_json(py, v) {
                                                meta_map.insert(k.clone(), val);
                                            }
                                        }
                                        metadata = Some(meta_map);
                                    }
                                    batch.push(Point {
                                        id: p.id.clone(),
                                        vector: p.vector.clone(),
                                        metadata,
                                    });
                                }
                            }
                            _ => {
                                done = true;
                                break;
                            }
                        }
                    }
                });

                if !batch.is_empty() {
                    inner.insert(batch).await.map_err(to_py_err)?;
                }

                if done {
                    break;
                }
            }
            Ok(())
        })
    }
}

// Convert PyObject to serde_json::Value
fn py_to_json<'py>(py: Python<'py>, obj: &Py<PyAny>) -> PyResult<Value> {
    let bound = obj.bind(py);
    if bound.is_none() {
        return Ok(Value::Null);
    }

    if let Ok(s) = bound.extract::<String>() {
        return Ok(Value::String(s));
    }

    if let Ok(b) = bound.extract::<bool>() {
        return Ok(Value::Bool(b));
    }

    if let Ok(i) = bound.extract::<i64>() {
        return Ok(Value::Number(serde_json::Number::from(i)));
    }

    if let Ok(f) = bound.extract::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return Ok(Value::Number(n));
        }
    }

    if let Ok(dict) = bound.cast::<PyDict>() {
        let mut map = serde_json::Map::new();
        for (k, v) in dict {
            let k_str = k.extract::<String>()?;
            map.insert(k_str, py_to_json(py, &v.unbind())?);
        }
        return Ok(Value::Object(map));
    }

    if let Ok(list) = bound.cast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list {
            arr.push(py_to_json(py, &item.unbind())?);
        }
        return Ok(Value::Array(arr));
    }

    Ok(Value::String(bound.to_string()))
}

fn json_to_py(py: Python, v: Value) -> Py<PyAny> {
    match v {
        Value::Null => py.None(),
        Value::Bool(b) => b.into_py_any(py).unwrap(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py_any(py).unwrap()
            } else if let Some(f) = n.as_f64() {
                f.into_py_any(py).unwrap()
            } else {
                n.to_string().into_py_any(py).unwrap()
            }
        }
        Value::String(s) => s.into_py_any(py).unwrap(),
        Value::Array(a) => {
            let list = PyList::new(py, a.into_iter().map(|i| json_to_py(py, i))).unwrap();
            list.into()
        }
        Value::Object(o) => {
            let dict = PyDict::new(py);
            for (k, v) in o {
                dict.set_item(k, json_to_py(py, v)).ok();
            }
            dict.into()
        }
    }
}

fn convert_search_response(py: Python, res: RustSearchResponse) -> PyResult<SearchResponse> {
    let mut py_results = Vec::with_capacity(res.results.len());
    for r in res.results {
        let py_metadata = r.metadata.map(|m| {
            let mut map = HashMap::new();
            for (k, v) in m {
                map.insert(k, json_to_py(py, v));
            }
            map
        });

        py_results.push(SearchResult {
            id: r.id,
            score: r.score,
            vector: r.vector,
            metadata: py_metadata,
        });
    }

    let mut py_aggregations = HashMap::new();
    for (k, v) in res.aggregations {
        py_aggregations.insert(k, json_to_py(py, v));
    }

    Ok(SearchResponse {
        results: py_results,
        aggregations: py_aggregations,
    })
}

#[pyfunction]
fn cli_main<'p>(py: Python<'p>, args: Vec<String>) -> PyResult<Bound<'p, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        embex_cli::run(args).await.map_err(|e| EmbexError::new_err(e.to_string()))
    })
}

#[pymodule]
fn embex(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EmbexClient>()?;
    m.add_class::<Collection>()?;
    m.add_class::<SearchResult>()?;
    m.add_class::<SearchResponse>()?;
    m.add_class::<SearchBuilder>()?;
    m.add_class::<EmbexPoint>()?;
    m.add_function(wrap_pyfunction!(cli_main, m)?)?;
    m.add("EmbexError", py.get_type::<EmbexError>())?;
    m.add("ConfigError", py.get_type::<EmbexConfigError>())?;
    m.add("DatabaseError", py.get_type::<EmbexDatabaseError>())?;
    m.add(
        "SerializationError",
        py.get_type::<EmbexSerializationError>(),
    )?;
    m.add("ValidationError", py.get_type::<EmbexValidationError>())?;
    Ok(())
}
