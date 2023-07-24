//Generate tokens from text

use std::str::Chars;

use crate::utils::{FileInfo, Position};

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType {
    I32,
    Plus,
    Newline,
    Eof,
    Equal,
    Identifier,
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
            TokenType::Newline => write!(f, "NEWLINE"),
            TokenType::Eof => write!(f, "EOF"),
            TokenType::Equal => write!(f, "equal"),
            TokenType::Identifier => write!(f, "identifier"),
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

    if next.is_none() {
        lexer.col += 1;
        lexer.current = '\0';
        return;
    }

    let next = next.unwrap();
    lexer.col += unicode_width::UnicodeWidthChar::width(lexer.current).unwrap();

    if lexer.current == '\n' || lexer.current == '\r' {
        lexer.line += 1;
        lexer.col = 0;
    }

    lexer.current = next;
}

#[allow(dead_code)]
pub fn print_tokens(len: usize, tokens: &Vec<Token>) {
    println!("\n\nGenerated tokens:\n========================");
    println!("Token list ({} tokens)", len);
    println!("------------------------");
    let mut idx: usize = 1;
    for tok in tokens {
        println!("{} | {} {}", idx, tok, tok.start.line);
        idx += 1;
    }
    println!("========================");
}

pub fn is_not_identi(cur: char) -> bool {
    if cur.is_ascii_digit() || cur == '+' || cur == '\n' || cur == '=' {
        false
    }
    else {
        true
    }
}

pub fn generate_tokens(lexer: &mut Lexer, _kwds: &[String]) -> (usize, Vec<Token>) {
    let mut tokens: Vec<Token> = Vec::new();

    while lexer.current != '\0' {
        let cur: char = lexer.current.into();

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
                data: String::from("\n"),
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
        } else {
            tokens.push(make_identifier(lexer));
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
    let mut data: String = String::from("");

    let tp: TokenType = TokenType::I32;

    let start = Position {
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col + 1,
    };

    while (lexer.current as char).is_numeric() || lexer.current == '_' {
        data.push(lexer.current as char);
        advance(lexer);
    }

    Token {
        data,
        tp,
        start,
        end: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col + 1,
        },
    }
}

fn make_identifier(lexer: &mut Lexer) -> Token {
    let mut data: String = String::from("");

    let tp: TokenType = TokenType::Identifier;

    let start = Position {
        line: lexer.line,
        startcol: lexer.col,
        endcol: lexer.col + 1,
    };

    while is_not_identi(lexer.current) {
        data.push(lexer.current as char);
        advance(lexer);
    }

    Token {
        data,
        tp,
        start,
        end: Position {
            line: lexer.line,
            startcol: lexer.col,
            endcol: lexer.col + 1,
        },
    }
}
