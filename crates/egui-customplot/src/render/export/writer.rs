use std::io;

use super::ExportError;

/// Both encoders stop before the encoded-output limit, not after a huge String.
pub(super) struct LimitedWriter {
    pub bytes: Vec<u8>,
    limit: usize,
}

impl LimitedWriter {
    pub fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }
    pub fn append(&mut self, bytes: &[u8]) -> Result<(), ExportError> {
        if bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(ExportError::LimitExceeded("encoded output bytes"));
        }
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| ExportError::AllocationFailed)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

impl io::Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.append(bytes).map_err(io::Error::other)?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Write for LimitedWriter {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.append(text.as_bytes()).map_err(|_| std::fmt::Error)
    }
}

pub(super) fn base64(bytes: &[u8], output: &mut LimitedWriter) -> Result<(), ExportError> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in bytes.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        output.append(&[
            ALPHABET[(n >> 18) as usize],
            ALPHABET[((n >> 12) & 63) as usize],
            if chunk.len() > 1 {
                ALPHABET[((n >> 6) & 63) as usize]
            } else {
                b'='
            },
            if chunk.len() > 2 {
                ALPHABET[(n & 63) as usize]
            } else {
                b'='
            },
        ])?;
    }
    Ok(())
}
