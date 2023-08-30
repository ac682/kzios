#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub page: usize,
    pub program: usize,
    pub heap: usize,
    pub stack: usize,
}

impl MemoryUsage {
    pub const fn new() -> Self {
        Self {
            page: 0,
            program: 0,
            heap: 0,
            stack: 0,
        }
    }
}
