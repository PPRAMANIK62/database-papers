use crate::{
    Db, Result,
    bytes::{be16, be32, byte},
    corrupt,
};

/// The four kinds of b-tree page, from the flag byte at offset 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageType {
    IndexInterior, // 0x02
    TableInterior, // 0x05
    IndexLeaf,     // 0x0a
    TableLeaf,     // 0x0d
}

impl PageType {
    pub fn from_flag(flag: u8) -> Result<PageType> {
        match flag {
            0x02 => Ok(PageType::IndexInterior),
            0x05 => Ok(PageType::TableInterior),
            0x0a => Ok(PageType::IndexLeaf),
            0x0d => Ok(PageType::TableLeaf),
            other => corrupt(format!("{other:#04x} is not a b-tree page type")),
        }
    }

    pub fn is_leaf(self) -> bool {
        matches!(self, PageType::IndexLeaf | PageType::TableLeaf)
    }

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

/// A run of 4 or more free bytes inside the cell content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeBlock {
    pub offset: u16,
    pub size: u16,
}

/// One parsed b-tree page (spec section 1.6)
#[derive(Debug, Clone)]
pub struct BTreePage {
    pub number: u32,
    pub data: Vec<u8>,
    pub usable_size: u32,
    /// 100 on page 1, where the database header comes first, 0 elsewhere.
    pub header_start: usize,
    pub page_type: PageType,
    pub first_freeblock: u16,
    pub cell_count: u16,
    pub content_start: u32,
    pub fragmented_bytes: u8,
    pub right_most: Option<u32>,
    pub cell_pointers: Vec<u16>,
    pub freeblocks: Vec<FreeBlock>,
}

impl BTreePage {
    pub fn parse(number: u32, data: Vec<u8>, usable_size: u32) -> Result<BTreePage> {
        let h = if number == 1 { 100 } else { 0 };
        let page_type = PageType::from_flag(byte(&data, h)?)?;
        let first_freeblock = be16(&data, h + 1)?;
        let cell_count = be16(&data, h + 3)?;
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

        let array_start = h + page_type.header_len();
        let cell_pointers = (0..cell_count as usize)
            .map(|i| be16(&data, array_start + 2 * i))
            .collect::<Result<Vec<u16>>>()?;
        let array_end = array_start + 2 * cell_count as usize;
        if (content_start as usize) < array_end || content_start > usable_size {
            return corrupt(format!(
                "page {number}: cell content starts at {content_start}, outside the page"
            ));
        }
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
    while at != 0 {
        if (at as u32) < content_start {
            return corrupt(format!(
                "freeblock at {at} is before the cell content area at {content_start}"
            ));
        }
        if blocks.last().is_some_and(|last| at <= last.offset) {
            return corrupt("freeblocks are out of order");
        }
        let next = be16(data, at as usize)?;
        let size = be16(data, at as usize + 2)?;
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
    pub fn btree_page(&self, n: u32) -> Result<BTreePage> {
        BTreePage::parse(n, self.page(n)?, self.usable_size())
    }
}
