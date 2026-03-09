// mod source;
mod token;

// pub use source::ByteSourceIter;
pub use token::{Token, TokenType, TokenValue};

use crate::{
    // lexer::source::{FileSource, StrSource},
    parser::ParserLoc,
    source::{ByteSource, ByteSourceIter, StrSource},
};

#[derive(Debug)]
pub struct TokenRegistry;

#[derive(Debug)]
pub struct Lexer<'src, S: ByteSource<'src>> {
    pub src: S,
    iter: <S as ByteSource<'src>>::Iter,
    pos: usize,
    loc: usize,
}

type LexResult<T> = Result<T, Token>; // errors have to be serialized into token value anyway, might as well do it immediately

impl<'src, S: ByteSource<'src>> Lexer<'src, S> {
    pub fn new(src: S) -> Self {
        let iter = src.iter();
        Self { src, iter, pos: 0, loc: 0 }
    }

    fn empty_token(&self, token_type: TokenType) -> Token {
        Token { token_type, token_value: TokenValue::None, loc: self.get_loc() }
    }
    fn token(&self, token_type: TokenType, token_value: TokenValue) -> Token {
        Token { token_type, token_value, loc: self.get_loc() }
    }
    fn make_eof_if_none<T>(&mut self, value: Option<T>) -> Result<T, Token> {
        value.ok_or_else(|| {
            self.pin_loc();
            self.empty_token(TokenRegistry::YYEOF)
        })
    }
    fn make_err<T>(&self, data: T) -> Token
    where
        T: Into<String>,
    {
        self.token(TokenRegistry::YYerror, TokenValue::String(data.into()))
    }

    fn peek_maybe(&mut self) -> LexResult<Option<u8>> {
        self.iter.peek().map_err(S::Iter::report_error).map_err(|e| self.make_err(e))
    }
    fn peek(&mut self) -> LexResult<u8> {
        self.peek_maybe().and_then(|maybe_b| self.make_eof_if_none(maybe_b))
    }

    fn next_maybe(&mut self) -> LexResult<Option<u8>> {
        self.pos += 1;
        self.iter.next().map_err(|e| self.make_err(S::Iter::report_error(&e)))
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

    fn take_into_while(
        &mut self,
        target: &mut Vec<u8>,
        pred: impl Fn(u8) -> bool,
    ) -> LexResult<()> {
        while let Some(b) = self.peek_maybe()? {
            if !pred(b) {
                break;
            }
            target.push(b);
            self.next_maybe()?;
        }
        Ok(())
    }

    fn take_while(&mut self, pred: impl Fn(u8) -> bool) -> LexResult<Vec<u8>> {
        let mut buf = vec![];
        self.take_into_while(&mut buf, pred)?;
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
                    self.token(TokenRegistry::YYUNDEF, TokenValue::String("=".to_owned()))
                }
            },
            b => self.token(TokenRegistry::YYUNDEF, TokenValue::String((b as char).to_string())),
        })
    }

    fn read_keyword_or_id(&mut self) -> LexResult<Token> {
        let bytes = self.take_while(|b| b.is_ascii_alphanumeric())?;
        let string =
            String::from_utf8(bytes).map_err(|_e| self.make_err("Invalid UTF-8 encountered"))?;
        Ok(match string.as_str() {
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
            _ => self.token(TokenRegistry::tIDENTIFIER, TokenValue::String(string)),
        })
    }

    fn read_numeric(&mut self) -> LexResult<Token> {
        let mut bytes = vec![];
        if self.peek()? == b'-' {
            bytes.push(self.next()?);
        };

        self.take_into_while(&mut bytes, |b| b.is_ascii_digit())?;

        //TODO long ints & floats??
        if let Some(b'.') = self.peek_maybe()? {
            //floats
            bytes.push(self.next()?);
            if !self.peek_maybe()?.is_some_and(|b| b.is_ascii_digit()) {
                return Ok(self.token(
                    TokenRegistry::YYUNDEF,
                    TokenValue::String(
                        String::from_utf8(bytes)
                            .map_err(|_e| self.make_err("Invalid UTF-8 encountered"))?,
                    ),
                ));
            }
            self.take_into_while(&mut bytes, |b| b.is_ascii_digit())?;
            let fl = str::from_utf8(&bytes)
                .map_err(|_e| self.make_err("Invalid UTF-8 encountered"))?
                .parse::<f32>()
                .map_err(|_e| self.make_err("Invalid num encountered"))?; //TODO smarter
            Ok(self.token(TokenRegistry::tFLOAT, TokenValue::Float(fl)))
        } else {
            //ints
            let i = str::from_utf8(&bytes)
                .map_err(|_e| self.make_err("Invalid UTF-8 encountered"))?
                .parse::<i32>()
                .map_err(|_e| self.make_err("Invalid num encountered"))?; //TODO smarter
            Ok(self.token(TokenRegistry::tINTEGER, TokenValue::Int(i)))
        }
    }

    fn next_token(&mut self) -> LexResult<Token> {
        loop {
            match self.peek()? {
                //skip whitespace
                b if b.is_ascii_whitespace() => self.skip_while(|b| b.is_ascii_whitespace())?,
                //handle comments
                b'/' => {
                    self.next()?;
                    match self.peek_maybe()? {
                        Some(b'/') => {
                            // singleline comment, skip until line break
                            self.skip_while(|b| b != b'\n')?;
                        },
                        _ => {
                            return Ok(self.token(
                                TokenRegistry::YYUNDEF,
                                TokenValue::String("/".to_owned()),
                            ));
                        },
                    };
                },
                _ => break,
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

impl<'src> Lexer<'src, StrSource<'src>> {
    pub fn of_str(str: &'src str) -> Self {
        Self::new(StrSource::new(str))
    }
}
