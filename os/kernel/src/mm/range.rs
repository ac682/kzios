use alloc::vec::Vec;
use erhino_shared::mem::{PageNumber, page::PageLevel};

use crate::{print, println};

pub struct PageRange {
    start: PageNumber,
    end: PageNumber,
}

impl PageRange {
    pub fn new(start: PageNumber, count: usize) -> Self {
        Self {
            start,
            end: start + count,
        }
    }

    pub fn segments(&self, max_level: PageLevel) -> <Vec<PageSegment> as IntoIterator>::IntoIter {
        let mut segments = Vec::<PageSegment>::new();
        Self::segments_internal(&mut segments, self.start, self.start + self.end, max_level);
        segments.into_iter()
    }

    fn segments_internal(
        container: &mut Vec<PageSegment>,
        start: PageNumber,
        end: PageNumber,
        level: PageLevel,
    ) {
        if level == PageLevel::Kilo {
            for i in start..end {
                container.push(PageSegment::new(i, PageLevel::Kilo));
            }
        } else {
            let l = level.floor(start);
            let r = level.ceil(end);
            let level_count = level.measure((r - l) as usize);
            for i in 0..level_count {
                let page_start = l + level.shift(i);
                let page_end = l + level.shift(i + 1);
                if i == 0 && start > page_start {
                    Self::segments_internal(
                        container,
                        start,
                        page_end,
                        level.next_level().unwrap(),
                    );
                } else if i == level_count - 1 && end < page_end {
                    Self::segments_internal(
                        container,
                        page_start,
                        end,
                        level.next_level().unwrap(),
                    );
                } else {
                    container.push(PageSegment::new(page_start, level));
                }
            }
        }
    }
}

pub struct PageSegment {
    number: PageNumber,
    size: PageLevel,
}

impl PageSegment {
    pub fn new(page_number: PageNumber, size: PageLevel) -> Self {
        Self {
            number: page_number,
            size,
        }
    }

    pub fn page_number(&self) -> PageNumber {
        self.number
    }

    pub fn size(&self) -> PageLevel {
        self.size
    }
}
