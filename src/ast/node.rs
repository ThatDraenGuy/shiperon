use std::{
    fmt::{Debug, Display},
    ops::Deref,
    rc::Rc,
};

use serde::{Serialize, ser::SerializeStruct};

use crate::{
    parser::{ParserLoc, WithParserLoc},
    source::ByteSource,
};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeLoc {
    pub line: usize,
    pub col: usize,
}

impl Debug for NodeLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Ln {}, Col {}", self.line, self.col))
    }
}

impl Display for NodeLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Serialize for NodeLoc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("Ln {}, Col {}", self.line, self.col))
    }
}

pub trait NodeData: Debug + Serialize {
    fn name() -> &'static str;
}

#[derive(Clone)]
pub struct Node<'src, N: NodeData> {
    pub raw_loc: ParserLoc,
    pub start: NodeLoc,
    pub end: NodeLoc,
    pub src: &'src [u8],
    pub data: N,
}

impl<'src, N: NodeData> Deref for Node<'src, N> {
    type Target = N;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'src, N: NodeData> Node<'src, N> {
    pub fn new<S: ByteSource<'src>>(data: N, loc: ParserLoc, src: &S) -> Rc<Self> {
        let (start, end) = src.resolve(loc);
        Rc::new(Node { raw_loc: loc, start, end, src: src.source(loc), data })
    }

    pub fn src(&self) -> &'src str {
        str::from_utf8(self.src).unwrap_or("invalid utf-8 string")
    }
}

impl<'src, N: NodeData> WithParserLoc for Node<'src, N> {
    fn loc(&self) -> ParserLoc {
        self.raw_loc
    }
}

impl<'src, N: NodeData> Debug for Node<'src, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("raw_loc", &self.raw_loc)
            .field("start", &self.start)
            .field("end", &self.end)
            .field("src", &str::from_utf8(self.src).unwrap_or("non utf-8 fragment"))
            .field("data", &self.data)
            .finish()
    }
}

impl<'src, N: NodeData> Display for Node<'src, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} ({} - {})", N::name(), self.start, self.end))
    }
}

impl<'src, N: NodeData> Serialize for Node<'src, N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Node", 5)?;
        state.serialize_field("raw_loc", &self.raw_loc)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.serialize_field("src", &str::from_utf8(self.src).unwrap_or("non utf-8 fragment"))?;
        state.serialize_field("data", &self.data)?;
        state.end()
    }
}
