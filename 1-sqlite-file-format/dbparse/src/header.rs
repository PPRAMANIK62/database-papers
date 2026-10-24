//! The 100-byte database header at the start of page 1 (spec section 1.3).
//!
//! Every field sits at a fixed offset and is big-endian. `Header::parse`
//! reads them all and rejects a file that breaks one of the header's
//! rules: the magic string, the read version, the payload fractions,
//! the text encoding and the smallest usable page size.

use crate::{
    Result,
    bytes::{be16, be32},
    corrupt,
    pager::decode_page_size,
};

/// The first 16 bytes of every database file: the text "SQLite format 3"
/// and a zero byte.
pub const MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// How text values are stored, from offset 56. One encoding covers the
/// whole file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    /// 1. UTF-8.
    UTF8,
    /// UTF-16, little-endian.
    UTF16LE,
    /// UTF-16, big-endian.
    UTF16BE,
}

/// The 100-byte database header (spec section 1.3). Each field's doc starts
/// with its byte offset.
#[derive(Debug, Clone)]
pub struct Header {
    /// 16: bytes per page, a power of two from 512 to 65536.
    pub page_size: u32,
    /// 18. 1 for a rollback journal: 2 for WAL (Write Ahead Log)
    pub write_version: u8,
    /// 19. like the write version. Above 2, we may not read the file.
    pub read_version: u8,
    /// 20. bytes at the end of each page kept for extensions, usually 0.
    pub reserved: u8,
    /// 24. goes up by one every time a transaction changes the file.
    pub change_counter: u32,
    /// 28. the database size in pages, if `page_count_is_valid` says so.
    pub page_count: u32,
    /// 32. the first freelist trunk page, or 0 if the freelist is empty.
    pub first_freelist_trunk: u32,
    /// 36. how many pages the freelist holds, trunks and leaves together.
    pub freelist_count: u32,
    /// 40. goes up by one every time the schema changes.
    pub schema_cookie: u32,
    /// 44. which schema format (1 to 4) the file uses.
    pub schema_format: u32,
    /// 48. the suggested page cache size. SQLite's hint, not ours.
    pub default_cache_size: i32,
    /// 52. the largest root page, in auto-vaccum files, 0 otheriwse.
    pub largest_root_page: u32,
    /// 56. how text values are encoded.
    pub text_encoding: TextEncoding,
    /// 60. a number the application sets with `PRAGMA user_version`.
    pub user_version: i32,
    /// 64. true for incremental vaccum, in an auto-vaccum file.
    pub incremental_vaccum: bool,
    /// 68. a number the application sets with `PRAGMA application_id`.
    pub application_id: i32,
    /// 92. the change counter's value when `page_count` was last written.
    pub version_valid_for: u32,
    /// 96. the SQLite verison that last wrote the file, as X * 1000000 +
    /// Y * 1000 + Z.
    pub sqlite_version: u32, // 96
}

impl Header {
    /// Reads the 100 header bytes, or says which rule they break.
    pub fn parse(b: &[u8; 100]) -> Result<Header> {
        if &b[0..16] != MAGIC {
            return corrupt("not an SQLite database: the magic string is wrong");
        }
        // Spec section 1.3.3: a read version above 2 means a newer format.
        if b[19] > 2 {
            return corrupt(format!("read version {} needs a newer SQLite", b[19]));
        }
        // Spec section 1.3.5: these three bytes are fixed at 64, 32, 32.
        if (b[21], b[22], b[23]) != (64, 32, 32) {
            return corrupt("the payload fractions must be 64, 32 and 32");
        }
        let text_encoding = match be32(b, 56)? {
            1 => TextEncoding::UTF8,
            2 => TextEncoding::UTF16LE,
            3 => TextEncoding::UTF16BE,
            n => return corrupt(format!("text encoding {n} is not 1, 2 or 3")),
        };
        // `b` is exactly 100 bytes, so the single-byte reads can't fail.
        let header = Header {
            page_size: decode_page_size(be16(b, 16)?)?,
            write_version: b[18],
            read_version: b[19],
            reserved: b[20],
            change_counter: be32(b, 24)?,
            page_count: be32(b, 28)?,
            first_freelist_trunk: be32(b, 32)?,
            freelist_count: be32(b, 36)?,
            schema_cookie: be32(b, 40)?,
            schema_format: be32(b, 44)?,
            default_cache_size: be32(b, 48)? as i32,
            largest_root_page: be32(b, 52)?,
            text_encoding,
            user_version: be32(b, 60)? as i32,
            incremental_vaccum: be32(b, 64)? != 0,
            application_id: be32(b, 68)? as i32,
            version_valid_for: be32(b, 92)?,
            sqlite_version: be32(b, 96)?,
        };
        // Spec section: 1.3.4: the payload formulas need at least 480 bytes.
        if header.usable_size() < 480 {
            return corrupt("the usable is page size is below 480 bytes");
        }
        Ok(header)
    }

    /// The page size minus the reserved bytes at the end of each page.
    pub fn usable_size(&self) -> u32 {
        self.page_size - self.reserved as u32
    }

    /// The in-header page count is trusted only when the change counter
    /// matches the version-valid-for number (spec section 1.3.7).
    pub fn page_count_is_valid(&self) -> bool {
        self.page_count != 0 && self.change_counter == self.version_valid_for
    }
}
