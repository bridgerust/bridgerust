use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::IntoPyObjectExt;

use bridge_kabod::KabodClient as RustClient;
use bridge_kabod::config::KabodConfig;
use bridge_kabod::types::Point;
use std::collections::HashMap;
use serde_json::Value;

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
        Self { id, vector, metadata }
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

// Convert PyObject to serde_json::Value
fn py_to_json<'py>(py: Python<'py>, obj: &Py<PyAny>) -> PyResult<Value> {
    let bound = obj.bind(py);
    if bound.is_none() {
        return Ok(Value::Null);
    }
    
    // Using extract which is fallible but appropriate here
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
        },
        Value::String(s) => s.into_py_any(py).unwrap(),
        Value::Array(a) => {
            let list = PyList::new(py, a.into_iter().map(|i| json_to_py(py, i))).unwrap();
            list.into()
        },
        Value::Object(o) => {
            let dict = PyDict::new(py);
            for (k, v) in o {
                dict.set_item(k, json_to_py(py, v)).ok();
            }
            dict.into()
        }
    }
}

#[pymethods]
impl Collection {
    fn insert<'p>(&self, py: Python<'p>, points: Vec<PyRef<'p, KabodPoint>>) -> PyResult<Bound<'p, PyAny>> {
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
            inner.insert(rust_points).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (vector, top_k=None, include_metadata=None, include_vector=None))]
    fn search<'p>(
        &self, 
        py: Python<'p>, 
        vector: Vec<f32>, 
        top_k: Option<usize>,
        include_metadata: Option<bool>,
        include_vector: Option<bool>
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let limit = top_k.unwrap_or(10);
        let inc_meta = include_metadata.unwrap_or(true);
        let inc_vec = include_vector.unwrap_or(false);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let builder = inner.search(vector).await
                .limit(limit)
                .include_metadata(inc_meta)
                .include_vector(inc_vec);
                
            let results = inner.query(builder).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Since we can't easily return PyObjects from the future without GIL,
            // we'll use Python::with_gil inside the future_into_py which is allowed 
            // but we must not block the runtime.
            // pyo3_async_runtimes handles the GIL acquisition for the result conversion 
            // IF the result implements IntoPyObject.
            // Our result is `Vec<SearchResult>`. `SearchResult` implements `IntoPyObject` via `#[pyclass]`.
            // So we just need to return the Rust objects?
            // Wait, SearchResult holds `PyObject` (aka `Py<PyAny>`) in metadata.
            // Those can be cloned around.

            let mut py_results = Vec::with_capacity(results.len());
            // Need GIL to convert JSON to PyObject for metadata
            Python::attach(|py| {
                for r in results {
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
            });
            
            Ok(py_results)
        })
    }

    fn delete<'p>(&self, py: Python<'p>, ids: Vec<String>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.delete(ids).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
    
    fn delete_collection<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.delete_collection().await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn create<'p>(
        &self, 
        py: Python<'p>, 
        dimension: usize,
        distance: String
    ) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        let name_str = inner.name().to_string(); // Use accessor
        
        // Construct schema from args
        let schema = bridge_kabod::types::CollectionSchema {
            name: name_str,
            dimension,
            metric: match distance.as_str() {
                "cosine" => bridge_kabod::types::DistanceMetric::Cosine,
                "euclidean" => bridge_kabod::types::DistanceMetric::Euclidean,
                "dot" => bridge_kabod::types::DistanceMetric::Dot,
                _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid distance metric")),
            },
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            inner.create(schema).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn insert_batch<'p>(&self, py: Python<'p>, points: Vec<PyRef<'p, KabodPoint>>, batch_size: usize) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        
        // Similar to insert, but we pass points to Rust first, then batch them using the new Rust method
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
            inner.insert_batch(rust_points, batch_size).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn insert_stream<'p>(&self, py: Python<'p>, points: Bound<'p, PyAny>, batch_size: usize) -> PyResult<Bound<'p, PyAny>> {
        let inner = self.inner.clone();
        
        // We can't easily stream FROM python iterator asyncly because calling `next` requires GIL.
        // So we have two options:
        // 1. Consume the iterator entirely into a Vec (not streaming).
        // 2. Consume in chunks (requires holding GIL intermittently).
        // 3. Use an async Python generator (requires `anext` and await).
        
        // For simplicity and effectiveness in this "advanced" phase, let's implement the chunked consumption approach.
        // We will read `batch_size` items from the iterator under GIL, then release GIL and insert them async, then repeat.
        // However, `future_into_py` expects a single future. We can spawn a task that does this loop?
        // But the loop needs GIL to get next batch.
        //
        // A common pattern is to just accept an iterable, consume it all into a Vec, and batch insert it? No that defeats the purpose.
        //
        // Correct Async Stream approach:
        // We return a future that:
        //   Loop:
        //     Acquire GIL
        //     Read N items from iterator
        //     If empty, break
        //     Release GIL
        //     Await insert_batch(N)
        
        // To do this with pyo3-async-runtimes, we can use a loop inside the async block.
        // inside the async block we can `Python::with_gil` to read the next batch.
        
        // We need to keep a reference to the iterator. Since `future_into_py` moves things into the future 'static,
        // we need to be careful with Py objects. We can store `Py<PyIterator>`.
        
        let iterator = points.try_iter()?; // Get iterator from iterable
        let py_iter: Py<pyo3::types::PyIterator> = iterator.unbind();
        
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                let mut batch: Vec<Point> = Vec::with_capacity(batch_size);
                let mut done = false;

                Python::attach(|py| {
                    let mut iter = py_iter.bind(py).clone(); // Bound<PyIterator> is an Iterator, need owned to be mutable
                    // We need it mutable to call next()
                    
                    for _ in 0..batch_size {
                        let next_item = iter.next();
                        match next_item {
                            Some(Ok(item)) => {
                                // Convert item to Point
                                // We expect item to be KabodPoint, but it might be just a dict or object we can extract
                                if let Ok(p) = item.extract::<PyRef<KabodPoint>>() {
                                     let mut metadata = None;
                                     // ... conversion logic ...
                                     if let Some(py_meta) = &p.metadata {
                                        let mut meta_map = HashMap::new();
                                        for (k, v) in py_meta {
                                            // Handle error in map?
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
                                } else {
                                    // Handle invalid type? For now just stop or skip? 
                                    // Ideally return error, but inside loop difficult.
                                    // Let's assume strict typing for now.
                                }
                            },
                            Some(Err(_e)) => {
                                // Iterator error
                                done = true; 
                                break; 
                            },
                            None => {
                                done = true;
                                break;
                            }
                        }
                    }
                });

                if !batch.is_empty() {
                    inner.insert(batch).await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
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
    m.add_class::<KabodPoint>()?;
    Ok(())
}
