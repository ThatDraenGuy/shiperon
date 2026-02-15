mod token;

pub use token::Token;

use crate::parser::ParserLoc;

type TokenType = i32;

#[derive(Debug)]
pub struct Lexer {
    src: Vec<u8>,
    pos: usize,
    loc: usize,
}

impl Lexer {
    pub fn new<TSrc>(src: TSrc) -> Self
    where
        TSrc: Into<Vec<u8>>,
    {
        Self { src: src.into(), pos: 0, loc: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }
    fn next(&mut self) -> Option<u8> {
        let res = self.peek();
        self.pos += 1;
        res
    }

    fn set_loc(&mut self) {
        self.loc = self.pos;
    }

    fn consume_loc(&mut self) -> ParserLoc {
        ParserLoc { begin: self.loc as u32, end: self.pos as u32 }
    }

    fn skip_while(&mut self, pred: impl Fn(u8) -> bool) {
        while let Some(b) = self.peek() {
            if !pred(b) {
                break;
            }
            self.next();
        }
    }

    fn take_until(&mut self, pred: impl Fn(u8) -> bool) -> &[u8] {
        let start = self.pos;
        self.skip_while(pred);
        let end = self.pos;
        &self.src[start..end] //TODO check indexes
    }

    fn empty_token(&mut self, token_type: TokenType) -> Token {
        Token { token_type, token_value: vec![], loc: self.consume_loc() }
    }
    fn token(&mut self, token_type: TokenType, token_value: Vec<u8>) -> Token {
        Token { token_type, token_value, loc: self.consume_loc() }
    }

    fn read_symbolic(&mut self) -> Option<Token> {
        Some(match self.next()? {
            b'.' => self.empty_token(Self::tDOT),
            b',' => self.empty_token(Self::tCOMMA),
            b'(' => self.empty_token(Self::tLPAREN),
            b')' => self.empty_token(Self::tRPAREN),
            b':' => {
                if self.peek() == Some(b'=') {
                    self.next();
                    self.empty_token(Self::tASSIGN)
                } else {
                    self.empty_token(Self::tCOLON)
                }
            },
            b'=' => {
                if self.peek() == Some(b'>') {
                    self.next();
                    self.empty_token(Self::tARROW)
                } else {
                    self.token(Self::YYUNDEF, vec![b'='])
                }
            },
            b => self.token(Self::YYUNDEF, vec![b]),
        })
    }

    fn read_keyword_or_id(&mut self) -> Option<Token> {
        let bytes = self.take_until(|b| b.is_ascii_alphanumeric());
        Some(match str::from_utf8(bytes).ok()? {
            "class" => self.empty_token(Self::kCLASS),
            "extends" => self.empty_token(Self::kEXTENDS),
            "is" => self.empty_token(Self::kIS),
            "end" => self.empty_token(Self::kEND),
            "var" => self.empty_token(Self::kVAR),
            "method" => self.empty_token(Self::kMETHOD),
            "this" => self.empty_token(Self::kTHIS),
            "while" => self.empty_token(Self::kWHILE),
            "loop" => self.empty_token(Self::kLOOP),
            "if" => self.empty_token(Self::kIF),
            "then" => self.empty_token(Self::kTHEN),
            "else" => self.empty_token(Self::kELSE),
            "return" => self.empty_token(Self::kRETURN),
            _ => {
                let value = bytes.to_vec();
                self.token(Self::tIDENTIFIER, value)
            },
        })
    }

    fn read_numeric(&mut self) -> Option<Token> {
        let negated = if self.peek()? == b'-' {
            self.next();
            true
        } else {
            false
        };

        let bytes = self.take_until(|b| b.is_ascii_digit());
        str::from_utf8(bytes).ok()?.parse::<u32>().ok()?; //TODO smarter
        let mut value = if negated { vec![b'-'] } else { vec![] };
        value.extend_from_slice(bytes);
        Some(self.token(Self::tINTEGER, value))
        //TODO floats
    }

    fn next_token(&mut self) -> Option<Token> {
        let peek = self.peek()?;

        // skip whitespace
        if peek.is_ascii_whitespace() {
            self.skip_while(|b| b.is_ascii_whitespace());
        }

        // handle comments
        if peek == b'/' {
            self.next();
            match self.peek()? {
                b'/' => {
                    // singleline comment, skip until line break
                    self.skip_while(|b| b != b'\n');
                },
                b'*' => {
                    //TODO multiline comment, skip until `*/`
                },
                _ => {},
            };
        }

        self.set_loc();

        match self.peek()? {
            b if b.is_ascii_alphabetic() => self.read_keyword_or_id(),
            b if b.is_ascii_digit() || b == b'-' => self.read_numeric(),
            _ => self.read_symbolic(),
        }
    }

    pub fn yylex(&mut self) -> Token {
        self.next_token().unwrap_or_else(|| Token {
            token_type: Self::YYEOF,
            token_value: vec![],
            loc: ParserLoc { begin: self.pos as u32, end: self.pos as u32 },
        })
    }
}
