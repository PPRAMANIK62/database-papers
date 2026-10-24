//! Reading fixed-size numbers out of a byte slice.
//!
//! Every multi-byte number in the file is big-endian: the first byte is the
//! most significant (spec section 1.3). Each reader checks its bounds and
//! returns an error instead of panicking, because a corrupt file can point
//! anywhere.

use crate::{Result, corrupt};

/// Reads the byte at `at`, or fails if it is past the end.
pub fn byte(b: &[u8], at: usize) -> Result<u8> {
    match b.get(at) {
        Some(&v) => Ok(v),
        None => corrupt(format!("1-byte read at offset {at} runs past the end")),
    }
}

/// Reads a big-endian `u16` at `at`, or fails if it runs past the end.
pub fn be16(b: &[u8], at: usize) -> Result<u16> {
    match b.get(at..at + 2) {
        Some(s) => Ok(u16::from_be_bytes([s[0], s[1]])),
        None => corrupt(format!("2-byte read at offset {at} runs past the end")),
    }
}

/// Reads a big-endian `u32` at `at`, or fails if it runs past the end.
pub fn be32(b: &[u8], at: usize) -> Result<u32> {
    match b.get(at..at + 4) {
        Some(s) => Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]])),
        None => corrupt(format!("4-byte read at offset {at} runs past the end")),
    }
}
