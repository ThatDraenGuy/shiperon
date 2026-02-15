use std::fmt::Debug;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ParserLoc {
    pub begin: u32,
    pub end: u32,
}

impl Debug for ParserLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}...{}", self.begin, self.end))
    }
}
