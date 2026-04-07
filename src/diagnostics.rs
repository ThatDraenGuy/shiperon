use std::fmt::Display;

use derive_more::{Display, From};

use crate::{
    ByteSource,
    analyzer::AnalysisError,
    parser::{ParseError, ParserLoc},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorLevel {
    Warn,
    Err,
}

pub trait Renderable<'src> {
    fn render(&self, src: &impl ByteSource<'src>) -> String;
}

#[derive(Debug, Clone)]
pub struct Diagnostic<'src> {
    pub level: ErrorLevel,
    pub loc: ParserLoc,
    pub reason: Reason<'src>,
}

impl<'src> Renderable<'src> for Diagnostic<'src> {
    fn render(&self, src: &impl ByteSource<'src>) -> String {
        let (start, _end) = src.resolve(self.loc);

        let view = str::from_utf8(src.source(self.loc)).unwrap_or("invalid utf-8 string");
        format!(
            "{} at {}:\n{}\n^\n{}",
            match self.level {
                ErrorLevel::Warn => "Warning",
                ErrorLevel::Err => "Error",
            },
            &start,
            view,
            self.reason.render(src)
        )
    }
}

impl<'src> Display for Diagnostic<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:?}] ({:?}) {}", self.level, self.loc, self.reason))
    }
}

#[derive(Debug, Clone, Display, From)]
pub enum Reason<'src> {
    Parse(ParseError<'src>),
    Analysys(AnalysisError<'src>),
}

impl<'src> Renderable<'src> for Reason<'src> {
    fn render(&self, src: &impl ByteSource<'src>) -> String {
        match self {
            Reason::Parse(parse_error) => parse_error.render(src),
            Reason::Analysys(analysis_error) => analysis_error.render(src),
        }
    }
}
