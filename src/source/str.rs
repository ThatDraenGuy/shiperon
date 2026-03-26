use std::{cmp::Ordering, iter::Peekable, str::Bytes};

use crate::{ast::NodeLoc, parser::ParserLoc};

use super::{ByteSource, ByteSourceIter};

#[derive(Debug)]
pub struct StrSource<'src> {
    str: &'src str,
    line_breaks: Vec<(usize, usize)>,
}

impl<'src> StrSource<'src> {
    pub fn new(str: &'src str) -> Self {
        let mut line_start = 0;
        let mut line_breaks = vec![];
        for line in str.as_bytes().split(|b| *b == b'\n') {
            let line_end = line_start + line.len();
            line_breaks.push((line_start, line_end));
            line_start = line_end + 1;
        }
        Self { str, line_breaks }
    }

    fn get_from_pos(&self, pos: usize) -> NodeLoc {
        let line_idx = self
            .line_breaks
            .binary_search_by(|(line_start, line_end)| {
                if *line_end < pos {
                    return Ordering::Less;
                }
                if *line_start > pos {
                    return Ordering::Greater;
                }
                Ordering::Equal
            })
            .expect("there should be line");

        let (line_start, _) = self.line_breaks[line_idx];
        NodeLoc { line: line_idx + 1, col: pos - line_start + 1 }
    }
}

impl<'src> ByteSource<'src> for StrSource<'src> {
    type Iter = Peekable<Bytes<'src>>;

    fn iter(&self) -> Self::Iter {
        self.str.bytes().peekable()
    }

    fn source(&self, loc: ParserLoc) -> &'src [u8] {
        &self.str.as_bytes()[loc.begin as usize..loc.end as usize]
    }

    fn resolve(&self, loc: ParserLoc) -> (NodeLoc, NodeLoc) {
        (self.get_from_pos(loc.begin as usize), self.get_from_pos(loc.end as usize))
    }
}

impl<'src> ByteSourceIter for Peekable<Bytes<'src>> {
    type Error = ();

    fn peek(&mut self) -> Result<Option<u8>, &Self::Error> {
        Ok(self.peek().copied())
    }

    fn next(&mut self) -> Result<Option<u8>, Self::Error> {
        Ok(std::iter::Iterator::next(self))
    }

    fn report_error(_e: &Self::Error) -> String {
        "".into()
    }
}
