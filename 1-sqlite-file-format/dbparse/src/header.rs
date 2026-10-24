use crate::{
    Result,
    bytes::{be16, be32},
    corrupt,
    pager::decode_page_size,
};

pub const MAGIC: &[u8; 16] = b"SQLite format 3\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncoding {
    UTF8,
    UTF16LE,
    UTF16BE,
}

/// The 100-byte database header (spec section 1.3)
#[derive(Debug, Clone)]
pub struct Header {
    pub page_size: u32,              // offset 16
    pub write_version: u8,           // 18
    pub read_version: u8,            // 19
    pub reserved: u8,                // 20
    pub change_counter: u32,         // 24
    pub page_count: u32,             // 28
    pub first_freelist_trunk: u32,   // 32
    pub freelist_count: u32,         // 36
    pub schema_cookie: u32,          // 40
    pub schema_format: u32,          // 44
    pub default_cache_size: i32,     // 48
    pub largest_root_page: u32,      // 52
    pub text_encoding: TextEncoding, // 56
    pub user_version: i32,           // 60
    pub incremental_vaccum: bool,    // 64
    pub application_id: i32,         // 68
    pub version_valid_for: u32,      // 92
    pub sqlite_version: u32,         // 96
}

impl Header {
    pub fn parse(b: &[u8; 100]) -> Result<Header> {
        if &b[0..16] != MAGIC {
            return corrupt("not an SQLite database: the magic string is wrong");
        }
        if b[19] > 2 {
            return corrupt(format!("read version {} needs a newer SQLite", b[19]));
        }
        if (b[21], b[22], b[23]) != (64, 32, 32) {
            return corrupt("the payload fractions must be 64, 32 and 32");
        }
        let text_encoding = match be32(b, 56)? {
            1 => TextEncoding::UTF8,
            2 => TextEncoding::UTF16LE,
            3 => TextEncoding::UTF16BE,
            n => return corrupt(format!("text encoding {n} is not 1, 2 or 3")),
        };
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
