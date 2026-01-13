use std::fmt::Display;

#[derive(Debug)]
pub struct BridgeError(pub String);

impl Display for BridgeError {
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

impl From<&str> for BridgeError {
    fn from(s: &str) -> Self {
        BridgeError(s.to_string())
    }
}

impl From<anyhow::Error> for BridgeError {
    fn from(e: anyhow::Error) -> Self {
        BridgeError(e.to_string())
    }
}

#[cfg(feature = "python")]
impl From<BridgeError> for crate::pyo3::PyErr {
    fn from(e: BridgeError) -> Self {
        crate::pyo3::exceptions::PyRuntimeError::new_err(e.0)
    }
}

#[cfg(feature = "nodejs")]
impl From<BridgeError> for crate::napi::Error {
    fn from(e: BridgeError) -> Self {
        crate::napi::Error::from_reason(e.0)
    }
}

impl BridgeError {
    #[cfg(all(feature = "python", not(feature = "nodejs")))]
    pub fn into_platform(self) -> crate::pyo3::PyErr {
        self.into()
    }

    #[cfg(all(feature = "nodejs", not(feature = "python")))]
    pub fn into_platform(self) -> crate::napi::Error {
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
