//! One b-tree page: its header, its cell pointer array and its free space
//! (spec section 1.6).
//!
//! A b-tree page is a slotted page. The header and the cell pointer array
//! grow down from the top, the cells grow up from the bottom, and the
//! space between them is free:
//!
//! ```text
//! | header | cell pointers -> | unallocated | <- cells |
//! ```
//!
//! This module parses the top of the page and adds up the free space.

use crate::{
    Db, Result,
    bytes::{be16, be32, byte},
    corrupt,
};

/// The four kinds of b-tree page, from the flag byte at offset 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    /// 0x02: keys and child pointers of an index b-tree.
    IndexInterior,
    /// 0x05: rowids and child pointers of a table b-tree.
    TableInterior,
    /// 0x0a: the bottom of an index b-tree: keys only.
    IndexLeaf,
    /// 0x0d: the bottom of a table b-tree: rowids and rows.
    TableLeaf,
}

impl PageType {
    /// Reads the flag byte. Any other value means this isn't a b-tree page.
    pub fn from_flag(flag: u8) -> Result<PageType> {
        match flag {
            0x02 => Ok(PageType::IndexInterior),
            0x05 => Ok(PageType::TableInterior),
            0x0a => Ok(PageType::IndexLeaf),
            0x0d => Ok(PageType::TableLeaf),
            other => corrupt(format!("{other:#04x} is not a b-tree page type")),
        }
    }

    /// True for the two leaf types, which have no children.
    pub fn is_leaf(self) -> bool {
        matches!(self, PageType::IndexLeaf | PageType::TableLeaf)
    }

    /// True for the two table types, whose keys are rowids.
    pub fn is_table(self) -> bool {
        matches!(self, PageType::TableInterior | PageType::TableLeaf)
    }

    /// Leaf pages have an 8-byte header, while interior pages add a
    /// 4-byte right-most pointer.
    pub fn header_len(self) -> usize {
        if self.is_leaf() { 8 } else { 12 }
    }

    pub fn name(self) -> &'static str {
        match self {
            PageType::IndexInterior => "index interior",
            PageType::TableInterior => "table interior",
            PageType::IndexLeaf => "index leaf",
            PageType::TableLeaf => "table leaf",
        }
    }
}

/// A run of 4 or more free bytes inside the cell content area. Its first
/// 2 bytes hold the offset of the next freeblock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeBlock {
    /// Where the freeblock starts, from the start of the page.
    pub offset: u16,
    /// Its size in bytes, including its own 4-byte next offset.
    pub size: u16,
}

/// One parsed b-tree page (spec section 1.6)
#[derive(Debug, Clone)]
pub struct BTreePage {
    /// The page number, from 1.
    pub number: u32,
    /// The whole page, kept so cells can be read out of it later.
    pub data: Vec<u8>,
    /// The usable size: cells may not run past this offset.
    pub usable_size: u32,
    /// 100 on page 1, where the database header comes first, 0 elsewhere.
    pub header_start: usize,
    /// Header byte 0: what kind of page this is.
    pub page_type: PageType,
    /// Header byte 1-2: the first freeblock's offset, or 0 if none.
    pub first_freeblock: u16,
    /// Header byte 3-4: how many cells the page holds.
    pub cell_count: u16,
    /// Header byte 5-6: where the cell content area starts.
    /// A stored 0 means 65536.
    pub content_start: u32,
    /// Header byte 7: free bytes in pieces of 1 to 3, too small to be freeblocks.
    pub fragmented_bytes: u8,
    /// Header bytes 8-11: interior pages only: the child page that holds
    /// every key bigger than the last cell's.
    pub right_most: Option<u32>,
    /// The offset of each cell, in key order.
    pub cell_pointers: Vec<u16>,
    /// The freeblocks chain, in page order.
    pub freeblocks: Vec<FreeBlock>,
}

