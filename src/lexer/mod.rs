mod source;
mod token;

use std::{error::Error, io, path::Path};

pub use source::ByteSource;
pub use token::Token;

use crate::{
    lexer::source::{FileSource, StrSource},
    parser::ParserLoc,
};

type TokenType = i32;

#[derive(Debug)]
pub struct TokenRegistry;

#[derive(Debug)]
pub struct Lexer<T: ByteSource> {
    src: T,
    pos: usize,
    loc: usize,
}

enum LexError {
    Eof,
    Inner(Vec<u8>),
}

impl<E: Error> From<E> for LexError {
    fn from(value: E) -> Self {
        Self::Inner(value.to_string().into_bytes())
    }
}

type LexResult<T> = Result<T, LexError>;

impl<S: ByteSource> Lexer<S> {
    pub fn new(src: S) -> Self {
        Self { src, pos: 0, loc: 0 }
    }

    fn peek(&mut self) -> LexResult<u8> {
        match self.src.peek() {
            Ok(Some(b)) => Ok(b),
            Ok(None) => Err(LexError::Eof),
            Err(e) => Err(LexError::Inner(S::report_error(e).into_bytes())),
        }
    }
    fn next(&mut self) -> LexResult<u8> {
        self.pos += 1;
        match self.src.next() {
            Ok(Some(b)) => Ok(b),
            Ok(None) => Err(LexError::Eof),
            Err(e) => Err(LexError::Inner(S::report_error(&e).into_bytes())),
        }
    }

    fn pin_loc(&mut self) {
        self.loc = self.pos;
    }

    fn get_loc(&mut self) -> ParserLoc {
        ParserLoc { begin: self.loc as u32, end: self.pos as u32 }
    }

    fn skip_while(&mut self, pred: impl Fn(u8) -> bool) -> LexResult<()> {
        loop {
            let b = self.peek()?;
            if !pred(b) {
                break;
            }
            self.next()?;
        }
        Ok(())
    }

    fn take_until(&mut self, pred: impl Fn(u8) -> bool) -> LexResult<Vec<u8>> {
        let mut buf = vec![];
        while let Ok(b) = self.peek() {
            if !pred(b) {
                break;
            }
            buf.push(b);
            self.next()?;
        }
        Ok(buf)
    }

    fn empty_token(&mut self, token_type: TokenType) -> Token {
        Token { token_type, token_value: vec![], loc: self.get_loc() }
    }
    fn token(&mut self, token_type: TokenType, token_value: Vec<u8>) -> Token {
        Token { token_type, token_value, loc: self.get_loc() }
    }

    fn read_symbolic(&mut self) -> LexResult<Token> {
        Ok(match self.next()? {
            b'.' => self.empty_token(TokenRegistry::tDOT),
            b',' => self.empty_token(TokenRegistry::tCOMMA),
            b'(' => self.empty_token(TokenRegistry::tLPAREN),
            b')' => self.empty_token(TokenRegistry::tRPAREN),
            b':' => {
                if let Ok(b'=') = self.peek() {
                    self.next()?;
                    self.empty_token(TokenRegistry::tASSIGN)
                } else {
                    self.empty_token(TokenRegistry::tCOLON)
                }
            },
            b'=' => {
                if let Ok(b'>') = self.peek() {
                    self.next()?;
                    self.empty_token(TokenRegistry::tARROW)
                } else {
                    self.token(TokenRegistry::YYUNDEF, vec![b'='])
                }
            },
            b => self.token(TokenRegistry::YYUNDEF, vec![b]),
        })
    }

    fn read_keyword_or_id(&mut self) -> LexResult<Token> {
        let bytes = self.take_until(|b| b.is_ascii_alphanumeric())?;
        Ok(match str::from_utf8(&bytes)? {
            "class" => self.empty_token(TokenRegistry::kCLASS),
            "extends" => self.empty_token(TokenRegistry::kEXTENDS),
            "is" => self.empty_token(TokenRegistry::kIS),
            "end" => self.empty_token(TokenRegistry::kEND),
            "var" => self.empty_token(TokenRegistry::kVAR),
            "method" => self.empty_token(TokenRegistry::kMETHOD),
            "this" => self.empty_token(TokenRegistry::kTHIS),
            "while" => self.empty_token(TokenRegistry::kWHILE),
            "loop" => self.empty_token(TokenRegistry::kLOOP),
            "if" => self.empty_token(TokenRegistry::kIF),
            "then" => self.empty_token(TokenRegistry::kTHEN),
            "else" => self.empty_token(TokenRegistry::kELSE),
            "return" => self.empty_token(TokenRegistry::kRETURN),
            _ => self.token(TokenRegistry::tIDENTIFIER, bytes),
        })
    }

    fn read_numeric(&mut self) -> LexResult<Token> {
        let negated = if self.peek()? == b'-' {
            self.next()?;
            true
        } else {
            false
        };

        let bytes = self.take_until(|b| b.is_ascii_digit())?;
        str::from_utf8(&bytes)?.parse::<u32>()?; //TODO smarter
        let mut value = if negated { vec![b'-'] } else { vec![] };
        value.extend_from_slice(&bytes);
        Ok(self.token(TokenRegistry::tINTEGER, value))
        //TODO floats
    }

    fn next_token(&mut self) -> LexResult<Token> {
        let peek = self.peek()?;

        // skip whitespace
        if peek.is_ascii_whitespace() {
            self.skip_while(|b| b.is_ascii_whitespace())?;
        }

        // handle comments
        if peek == b'/' {
            self.next()?;
            match self.peek()? {
                b'/' => {
                    // singleline comment, skip until line break
                    self.skip_while(|b| b != b'\n')?;
                },
                b'*' => {
                    //TODO multiline comment, skip until `*/`
                },
                _ => {},
            };
        }

        self.pin_loc();

        match self.peek()? {
            b if b.is_ascii_alphabetic() => self.read_keyword_or_id(),
            b if b.is_ascii_digit() || b == b'-' => self.read_numeric(),
            _ => self.read_symbolic(),
        }
    }

    pub fn yylex(&mut self) -> Token {
        self.next_token().unwrap_or_else(|e| match e {
            LexError::Eof => {
                self.pin_loc();
                self.empty_token(TokenRegistry::YYEOF)
            },
            LexError::Inner(data) => self.token(TokenRegistry::YYerror, data),
        })
    }
}

impl Lexer<FileSource> {
    pub fn of_file<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self::new(FileSource::new(path)?))
    }
}

impl<'a> Lexer<StrSource<'a>> {
    pub fn of_str(str: &'a str) -> Self {
        Self::new(StrSource::new(str))
    }
}
