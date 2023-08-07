//Generate tokens from text

use std::str::Chars;

use crate::{utils::{FileInfo, Position}, errors::{raise_error, ErrorType}};

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    I32,
    Plus,
    Newline,
    Eof,
    Equal,
    Identifier,
    Keyword,
    Ampersand,
    I8,
    I16,
    I64,
    I128,
}

pub struct Lexer<'a> {
    pub current: char,
    pub line: usize,
    pub col: usize,
    pub chars: Chars<'a>,
    pub info: FileInfo<'a>,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub data: String,
    pub tp: TokenType,
    pub start: Position, //Inclusive
    pub end: Position,   //Inclusive
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: '{}'", self.tp, self.data)
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TokenType::I32 => write!(f, "i32"),
            TokenType::Plus => write!(f, "plus"),
            TokenType::Newline => write!(f, "\\n"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Equal => write!(f, "equal"),
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::Keyword => write!(f, "keyword"),
            TokenType::Ampersand => write!(f, "ampersand"),
            TokenType::I8 => write!(f, "i8"),
            TokenType::I16 => write!(f, "i16"),
            TokenType::I64 => write!(f, "i64"),
            TokenType::I128 => write!(f, "i128"),
        }
    }
}

pub fn new<'a>(info: &mut crate::utils::FileInfo<'a>) -> Lexer<'a> {
    let mut chars = info.data.clone();
    let current = chars.next().unwrap();
    Lexer {
        current,
        line: 0,
        col: 0,
        chars,
        info: info.clone(),
    }
}

fn advance(lexer: &mut Lexer) {
    let next = lexer.chars.next();

    if lexer.current != '\n' && lexer.current != '\r' {
        lexer.col += unicode_width::UnicodeWidthChar::width(lexer.current).unwrap();
    }

    if next.is_none() {
        lexer.current = '\0';
        return;
    }

    let next = next.unwrap();

    if lexer.current == '\n' || lexer.current == '\r' {
        lexer.line += 1;
        lexer.col = 0;
    }

    lexer.current = next;
}

#[allow(dead_code)]
pub fn print_tokens(len: usize, tokens: &Vec<Token>) {
    println!("Generated tokens:\n========================");
    println!("Token list ({} tokens)", len);
    println!("------------------------");
    let mut idx: usize = 1;
    for tok in tokens {
        println!("{} | {} {}", idx, tok, tok.start.line);
        idx += 1;
    }
    println!("========================");
}

pub fn is_identi(cur: char) -> bool {
    !(cur.is_ascii_digit() || cur == '+' || cur == '\n' || cur == '=' || cur.is_whitespace())
}

pub fn generate_tokens(lexer: &mut Lexer, kwds: &[String]) -> (usize, Vec<Token>) {
    let mut tokens: Vec<Token> = Vec::new();

    while lexer.current != '\0' {
        let cur = lexer.current;

        if cur.is_ascii_digit() {
            tokens.push(make_number(lexer));
        } else if cur == '+' {
            tokens.push(Token {
                data: String::from("+"),
                tp: TokenType::Plus,
                start: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
                end: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
            });
            advance(lexer);
        } else if cur == '\n' {
            tokens.push(Token {
                data: String::from("\\n"),
                tp: TokenType::Newline,
                start: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
                end: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
            });
            advance(lexer);
        } else if cur == '=' {
            tokens.push(Token {
                data: String::from("="),
                tp: TokenType::Equal,
                start: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
                end: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
            });
            advance(lexer);
        } else if cur == '&' {
            tokens.push(Token {
                data: String::from("&"),
                tp: TokenType::Ampersand,
                start: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
                end: Position {
                    line: lexer.line,
                    startcol: lexer.col,
                    endcol: lexer.col + 1,
                },
            });
            advance(lexer);
        } else if !cur.is_whitespace() {
            tokens.push(make_identifier(lexer, kwds));
        } else {
            advance(lexer);
        }
    }

    tokens.push(Token {
        data: String::from("\\0"),
        tp: TokenType::Eof,
        start: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col + 1,
        },
        end: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col + 1,
        },
    });

    (tokens.len(), tokens)
}

fn make_number(lexer: &mut Lexer) -> Token {
    let start_col = lexer.col;
    let mut data: String = String::from("");

    let mut tp: TokenType = TokenType::I32;

    let start = Position {
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col + 1,
    };

    while lexer.current.is_numeric() || lexer.current == '_' {
        data.push(lexer.current);
        advance(lexer);
        if lexer.current == 'i' {
            advance(lexer);
            let mut specified_type = String::new();
            while lexer.current.is_numeric() {
                specified_type.push(lexer.current);
                advance(lexer);
            }
            match specified_type.as_str() {
                "8" => {
                    tp = TokenType::I8;
                }
                "16" => {
                    tp = TokenType::I16;
                }
                "32" => {
                    tp = TokenType::I32;
                }
                "64" => {
                    tp = TokenType::I64;
                }
                "128" => {
                    tp = TokenType::I128;
                }
                _ => {
                    raise_error(&format!("Invalid specified type i{}.", specified_type), ErrorType::InvalidSpecifiedNumericType, &Position {
                        line: lexer.line,
                        startcol: start_col,
                        endcol: lexer.col,
                    }, &lexer.info);
                }
            }
        }
    }

    Token {
        data,
        tp,
        start,
        end: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col,
        },
    }
}

fn make_identifier(lexer: &mut Lexer, kwds: &[String]) -> Token {
    let mut data: String = String::from("");

    let start = Position {
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col + 1,
    };

    while is_identi(lexer.current) && lexer.current != '\0' {
        data.push(lexer.current);
        advance(lexer);
    }

    let tp = if kwds.contains(&data) {
        TokenType::Keyword
    } else {
        TokenType::Identifier
    };

    Token {
        data,
        tp,
        start,
        end: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col,
        },
    }
}
