use std::{fmt::Debug, rc::Rc};

use serde::{Serialize, ser::SerializeStruct};

use crate::{
    parser::{ParserLoc, WithParserLoc},
    source::ByteSource,
};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Serialize)]
pub struct NodeLoc {
    pub line: usize,
    pub col: usize,
}

pub trait NodeData: Debug + Serialize {}

#[derive(Clone)]
pub struct Node<'src, N: NodeData> {
    pub raw_loc: ParserLoc,
    pub start: NodeLoc,
    pub end: NodeLoc,
    pub src: &'src [u8],
    pub data: N,
}

impl<'src, N: NodeData> Node<'src, N> {
    pub fn new<S: ByteSource<'src>>(data: N, loc: ParserLoc, src: &S) -> Rc<Self> {
        let (start, end) = src.resolve(loc);
        Rc::new(Node { raw_loc: loc, start, end, src: src.source(loc), data })
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
