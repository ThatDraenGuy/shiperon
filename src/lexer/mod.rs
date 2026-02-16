mod source;
mod token;

use std::{io, path::Path};

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

type LexResult<T> = Result<T, Token>; // errors have to be serialized into token value anyway, might as well do it immediately

impl<S: ByteSource> Lexer<S> {
    pub fn new(src: S) -> Self {
        Self { src, pos: 0, loc: 0 }
    }

    fn empty_token(&self, token_type: TokenType) -> Token {
        Token { token_type, token_value: vec![], loc: self.get_loc() }
    }
    fn token(&self, token_type: TokenType, token_value: Vec<u8>) -> Token {
        Token { token_type, token_value, loc: self.get_loc() }
    }
    fn make_eof_if_none<T>(&mut self, value: Option<T>) -> Result<T, Token> {
        value.ok_or_else(|| {
            self.pin_loc();
            self.empty_token(TokenRegistry::YYEOF)
        })
    }
    fn make_err(&self, data: Vec<u8>) -> Token {
        self.token(TokenRegistry::YYerror, data)
    }

    fn peek_maybe(&mut self) -> LexResult<Option<u8>> {
        self.src.peek().map_err(|e| S::report_error(e).into_bytes()).map_err(|e| self.make_err(e))
    }
    fn peek(&mut self) -> LexResult<u8> {
        self.peek_maybe().and_then(|maybe_b| self.make_eof_if_none(maybe_b))
    }

    fn next_maybe(&mut self) -> LexResult<Option<u8>> {
        self.pos += 1;
        self.src.next().map_err(|e| self.make_err(S::report_error(&e).into_bytes()))
    }
    fn next(&mut self) -> LexResult<u8> {
        self.next_maybe().and_then(|maybe_b| self.make_eof_if_none(maybe_b))
    }

    fn pin_loc(&mut self) {
        self.loc = self.pos;
    }
    fn get_loc(&self) -> ParserLoc {
        ParserLoc { begin: self.loc as u32, end: self.pos as u32 }
    }

    fn skip_while(&mut self, pred: impl Fn(u8) -> bool) -> LexResult<()> {
        while let Some(b) = self.peek_maybe()? {
            if !pred(b) {
                break;
            }
            self.next_maybe()?;
        }
        Ok(())
    }

    fn take_while(&mut self, pred: impl Fn(u8) -> bool) -> LexResult<Vec<u8>> {
        let mut buf = vec![];
        while let Some(b) = self.peek_maybe()? {
            if !pred(b) {
                break;
            }
            buf.push(b);
            self.next_maybe()?;
        }
        Ok(buf)
    }

    fn read_symbolic(&mut self) -> LexResult<Token> {
        Ok(match self.next()? {
            b'.' => self.empty_token(TokenRegistry::tDOT),
            b',' => self.empty_token(TokenRegistry::tCOMMA),
            b'(' => self.empty_token(TokenRegistry::tLPAREN),
            b')' => self.empty_token(TokenRegistry::tRPAREN),
            b':' => {
                if let Some(b'=') = self.peek_maybe()? {
                    self.next_maybe()?;
                    self.empty_token(TokenRegistry::tASSIGN)
                } else {
                    self.empty_token(TokenRegistry::tCOLON)
                }
            },
            b'=' => {
                if let Some(b'>') = self.peek_maybe()? {
                    self.next_maybe()?;
                    self.empty_token(TokenRegistry::tARROW)
                } else {
                    self.token(TokenRegistry::YYUNDEF, vec![b'='])
                }
            },
            b => self.token(TokenRegistry::YYUNDEF, vec![b]),
        })
    }

    fn read_keyword_or_id(&mut self) -> LexResult<Token> {
        let bytes = self.take_while(|b| b.is_ascii_alphanumeric())?;
        Ok(
            match str::from_utf8(&bytes)
                .map_err(|_e| self.make_err("Invalid UTF-8 encountered".to_owned().into_bytes()))?
            {
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
            },
        )
    }

    fn read_numeric(&mut self) -> LexResult<Token> {
        let negated = if self.peek()? == b'-' {
            self.next_maybe()?;
            true
        } else {
            false
        };

        let bytes = self.take_while(|b| b.is_ascii_digit())?;
        str::from_utf8(&bytes)
            .map_err(|_e| self.make_err("Invalid UTF-8 encountered".to_owned().into_bytes()))?
            .parse::<u32>()
            .map_err(|_e| self.make_err("Invalid num encountered".to_owned().into_bytes()))?; //TODO smarter
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
        self.next_token().unwrap_or_else(|e| e)
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
