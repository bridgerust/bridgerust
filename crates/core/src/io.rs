//! Streaming I/O utilities

use std::io::{self, Cursor, Read};

/// Thin wrapper around any `Read` source with convenient chunked APIs.
pub struct StreamReader<R = Cursor<Vec<u8>>> {
    reader: R,
}

impl StreamReader<Cursor<Vec<u8>>> {
    /// Creates an empty in-memory stream.
    #[must_use]
    pub fn new() -> Self {
        Self::from_bytes(Vec::new())
    }

    /// Creates an in-memory stream from bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            reader: Cursor::new(bytes.into()),
        }
    }
}

impl<R: Read> StreamReader<R> {
    /// Wraps any reader.
    #[must_use]
    pub fn from_reader(reader: R) -> Self {
        Self { reader }
    }

    /// Reads up to `size` bytes and returns what was read.
    pub fn read_chunk(&mut self, size: usize) -> io::Result<Vec<u8>> {
        if size == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = vec![0_u8; size];
        let read = self.reader.read(&mut buffer)?;
        buffer.truncate(read);
        Ok(buffer)
    }

    /// Reads the remaining bytes to the end of the stream.
    pub fn read_to_end(&mut self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        self.reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    /// Returns the wrapped reader.
    #[must_use]
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl Default for StreamReader<Cursor<Vec<u8>>> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::StreamReader;
    use std::io::Cursor;

    #[test]
    fn empty_stream_returns_no_bytes() {
        let mut reader = StreamReader::new();
        assert!(reader.read_chunk(4).unwrap().is_empty());
        assert!(reader.read_to_end().unwrap().is_empty());
    }

    #[test]
    fn reads_in_chunks() {
        let mut reader = StreamReader::from_bytes(b"bridgerust".to_vec());
        assert_eq!(reader.read_chunk(6).unwrap(), b"bridge");
        assert_eq!(reader.read_chunk(6).unwrap(), b"rust");
        assert!(reader.read_chunk(1).unwrap().is_empty());
    }

    #[test]
    fn read_to_end_returns_remaining_data() {
        let mut reader = StreamReader::from_bytes(b"abcdef");
        assert_eq!(reader.read_chunk(2).unwrap(), b"ab");
        assert_eq!(reader.read_to_end().unwrap(), b"cdef");
    }

    #[test]
    fn zero_sized_chunk_does_not_consume_input() {
        let mut reader = StreamReader::from_bytes(b"ok");
        assert!(reader.read_chunk(0).unwrap().is_empty());
        assert_eq!(reader.read_to_end().unwrap(), b"ok");
    }

    #[test]
    fn supports_any_read_source() {
        let cursor = Cursor::new(b"xyz".to_vec());
        let mut reader = StreamReader::from_reader(cursor);
        assert_eq!(reader.read_to_end().unwrap(), b"xyz");
    }
}
