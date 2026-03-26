use std::fmt::Debug;

pub trait ByteSourceIter: Debug {
    type Error;
    fn peek(&mut self) -> Result<Option<u8>, &Self::Error>;
    fn next(&mut self) -> Result<Option<u8>, Self::Error>;
    fn report_error(e: &Self::Error) -> String;
}
