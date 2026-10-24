//! The pager: the only code that touches the file.
//!
//! The file is an array of equal-sized pages, numbered from 1 (spec
//! section 1.2). Page `n` starts at byte `(n - 1) * page_size`. The pager
//! reads the first 100 bytes once to learn the page size, then only ever
//! reads the whole page.

use std::{fs::File, os::unix::fs::FileExt, path::Path};

use crate::{Result, bytes::be16, corrupt};

/// Turns the 2-byte page size field into a size in bytes.
/// 65536 doesn't fit in 2 bytes, so the file stores it as 1.
pub fn decode_page_size(raw: u16) -> Result<u32> {
    let size = if raw == 1 { 65536 } else { raw as u32 };
    // Spec section 1.3.2: a power of two from 512 to 65536.
    if size < 512 || !size.is_power_of_two() {
        return corrupt(format!(
            "page size {raw} is not a power of two from 512 to 65536"
        ));
    }
    Ok(size)
}

/// Reads whole pages from a database file.
pub struct Pager {
    /// The open file. Reads go through `read_exact_at`, so there is no
    /// shared file position to keep track of.
    file: File,
    /// Bytes 0 to 99: the database header, parsed later by `Header`.
    first_bytes: [u8; 100],
    /// The page size in bytes, from offset 16 of the header.
    page_size: u32,
    /// How many whole pages the file's length holds.
    file_pages: u32,
}

impl Pager {
    /// Opens the file and reads its header to learn the page size.
    pub fn open(path: impl AsRef<Path>) -> Result<Pager> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        // SQLite opens an empty file as a new database with no pages yet.
        // There is nothing for us to read, so we say so.
        if len == 0 {
            return corrupt("the file is empty: a new database with no pages yet");
        }
        if len < 100 {
            return corrupt(format!(
                "the file is {len} bytes, too short for the 100-byte header"
            ));
        }
        // The one read that isn't a whole page: the 100-byte header.
        let mut first_bytes = [0u8; 100];
        file.read_exact_at(&mut first_bytes, 0)?;
        let page_size = decode_page_size(be16(&first_bytes, 16)?)?;
        // A partial page at the end of the file doesn't count.
        let file_pages = (len / page_size as u64) as u32;
        Ok(Pager {
            file,
            first_bytes,
            page_size,
            file_pages,
        })
    }

    /// The page size in bytes.
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// The number of whole pages in the file, calculates from its length.
    pub fn file_pages(&self) -> u32 {
        self.file_pages
    }

    /// The first 100 bytes of the file, as read by `open`.
    pub fn first_bytes(&self) -> &[u8; 100] {
        &self.first_bytes
    }

    /// Reads page `n`. Pages are numbered from 1.
    pub fn page(&self, n: u32) -> Result<Vec<u8>> {
        if n == 0 || n > self.file_pages {
            return corrupt(format!(
                "page {n} is outside the file (pages 1 to {})",
                self.file_pages
            ));
        }
        let mut buf = vec![0u8; self.page_size as usize];
        // Compute in u64: offsets past 4 GiB don't fit in a u32.
        let offset = (n as u64 - 1) * self.page_size as u64;
        self.file.read_exact_at(&mut buf, offset)?;
        Ok(buf)
    }
}

#[test]
fn page_sizes() {
    assert_eq!(decode_page_size(4096).unwrap(), 4096);
    assert_eq!(decode_page_size(1).unwrap(), 65536);
    assert!(decode_page_size(1000).is_err());
    assert!(decode_page_size(256).is_err());
}
