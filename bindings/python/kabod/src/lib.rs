use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use bridge_kabod::KabodClient as RustClient;
use bridge_kabod::config::KabodConfig;
use bridge_kabod::types::Point;
use serde_json::Value;
use std::collections::HashMap;

#[pyclass]
struct KabodClient {
    inner: RustClient,
}

#[pymethods]
impl KabodClient {
    #[new]
    #[pyo3(signature = (provider, url, api_key=None))]
    fn new(provider: String, url: String, api_key: Option<String>) -> PyResult<Self> {
        let config = KabodConfig {
            provider,
            url,
            api_key,
            timeout_ms: None,
            options: Default::default(),
        };

        let client = RustClient::new(config)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self { inner: client })
    }

    fn collection(&self, name: String) -> Collection {
        Collection {
            inner: self.inner.collection(&name),
        }
    }
}

#[pyclass]
struct Collection {
    inner: bridge_kabod::client::Collection,
}

#[pyclass(name = "Point")]
struct KabodPoint {
    #[pyo3(get, set)]
    id: String,
    #[pyo3(get, set)]
    vector: Vec<f32>,
    #[pyo3(get, set)]
    metadata: Option<HashMap<String, Py<PyAny>>>,
}

#[pymethods]
impl KabodPoint {
    #[new]
    fn new(id: String, vector: Vec<f32>, metadata: Option<HashMap<String, Py<PyAny>>>) -> Self {
        Self {
            id,
            vector,
            metadata,
        }
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

    Ok(Value::String(bound.to_string()))
}

// Helper to convert JSON Value to PyObject
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

#[pyclass]
struct SearchBuilder {
    inner: Option<bridge_kabod::client::SearchBuilder>,
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
        let inner = self
            .inner
            .take()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Search already executed"))?;

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = inner
                .execute()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Python::attach(|py| {
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
            })
        })
    }
}

#[pymethods]
impl Collection {
    fn insert<'p>(
        &self,
        py: Python<'p>,
        points: Vec<PyRef<'p, KabodPoint>>,
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
            inner
                .insert(rust_points)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn search(&self, vector: Vec<f32>) -> SearchBuilder {
        SearchBuilder {
            inner: Some(self.inner.search(vector)),
        }
    }

    fn delete<'p>(&self, py: Python<'p>, ids: Vec<String>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .delete(ids)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn delete_collection<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .delete_collection()
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn create<'p>(
        &self,
        py: Python<'p>,
        dimension: usize,
        distance: String,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let name_str = inner.name().to_string();

        let schema = bridge_kabod::types::CollectionSchema {
            name: name_str,
            dimension,
            metric: match distance.as_str() {
                "cosine" => bridge_kabod::types::DistanceMetric::Cosine,
                "euclidean" => bridge_kabod::types::DistanceMetric::Euclidean,
                "dot" => bridge_kabod::types::DistanceMetric::Dot,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Invalid distance metric",
                    ));
                }
            },
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner
                .create(schema)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn insert_batch<'p>(
        &self,
        py: Python<'p>,
        points: Vec<PyRef<'p, KabodPoint>>,
        batch_size: usize,
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
            inner
                .insert_batch(rust_points, batch_size)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn insert_stream<'p>(
        &self,
        py: Python<'p>,
        points: Bound<'p, PyAny>,
        batch_size: usize,
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let iterator = points.try_iter()?;
        let py_iter: Py<pyo3::types::PyIterator> = iterator.unbind();

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
                                if let Ok(p) = item.extract::<PyRef<KabodPoint>>() {
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
                            Some(Err(_e)) => {
                                done = true;
                                break;
                            }
                            None => {
                                done = true;
                                break;
                            }
                        }
                    }
                });

                if !batch.is_empty() {
                    inner.insert(batch).await.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;
                }

                if done {
                    break;
                }
            }
            Ok(())
        })
    }
}

#[pymodule]
fn kabod(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<KabodClient>()?;
    m.add_class::<Collection>()?;
    m.add_class::<SearchResult>()?;
    m.add_class::<SearchResponse>()?;
    m.add_class::<SearchBuilder>()?;
    m.add_class::<KabodPoint>()?;
    Ok(())
}
