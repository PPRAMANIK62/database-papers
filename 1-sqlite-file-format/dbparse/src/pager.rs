use std::{fs::File, os::unix::fs::FileExt, path::Path};

use crate::{Result, bytes::be16, corrupt};

/// Turns the 2-byte page size field into a size in bytes.
/// 65536 doesn't fit in 2 bytes, so the file stores it as 1.
pub fn decode_page_size(raw: u16) -> Result<u32> {
    let size = if raw == 1 { 65536 } else { raw as u32 };
    if size < 512 || !size.is_power_of_two() {
        return corrupt(format!(
            "page size {raw} is not a power of two from 512 to 65536"
        ));
    }
    Ok(size)
}

/// Reads whole pages from a database file.
pub struct Pager {
    file: File,
    first_bytes: [u8; 100],
    page_size: u32,
    file_pages: u32,
}

impl Pager {
    pub fn open(path: impl AsRef<Path>) -> Result<Pager> {
        let file = File::open(path)?;
        // The one read that isn't a whole page: the 100-byte header.
        let mut first_bytes = [0u8; 100];
        file.read_exact_at(&mut first_bytes, 0)?;
        let page_size = decode_page_size(be16(&first_bytes, 16)?)?;
        let file_pages = (file.metadata()?.len() / page_size as u64) as u32;
        Ok(Pager {
            file,
            first_bytes,
            page_size,
            file_pages,
        })
    }

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
