#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct Location {
    pub start: u32,
    pub end: u32,
}

impl Location {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, byte_index: u32) -> bool {
        byte_index >= self.start && byte_index <= self.end
    }
}
