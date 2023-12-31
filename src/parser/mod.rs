use std::collections::HashMap;

use crate::{
    errors::{raise_error, ErrorType},
    lexer::{Token, TokenType},
    utils::{FileInfo, Position}, parser::nodes::EnumNode,
};

pub mod nodes;
use self::nodes::{
    BinaryNode, BoolNode, CallNode, ConditionalNode, DecimalNode, DerefNode, FnNode,
    IdentifierNode, LetNode, Node, NodeType, OpType, ReferenceNode, ReturnNode, StoreNode,
};

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

    fn block(&mut self) -> Vec<Node> {
        self.skip_newlines();
        let mut nodes = Vec::new();

        while !self.current_is_type(TokenType::Eof) && !self.current_is_type(TokenType::RCurly) {
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
        match self.current.tp {
            TokenType::Keyword => self.keyword(),
            _ => self.expr(Precedence::Lowest),
        }
    }

    // =====================

    fn current_is_type(&mut self, tp: TokenType) -> bool {
        self.current.tp == tp
    }

    #[allow(dead_code)]
    fn next_is_type(&mut self, tp: TokenType) -> bool {
        self.advance();
        let res = self.current.tp == tp;
        self.backadvance();
        res
    }

    fn current_is_keyword(&mut self, name: &str) -> bool {
        if !self.current_is_type(TokenType::Keyword) {
            return false;
        }
        self.current.data == name
    }

    fn if_kwd_expect_keyword(&mut self, name: &str) {
        if self.current_is_type(TokenType::Keyword) && self.current.data != name {
            self.raise_error(
                format!(
                    "Because there is a keyword here, '{}' was expected, but got '{}'.",
                    name, self.current.data
                )
                .as_str(),
                ErrorType::InvalidTok,
            )
        }
    }

    fn expect(&mut self, tp: TokenType) {
        if !self.current_is_type(tp.clone()) {
            self.raise_error(
                format!(
                    "Invalid or unexpected token (expected '{}', got '{}').",
                    tp, self.current.tp
                )
                .as_str(),
                ErrorType::InvalidTok,
            )
        }
    }

    fn raise_error(&mut self, error: &str, errtp: ErrorType) -> ! {
        crate::errors::raise_error(
            error,
            errtp,
            &Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
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
                        opcol: None,
                        endcol: 0,
                    },
                    end: Position {
                        line: 0,
                        startcol: 0,
                        opcol: None,
                        endcol: 0,
                    },
                };
            }
        }
    }

    fn backadvance(&mut self) {
        self.idx -= 1;
        let next = self.tokens.get(self.idx - 1);

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
                        opcol: None,
                        endcol: 0,
                    },
                    end: Position {
                        line: 0,
                        startcol: 0,
                        opcol: None,
                        endcol: 0,
                    },
                };
            }
        }
    }

    fn get_precedence(&self) -> Precedence {
        match self.current.tp {
            TokenType::Plus => Precedence::Sum,
            TokenType::Equal => Precedence::Assign,
            TokenType::DoubleEqual | TokenType::NotEqual => Precedence::Comparison,

            _ => Precedence::Lowest,
        }
    }

    // =======================

    fn keyword(&mut self) -> Node {
        match self.current.data.as_str() {
            "let" => self.generate_let(),
            "true" => {
                let res = self.generate_true();
                self.advance();
                res
            }
            "false" => {
                let res = self.generate_false();
                self.advance();
                res
            }
            "fn" => self.generate_fn(),
            "return" => self.generate_return(),
            "if" => self.generate_if(),
            "else" => self.raise_error("'else' is not allowed here", ErrorType::FloatingElse),
            "elif" => self.raise_error("'elif' is not allowed here", ErrorType::FloatingElif),
            "enum" => self.generate_enum(),
            _ => {
                unreachable!();
            }
        }
    }

    fn generate_let(&mut self) -> Node {
        let startcol = self.current.start.startcol;
        self.advance();

        let is_mut = self.current_is_keyword("mut");
        self.if_kwd_expect_keyword("mut");
        if is_mut {
            self.advance();
        }

        self.expect(TokenType::Identifier);

        let name = self.atom().unwrap();

        self.advance();

        self.expect(TokenType::Equal);

        self.advance();

        let expr = self.expr(Precedence::Lowest);

        Node::new(
            Position {
                startcol,
                endcol: expr.pos.endcol,
                opcol: None,
                line: name.pos.line,
            },
            nodes::NodeType::Let,
            Box::new(LetNode {
                name: name.data.get_data().raw.get("value").unwrap().clone(),
                expr,
                is_mut,
            }),
        )
    }

    fn generate_true(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.start.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::Bool,
            Box::new(BoolNode { value: true }),
        )
    }

    fn generate_false(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.start.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::Bool,
            Box::new(BoolNode { value: false }),
        )
    }

    fn generate_fn(&mut self) -> Node {
        let startcol = self.current.start.startcol;

        self.advance();

        self.expect(TokenType::Identifier);
        let name = self.current.data.clone();

        self.advance();

        let mut args = Vec::new();

        let endcol = self.current.end.endcol;
        let endline = self.current.end.line;

        self.expect(TokenType::LParen);

        self.advance();

        while self.current_is_type(TokenType::Identifier) {
            args.push(self.current.data.clone());
            if !self.current_is_type(TokenType::Comma) && self.current_is_type(TokenType::RParen) {
                break;
            }
            self.advance();
            self.skip_newlines();
        }

        self.expect(TokenType::RParen);

        self.advance();

        let mut tp = None;
        if self.current_is_type(TokenType::Colon) {
            self.expect(TokenType::Colon);
            self.advance();
            tp = Some(self.expr(Precedence::Lowest));
        }

        self.skip_newlines();

        self.expect(TokenType::LCurly);

        self.advance();
        self.skip_newlines();

        let code = self.block();

        self.expect(TokenType::RCurly);

        self.advance();
        self.skip_newlines();

        Node::new(
            Position {
                startcol,
                endcol,
                opcol: None,
                line: endline,
            },
            nodes::NodeType::Fn,
            Box::new(FnNode {
                name,
                args,
                code,
                rettp: tp,
            }),
        )
    }

    fn generate_return(&mut self) -> Node {
        let startcol = self.current.start.startcol;
        self.advance();
        let expr = self.expr(Precedence::Lowest);

        Node::new(
            Position {
                startcol,
                endcol: expr.pos.endcol,
                opcol: None,
                line: expr.pos.line,
            },
            nodes::NodeType::Return,
            Box::new(ReturnNode { expr }),
        )
    }

    fn generate_if(&mut self) -> Node {
        let startcol = self.current.start.startcol;

        self.advance();

        let expr = self.expr(Precedence::Lowest);

        self.skip_newlines();

        self.expect(TokenType::LCurly);

        let mut endcol = self.current.end.endcol;
        let mut endline = self.current.end.line;

        self.advance();
        self.skip_newlines();

        let code = self.block();

        self.expect(TokenType::RCurly);

        self.advance();
        self.skip_newlines();

        let mut exprs = vec![expr];
        let mut codes = vec![code];
        let mut positions = vec![Position {
            startcol,
            endcol,
            opcol: None,
            line: endline,
        }];

        while self.current_is_keyword("elif") {
            self.advance();

            let expr = self.expr(Precedence::Lowest);

            self.skip_newlines();

            self.expect(TokenType::LCurly);

            endcol = self.current.end.endcol;
            endline = self.current.end.line;

            self.advance();
            self.skip_newlines();

            let code = self.block();

            self.expect(TokenType::RCurly);

            self.advance();
            self.skip_newlines();

            codes.push(code);
            exprs.push(expr);
            positions.push(Position {
                startcol,
                endcol,
                opcol: None,
                line: endline,
            });
        }

        let elsecode = if self.current_is_keyword("else") {
            self.advance();

            self.skip_newlines();

            self.expect(TokenType::LCurly);

            endcol = self.current.end.endcol;
            endline = self.current.end.line;

            self.advance();
            self.skip_newlines();

            let code = self.block();

            self.expect(TokenType::RCurly);

            self.advance();
            self.skip_newlines();
            positions.push(Position {
                startcol,
                endcol,
                opcol: None,
                line: endline,
            });

            Some(code)
        } else {
            None
        };

        Node::new(
            Position {
                startcol,
                endcol,
                opcol: None,
                line: endline,
            },
            nodes::NodeType::Conditional,
            Box::new(ConditionalNode {
                exprs,
                codes,
                elsecode,
                positions,
            }),
        )
    }

    fn generate_enum(&mut self) -> Node {
        let startcol = self.current.start.startcol;

        self.advance();
        
        self.expect(TokenType::Identifier);
        let name = self.current.data.clone();
        self.advance();

        self.skip_newlines();

        self.expect(TokenType::LCurly);

        let endcol = self.current.end.endcol;
        let endline = self.current.end.line;

        self.advance();
        self.skip_newlines();

        let mut variants = HashMap::new();
        while self.current_is_type(TokenType::Identifier) {
            variants.insert(self.current.data.clone(), 
            Node::new(
                Position {
                    startcol: self.current.start.startcol,
                    endcol: self.current.end.endcol,
                    opcol: None,
                    line: self.current.start.line,
                },
                nodes::NodeType::Identifier,
                Box::new(IdentifierNode {
                    value: "void".into(),
                }),
            ));

            self.advance();
            self.skip_newlines();
            if self.current_is_type(TokenType::RCurly) {
                break;
            }
            self.expect(TokenType::Comma);
            self.advance();
            self.skip_newlines();
        }

        self.expect(TokenType::RCurly);

        self.advance();
        self.skip_newlines();

        Node::new(
            Position {
                startcol,
                endcol,
                opcol: None,
                line: endline,
            },
            nodes::NodeType::Enum,
            Box::new(EnumNode {
                name,
                variants
            }),
        )
    }

    // =======================

    fn atom(&mut self) -> Option<Node> {
        match self.current.tp {
            TokenType::I8 => Some(self.generate_i8()),
            TokenType::I16 => Some(self.generate_i16()),
            TokenType::I32 => Some(self.generate_i32()),
            TokenType::I64 => Some(self.generate_i64()),
            TokenType::I128 => Some(self.generate_i128()),
            TokenType::U8 => Some(self.generate_u8()),
            TokenType::U16 => Some(self.generate_u16()),
            TokenType::U32 => Some(self.generate_u32()),
            TokenType::U64 => Some(self.generate_u64()),
            TokenType::U128 => Some(self.generate_u128()),
            TokenType::Identifier => Some(self.generate_identifier()),
            TokenType::Ampersand => Some(self.generate_reference()),
            TokenType::Keyword => {
                let res = self.keyword();
                self.backadvance();
                Some(res)
            }
            TokenType::Asterisk => Some(self.generate_asterisk()),
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
                TokenType::Plus | TokenType::DoubleEqual | TokenType::NotEqual => {
                    left = self.generate_binary(left, self.get_precedence())
                }
                TokenType::Equal => left = self.generate_assign(left),
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
    fn generate_i8(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::I8,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_i16(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::I16,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_i32(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::I32,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_i64(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::I64,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_i128(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::I128,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_u8(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::U8,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_u16(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::U16,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_u32(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::U32,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_u64(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::U64,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_u128(&mut self) -> Node {
        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::U128,
            Box::new(DecimalNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_identifier(&mut self) -> Node {
        if self.next_is_type(TokenType::LParen) {
            let startcol = self.current.start.startcol;
            let line = self.current.start.line;

            let name = self.current.data.clone();

            self.advance();
            self.advance();
            let mut args = Vec::new();
            while !self.current_is_type(TokenType::RParen) {
                args.push(self.expr(Precedence::Lowest));
                if self.current_is_type(TokenType::RParen) {
                    continue;
                }
                self.expect(TokenType::Comma);
                self.advance();
            }
            self.expect(TokenType::RParen);
            let endcol = self.current.end.endcol;

            return Node::new(
                Position {
                    startcol,
                    endcol,
                    opcol: None,
                    line,
                },
                nodes::NodeType::Call,
                Box::new(CallNode { name, args }),
            );
        }

        Node::new(
            Position {
                startcol: self.current.start.startcol,
                endcol: self.current.end.endcol,
                opcol: None,
                line: self.current.start.line,
            },
            nodes::NodeType::Identifier,
            Box::new(IdentifierNode {
                value: self.current.data.clone(),
            }),
        )
    }

    fn generate_reference(&mut self) -> Node {
        let pos = self.current.start.clone();
        self.advance();
        let expr = self.expr(Precedence::Lowest);
        self.backadvance();
        Node::new(
            Position {
                startcol: pos.startcol,
                endcol: expr.pos.endcol,
                opcol: None,
                line: pos.line,
            },
            nodes::NodeType::Reference,
            Box::new(ReferenceNode { expr }),
        )
    }

    fn generate_asterisk(&mut self) -> Node {
        let pos = self.current.start.clone();
        self.advance();
        let expr = self.expr(Precedence::Lowest);
        self.backadvance();
        Node::new(
            Position {
                startcol: pos.startcol,
                endcol: expr.pos.endcol,
                opcol: None,
                line: pos.line,
            },
            nodes::NodeType::Deref,
            Box::new(DerefNode { expr }),
        )
    }

    // ============ Expr ============
    fn generate_binary(&mut self, left: Node, prec: Precedence) -> Node {
        let op = match self.current.tp {
            TokenType::Plus => OpType::Add,
            TokenType::DoubleEqual => OpType::Eq,
            TokenType::NotEqual => OpType::Ne,
            _ => {
                unreachable!();
            }
        };

        let opcol = self.current.start.startcol;

        self.advance();

        let right = self.expr(prec);

        Node::new(
            Position {
                startcol: left.pos.startcol,
                endcol: right.pos.endcol,
                opcol: Some(opcol),
                line: left.pos.line,
            },
            nodes::NodeType::Binary,
            Box::new(BinaryNode { left, op, right }),
        )
    }

    fn generate_assign(&mut self, left: Node) -> Node {
        self.advance();

        if left.tp != NodeType::Identifier {
            raise_error(
                "Expected identifier node.",
                ErrorType::InvalidTok,
                &left.pos,
                &self.info,
            )
        }

        let expr = self.expr(Precedence::Lowest);

        Node::new(
            Position {
                startcol: left.pos.startcol,
                endcol: expr.pos.endcol,
                opcol: None,
                line: left.pos.line,
            },
            nodes::NodeType::Store,
            Box::new(StoreNode {
                name: left.data.get_data().raw.get("value").unwrap().clone(),
                expr,
            }),
        )
    }
}
