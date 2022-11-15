use crate::mem::PageNumber;

/// Three page level for memory paging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLevel {
    /// Each page has 4096 bytes
    Kilo,
    /// Mega page consist of 512 kilo pages
    Mega,
    /// Giga page consist of 512 mega pages
    Giga,
}

impl PageLevel {
    /// The numerical level of page
    pub fn value(&self) -> u8 {
        match self {
            Self::Kilo => 0,
            Self::Mega => 1,
            Self::Giga => 2,
        }
    }

    /// Bytes of one this page measures
    pub fn size_of_bytes(&self) -> usize {
        self.size_of_pages() * 4096
    }

    /// Kilo pages of one this page measures
    pub fn size_of_pages(&self) -> usize {
        match self {
            Self::Kilo => 1,
            Self::Mega => 512,
            Self::Giga => 512 * 512,
        }
    }

    /// The smaller level of this page level
    pub fn next_level(&self) -> Option<PageLevel> {
        match self {
            PageLevel::Giga => Some(PageLevel::Mega),
            PageLevel::Mega => Some(PageLevel::Kilo),
            PageLevel::Kilo => None,
        }
    }

    /// Level-aligned page number
    pub fn floor(&self, page_number: PageNumber) -> PageNumber {
        page_number >> (9 * self.value()) << (9 * self.value())
    }

    /// Next level-aligned page number
    pub fn ceil(&self, page_number: PageNumber) -> PageNumber {
        (page_number >> (9 * self.value()) << (9 * self.value())) + self.shift(1)
    }

    /// Biggest pages a range of page number can hold
    pub fn measure(&self, page_count: usize) -> usize {
        page_count >> (9 * self.value())
    }

    /// The certain segment of page number according to its level
    pub fn extract(&self, page_number: PageNumber) -> usize {
        (page_number >> (9 * self.value())) & 0x1ff
    }

    /// Make size of pages to page number of its level
    pub fn shift(&self, index: PageNumber) -> PageNumber {
        index << (9 * self.value())
    }
}

impl PartialOrd for PageLevel {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value().partial_cmp(&other.value())
    }
}
