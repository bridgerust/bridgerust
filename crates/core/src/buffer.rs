//! Zero-copy buffer utilities

use std::ops::{Deref, DerefMut};

/// Owned byte buffer with cheap borrowed access.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    /// Creates an empty buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty buffer with the requested capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Wraps an existing allocation without copying.
    #[must_use]
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns the current length of the buffer in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` when the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Appends one byte.
    pub fn push(&mut self, byte: u8) {
        self.data.push(byte);
    }

    /// Appends all bytes from `slice`.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.data.extend_from_slice(slice);
    }

    /// Clears all bytes from the buffer.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns a shared byte slice view.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Returns a mutable byte slice view.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.data.as_mut_slice()
    }

    /// Returns the owned storage without copying.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(value: Vec<u8>) -> Self {
        Self::from_vec(value)
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl Deref for Buffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn creates_empty_buffer() {
        let buffer = Buffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn from_vec_round_trips_without_copying() {
        let bytes = vec![1, 2, 3, 4];
        let buffer = Buffer::from_vec(bytes.clone());
        assert_eq!(buffer.as_slice(), bytes.as_slice());
        assert_eq!(buffer.into_inner(), bytes);
    }

    #[test]
    fn mutating_slice_updates_buffer() {
        let mut buffer = Buffer::from_vec(vec![1, 2, 3]);
        buffer.as_mut_slice()[1] = 9;
        assert_eq!(buffer.as_slice(), &[1, 9, 3]);
    }

    #[test]
    fn append_and_clear_behave_as_expected() {
        let mut buffer = Buffer::with_capacity(8);
        buffer.push(1);
        buffer.extend_from_slice(&[2, 3, 4]);
        assert_eq!(buffer.as_slice(), &[1, 2, 3, 4]);
        buffer.clear();
        assert!(buffer.is_empty());
    }
}
