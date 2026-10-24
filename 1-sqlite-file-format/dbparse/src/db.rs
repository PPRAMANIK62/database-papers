//! `Db`: an open database file. Every read the crate offers is a method
//! on it.
//!
//! `Db` owns the pager and the parsed header. Each later module adds the
//! reads for its part of the spec in its own `impl Db` block: `page.rs`
//! adds `btree_page`, `schema.rs` adds `schema`, and so on. This file only
//! opens the file and hands out pages.

use std::path::Path;

use crate::{Result, corrupt, header::Header, pager::Pager};

/// An open database file: the pager plus the parsed header.
pub struct Db {
    /// Reads pages from the file.
    pager: Pager,
    /// The header, parsed once on open.
    header: Header,
}

impl Db {
    /// Opens a database file and parses its header.
    pub fn open(path: impl AsRef<Path>) -> Result<Db> {
        let pager = Pager::open(path)?;
        let header = Header::parse(pager.first_bytes())?;
        Ok(Db { pager, header })
    }

    /// The parsed database header.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The usable bytes of each page: the page size minus the reserved
    /// bytes. Every b-tree formula uses this, not the page size.
    pub fn usable_size(&self) -> u32 {
        self.header.usable_size()
    }

    /// The database size in pages: the header's count if it's valid,
    /// otherwise the file's length.
    pub fn page_count(&self) -> u32 {
        if self.header.page_count_is_valid() {
            self.header.page_count
        } else {
            self.pager.file_pages()
        }
    }

    /// Reads the page `n` as raw bytes, refusing pages past the database's
    /// end even if the file is longer.
    pub fn page(&self, n: u32) -> Result<Vec<u8>> {
        if n > self.page_count() {
            return corrupt(format!("page {n} is past the end of the database"));
        }
        self.pager.page(n)
    }
}