impl BTreePage {
    /// Parses the page header, the cell pointers and the freeblock chain,
    /// and checks that they all fit inside the page.
    pub fn parse(number: u32, data: Vec<u8>, usable_size: u32) -> Result<BTreePage> {
        // Where the page starts: after the database header on page 1.
        let h = if number == 1 { 100 } else { 0 };
        let page_type = PageType::from_flag(byte(&data, h)?)?;
        let first_freeblock = be16(&data, h + 1)?;
        let cell_count = be16(&data, h + 3)?;
        // 65536 doesn't fit in 2 bytes, so a 64 KiB page with no cells stores 0.
        let content_start = match be16(&data, h + 5)? {
            0 => 65536,
            n => n as u32,
        };
        let fragmented_bytes = byte(&data, h + 7)?;
        let right_most = if page_type.is_leaf() {
            None
        } else {
            Some(be32(&data, h + 8)?)
        };

        // The cell pointer array starts after the page header:
        // one 2-byte offset per cell.
        let array_start = h + page_type.header_len();
        let cell_pointers = (0..cell_count as usize)
            .map(|i| be16(&data, array_start + 2 * i))
            .collect::<Result<Vec<u16>>>()?;
        let array_end = array_start + 2 * cell_count as usize;
        // The pointer array and the cell content area must not overlap.
        if (content_start as usize) < array_end || content_start > usable_size {
            return corrupt(format!(
                "page {number}: cell content starts at {content_start}, outside the page"
            ));
        }
        // Every cell must start inside the cell content area.
        if let Some(&p) = cell_pointers
            .iter()
            .find(|&&p| (p as u32) < content_start || p as u32 >= usable_size)
        {
            return corrupt(format!(
                "page {number}: cell pointer {p} is outside the cell content area"
            ));
        }
        let freeblocks = read_freeblocks(&data, first_freeblock, content_start, usable_size)?;

        Ok(BTreePage {
            number,
            data,
            usable_size,
            header_start: h,
            page_type,
            first_freeblock,
            cell_count,
            content_start,
            fragmented_bytes,
            right_most,
            cell_pointers,
            freeblocks,
        })
    }

    /// Bytes between the end of the cell pointer array and the first cell.
    pub fn unallocated(&self) -> u32 {
        let array_end =
            self.header_start + self.page_type.header_len() + 2 * self.cell_count as usize;
        // `parse` checked that `content_start >= array_end` so this can't go below 0.
        self.content_start - array_end as u32
    }

    /// All free space on the page: the gap, the freeblocks and the fragments.
    pub fn free_bytes(&self) -> u32 {
        let in_blocks: u32 = self.freeblocks.iter().map(|b| b.size as u32).sum();
        self.unallocated() + in_blocks + self.fragmented_bytes as u32
    }
}

/// Follows the freeblock chain. Each block starts with the offset of the
/// next block and its own size. Offsets increase along the chain.
fn read_freeblocks(
    data: &[u8],
    first: u16,
    content_start: u32,
    usable_size: u32,
) -> Result<Vec<FreeBlock>> {
    let mut blocks: Vec<FreeBlock> = Vec::new();
    let mut at = first;
    // A next-offset of 0 ends the chain.
    while at != 0 {
        // FreeBlocks are free space inside the cell content area, never in the header or the pointer array.
        if (at as u32) < content_start {
            return corrupt(format!(
                "freeblock at {at} is before the cell content area at {content_start}"
            ));
        }
        // Each block must come after the last one. This also stops a chain that
        // loops back on itself.
        if blocks.last().is_some_and(|last| at <= last.offset) {
            return corrupt("freeblocks are out of order");
        }
        let next = be16(data, at as usize)?;
        let size = be16(data, at as usize + 2)?;
        // A freeblock is at least 4 bytes: its own next offset and size.
        if size < 4 || at as u32 + size as u32 > usable_size {
            return corrupt(format!(
                "freeblock at {at} with size {size} doesn't fit the page"
            ));
        }
        blocks.push(FreeBlock { offset: at, size });
        at = next;
    }
    Ok(blocks)
}

impl Db {
    /// Reads page `n` and parses it as a b-tree page.
    pub fn btree_page(&self, n: u32) -> Result<BTreePage> {
        BTreePage::parse(n, self.page(n)?, self.usable_size())
    }
}
