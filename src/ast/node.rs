use std::fmt::Debug;

use crate::{
    parser::{ParserLoc, WithParserLoc},
    source::ByteSource,
};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct NodeLoc {
    pub line: usize,
    pub col: usize,
}

pub trait NodeData {}

#[derive(Debug, Clone)]
pub struct Node<'src, N: NodeData> {
    pub raw_loc: ParserLoc,
    pub start: NodeLoc,
    pub end: NodeLoc,
    pub src: &'src [u8],
    pub data: N,
}

impl<'src, N: NodeData> Node<'src, N> {
    pub fn new<S: ByteSource<'src>>(data: N, loc: ParserLoc, src: &S) -> Self {
        let (start, end) = src.resolve(loc);
        Node { raw_loc: loc, start, end, src: src.source(loc), data }
    }
}

impl<'src, N: NodeData> WithParserLoc for Node<'src, N> {
    fn loc(&self) -> ParserLoc {
        self.raw_loc
    }
}
