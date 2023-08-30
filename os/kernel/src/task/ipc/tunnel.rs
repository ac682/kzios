use erhino_shared::{mem::PageNumber, proc::Pid};

use crate::mm::frame::FrameTracker;

pub struct Tunnel {
    key: usize,
    pub owner: Pid,
    pub first: Option<(Pid, PageNumber)>,
    pub second: Option<(Pid, PageNumber)>,
    frame: FrameTracker,
}

impl Tunnel {
    pub fn new(key: usize, owner: Pid, frame: FrameTracker) -> Self {
        Self {
            key,
            owner: owner,
            first: None,
            second: None,
            frame,
        }
    }

    pub fn key(&self) -> usize {
        self.key
    }

    pub fn page_number(&self) -> PageNumber {
        self.frame.start()
    }

    pub fn link(&mut self, pid: Pid, number: PageNumber) -> bool {
        if let Some((first, _)) = self.first {
            if self.second.is_none() {
                if pid == self.owner || first == self.owner {
                    self.second = Some((pid, number));
                    return true;
                }
            }
        } else {
            self.first = Some((pid, number));
            self.owner = pid;
            return true;
        }
        false
    }

    pub fn unlink(&mut self, pid: Pid) -> Option<(bool, PageNumber)> {
        if self.second.is_some_and(|(s, _)| s == pid) {
            return self.second.take().map(|f| (false, f.1));
        } else {
            if self.first.is_some_and(|(f, _)| f == pid) {
                let first = self.first.take();
                self.first = self.second.take();
                if let Some((second, _)) = self.first {
                    self.owner = second;
                    first.map(|f| (false, f.1))
                } else {
                    first.map(|f| (true, f.1))
                }
            } else {
                None
            }
        }
    }
}

pub struct Endpoint {
    index: usize,
    key: usize,
}

impl Endpoint {
    pub const fn new(index: usize, key: usize) -> Self {
        Self { index, key }
    }

    pub fn key(&self) -> usize {
        self.key
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
