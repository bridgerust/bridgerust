pub use bridgerust_macros::{
    bridge, bridge_async, bridge_module, error, exception, export, new, pyo3_dummy, staticmethod,
    validate,
};

#[cfg(feature = "python")]
pub use pyo3;

#[cfg(feature = "python")]
pub use pyo3_async_runtimes;

pub mod stream;

pub mod collections;
pub mod convert;
pub mod error;
pub mod types;

#[cfg(feature = "nodejs")]
pub use napi;

#[cfg(feature = "nodejs")]
pub use napi_derive;

pub use error::BridgeError;
pub use types::JsonValue;

pub type Result<T> = std::result::Result<T, BridgeError>;
