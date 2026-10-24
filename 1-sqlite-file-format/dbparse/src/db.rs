use std::path::Path;

use crate::{Result, corrupt, header::Header, pager::Pager};

/// An open database file: the pager plus the parsed header.
pub struct Db {
    pager: Pager,
    header: Header,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> Result<Db> {
        let pager = Pager::open(path)?;
        let header = Header::parse(pager.first_bytes())?;
        Ok(Db { pager, header })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn usable_size(&self) -> u32 {
        self.header.usable_size()
    }

    pub fn page_count(&self) -> u32 {
        if self.header.page_count_is_valid() {
            self.header.page_count
        } else {
            self.pager.file_pages()
        }
    }

    pub fn page(&self, n: u32) -> Result<Vec<u8>> {
        if n > self.page_count() {
            return corrupt(format!("page {n} is past the end of the database"));
        }
        self.pager.page(n)
    }
}
