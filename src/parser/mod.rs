use crate::{
    errors::ErrorType,
    lexer::{Token, TokenType},
    utils::{FileInfo, Position},
};

pub mod nodes;
use self::nodes::{BinaryNode, DecimalNode, Node, OpType};

pub struct Parser<'a> {
    current: Token,
    info: FileInfo<'a>,
    tokens: Vec<Token>,
    idx: usize,
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Precedence {
    Lowest,
    Attr,
    To,
    Assign,
    LogicalOr,
    LogicalAnd,
    LogicalNot,
    Equals,
    Comparison,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    BitwiseShift,
    Sum,
    Product,
    BitwiseNot,
    Exp,
    Call,
    Index,
    Unary,
    Ternary,
    Max,
}

//Rules:
//Atomic: inplace
//Expressions + Keywords: leave off on next

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, info: &FileInfo<'a>) -> Self {
        Self {
            current: tokens.first().unwrap().clone(),
            info: info.clone(),
            tokens,
            idx: 1,
        }
    }

    pub fn generate_ast(&mut self) -> Vec<Node> {
        self.block()
    }

    #[allow(clippy::type_complexity)]
    fn block(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();

        while !self.current_is_type(TokenType::Eof) {
            nodes.push(self.parse_statement());
            self.skip_newlines();
        }

        nodes
    }

    fn skip_newlines(&mut self) {
        while self.current_is_type(TokenType::Newline) {
            self.advance();
        }
    }

    fn parse_statement(&mut self) -> Node {
        self.expr(Precedence::Lowest)
    }

    // =====================

    fn current_is_type(&mut self, tp: TokenType) -> bool {
        self.current.tp == tp
    }

    fn raise_error(&mut self, error: &str, errtp: ErrorType) -> ! {
        crate::errors::raise_error(
            error,
            errtp,
            &Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol - 1,
                line: self.current.start.line,
            },
            &self.info,
        );
    }

    fn advance(&mut self) {
        let next = self.tokens.get(self.idx);
        self.idx += 1;

        match next {
            Some(v) => {
                self.current = v.to_owned();
            }
            None => {
                self.current = Token {
                    data: String::from("\0"),
                    tp: TokenType::Eof,
                    start: Position {
                        line: 0,
                        startcol: 0,
                        endcol: 0,
                    },
                    end: Position {
                        line: 0,
                        startcol: 0,
                        endcol: 0,
                    },
                };
            }
        }
    }

    fn get_precedence(&self) -> Precedence {
        match self.current.tp {
            TokenType::Plus => Precedence::Sum,

            _ => Precedence::Lowest,
        }
    }

    // =======================

    fn atom(&mut self) -> Option<Node> {
        match self.current.tp {
            TokenType::I32 => Some(self.generate_i32()),
            _ => None,
        }
    }

    fn is_atomic(&self) -> bool {
        self.current.tp == TokenType::I32
    }

    fn expr(&mut self, prec: Precedence) -> Node {
        let mut left: Node;
        match self.atom() {
            None => self.raise_error("Invalid token.", ErrorType::InvalidTok),
            Some(val) => left = val,
        }

        self.advance();
        while !self.current_is_type(TokenType::Eof)
            && (prec as u32) < (self.get_precedence() as u32)
        {
            match self.current.tp {
                TokenType::Plus => left = self.generate_binary(left, self.get_precedence()),
                _ => {
                    break;
                }
            }
        }

        if self.is_atomic() {
            self.raise_error("Unexpected token.", ErrorType::InvalidTok);
        }
        left
    }

    // ============ Atomics ============
    fn generate_i32(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol - 1,
                line: self.current.start.line,
            },
            nodes::NodeType::I32,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    // ============ Expr ============
    fn generate_binary(&mut self, left: Node, prec: Precedence) -> Node {
        let op = match self.current.tp {
            TokenType::Plus => OpType::Add,
            _ => {
                unreachable!();
            }
        };

        self.advance();

        let right = self.expr(prec);

        Node::new(
            Position {
                startcol: left.pos.startcol,
                endcol: right.pos.endcol,
                line: left.pos.line,
            },
            nodes::NodeType::Binary,
            Box::new(BinaryNode { left, op, right }),
        )
    }
}
