mod iter;
pub use iter::ByteSourceIter;

mod str;
pub use str::StrSource;

use crate::{ast::NodeLoc, parser::ParserLoc};
use std::fmt::Debug;

pub trait ByteSource<'src>: Debug {
    type Iter: ByteSourceIter;

    fn iter(&self) -> Self::Iter;

    fn source(&self, loc: ParserLoc) -> &'src [u8];
    fn source_str(&self, loc: ParserLoc) -> &'src str;

    fn resolve(&self, loc: ParserLoc) -> (NodeLoc, NodeLoc);
}
