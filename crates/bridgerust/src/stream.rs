#[cfg(feature = "python")]
pub mod python {
    #![allow(deprecated)]
    use futures::Stream;
    use pyo3::prelude::*;
    use pyo3_async_runtimes::tokio::into_future;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    // We need a proper implementation that handles the generic T conversion.
    // Since PyFuture returns Py<PyAny>, we need to extract T from it.

    type StreamFuture<T> = Pin<Box<dyn Future<Output = PyResult<Option<T>>> + Send>>;

    pub struct PyStreamAdapter<T> {
        iterator: Py<PyAny>,
        active_future: Option<StreamFuture<T>>,
    }

    impl<T> PyStreamAdapter<T>
    where
        T: for<'a> FromPyObject<'a, 'a> + Send + 'static,
    {
        pub fn new(iterator: Py<PyAny>) -> Self {
            Self {
                iterator,
                active_future: None,
            }
        }
    }

    impl<T> Stream for PyStreamAdapter<T>
    where
        T: for<'a> FromPyObject<'a, 'a> + Send + 'static,
    {
        type Item = PyResult<T>;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            loop {
                if let Some(fut) = &mut self.active_future {
                    match fut.as_mut().poll(cx) {
                        Poll::Ready(result) => {
                            self.active_future = None;
                            match result {
                                Ok(Some(item)) => return Poll::Ready(Some(Ok(item))),
                                Ok(None) => return Poll::Ready(None), // StopAsyncIteration
                                Err(e) => return Poll::Ready(Some(Err(e))),
                            }
                        }
                        Poll::Pending => return Poll::Pending,
                    }
                }

                // No active future, call __anext__
                // We need GIL to call Python
                let fut_res = Python::with_gil(|py| {
                    let iter_bound = self.iterator.bind(py);
                    match iter_bound.call_method0("__anext__") {
                        Ok(awaitable) => {
                            // Convert Python awaitable to Rust Future
                            let fut = into_future(awaitable)?;
                            Ok(Some(fut))
                        }
                        Err(e) => {
                            if e.is_instance_of::<pyo3::exceptions::PyStopAsyncIteration>(py) {
                                Ok(None)
                            } else {
                                Err(e)
                            }
                        }
                    }
                });

                match fut_res {
                    Ok(Some(rust_fut)) => {
                        // We have a generic future returning Py<PyAny>. We need to map it to T.
                        let mapped_fut = async move {
                            let result = rust_fut.await;
                            match result {
                                Ok(py_obj) => Python::with_gil(|py| {
                                    let val = py_obj.extract::<T>(py).map_err(|e| e.into())?;
                                    Ok(Some(val))
                                }),
                                Err(e) => {
                                    Python::with_gil(|py| {
                                        if e.is_instance_of::<pyo3::exceptions::PyStopAsyncIteration>(py) {
                                            Ok(None)
                                        } else {
                                            Err(e)
                                        }
                                    })
                                }
                            }
                        };

                        self.active_future = Some(Box::pin(mapped_fut));
                        // Loop continues to poll the new future
                    }
                    Ok(None) => return Poll::Ready(None), // StopAsyncIteration immediately
                    Err(e) => return Poll::Ready(Some(Err(e))),
                }
            }
        }
    }
}
