use std::fmt::Debug;

use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub struct ParserLoc {
    pub begin: u32,
    pub end: u32,
}

impl ParserLoc {
    pub fn merge(left: Self, right: Self) -> Self {
        Self { begin: left.begin, end: right.end }
    }

    pub fn merge_from<L: WithParserLoc, R: WithParserLoc>(left: &L, right: &R) -> Self {
        Self::merge(left.loc(), right.loc())
    }
}

impl Debug for ParserLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:0>3}..{:0>3}", self.begin, self.end))
    }
}

impl Serialize for ParserLoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:0>3}..{:0>3}", self.begin, self.end))
    }
}

pub trait WithParserLoc {
    fn loc(&self) -> ParserLoc;
}
