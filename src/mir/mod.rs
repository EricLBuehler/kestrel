use std::{collections::HashMap, fmt::Display};

use crate::{
    codegen::BindingTags,
    errors::{raise_error, ErrorType},
    parser::nodes::{Node, NodeType, OpType},
    types::{BasicType, BuiltinTypes, Trait, TraitType, Type},
    utils::{FileInfo, Position},
};

mod check;

pub struct Mir<'a> {
    pub info: FileInfo<'a>,
    instructions: Vec<MirInstruction<'a>>,
    builtins: BuiltinTypes<'a>,
    namespace: HashMap<String, (Type<'a>, BindingTags)>,
}

#[derive(Clone)]
pub enum RawMirInstruction {
    I32(String),
    Add { left: usize, right: usize },
    Declare { name: String, is_mut: bool },
    Store { name: String, right: usize },
    Own(usize),
    Load(String),
    Reference(usize),
    Copy(usize),
    DropBinding(String, usize),
}

#[derive(Clone)]
pub struct MirInstruction<'a> {
    instruction: RawMirInstruction,
    pos: Position,
    tp: Option<Type<'a>>,
}

type MirResult<'a> = (usize, Type<'a>);

impl Display for RawMirInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawMirInstruction::Add { left, right } => {
                write!(f, "add .{left} .{right}")
            }
            RawMirInstruction::Declare { name, is_mut } => {
                write!(f, "declare {}{}", if *is_mut { "mut " } else { "" }, name)
            }
            RawMirInstruction::I32(value) => {
                write!(f, "i32 {value}")
            }
            RawMirInstruction::Load(name) => {
                write!(f, "load {name}")
            }
            RawMirInstruction::Own(result) => {
                write!(f, "own .{result}")
            }
            RawMirInstruction::Store { name, right } => {
                write!(f, "store {name} .{right}")
            }
            RawMirInstruction::Reference(right) => {
                write!(f, "ref .{right}")
            }
            RawMirInstruction::Copy(right) => {
                write!(f, "copy .{right}")
            }
            RawMirInstruction::DropBinding(name, _) => {
                write!(f, "dropbinding {name}")
            }
        }
    }
}

pub fn new<'a>(info: FileInfo<'a>, builtins: BuiltinTypes<'a>) -> Mir<'a> {
    Mir {
        info,
        instructions: Vec::new(),
        builtins,
        namespace: HashMap::new(),
    }
}

pub fn check(this: &mut Mir, instructions: &mut Vec<MirInstruction>) {
    let (mut namespace, references, bindings_drop) =
        check::generate_lifetimes(this, instructions);
    check::check(this, instructions, &mut namespace, &references);
    check::write_mir(bindings_drop, instructions.clone(), &mut namespace);
}

impl<'a> Mir<'a> {
    pub fn generate(&mut self, ast: &Vec<Node>) -> Vec<MirInstruction<'a>> {
        for node in ast {
            self.generate_expr(node);
        }

        self.instructions.clone()
    }

    fn generate_expr(&mut self, node: &Node) -> MirResult<'a> {
        match node.tp {
            NodeType::I32 => self.generate_i32(node),
            NodeType::Binary => self.generate_binary(node),
            NodeType::Let => self.generate_let(node),
            NodeType::Identifier => self.generate_load(node),
            NodeType::Store => self.generate_store(node),
            NodeType::Reference => self.generate_reference(node),
        }
    }
}

impl<'a> Mir<'a> {
    fn generate_i32(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i32>()
            .is_err()
        {
            let fmt: String = format!(
                "i32 literal in radix 10 out of bounds ({} to {}).",
                i32::MAX,
                i32::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::I32(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::I32).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I32).unwrap().clone(),
        )
    }

    fn generate_binary(&mut self, node: &Node) -> MirResult<'a> {
        let binary = node.data.get_data();
        let left = self.generate_expr(binary.nodes.get("left").unwrap());
        let right = self.generate_expr(binary.nodes.get("right").unwrap());

        let res = match binary.op.unwrap() {
            OpType::Add => {
                if let Some(Trait::Add { code: _, skeleton }) = left.1.traits.get(&TraitType::Add) {
                    skeleton(self, &node.pos, left.1, right.1)
                } else {
                    raise_error(
                        &format!("Type '{}' does not implement Add.", left.1.qualname()),
                        ErrorType::TypeMismatch,
                        &node.pos,
                        &self.info,
                    );
                }
            }
        };

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Add {
                left: left.0,
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: Some(res),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I32).unwrap().clone(),
        )
    }

    fn generate_let(&mut self, node: &Node) -> MirResult<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.generate_expr(letnode.nodes.get("expr").unwrap());
        let is_mut = letnode.booleans.get("is_mut").unwrap();

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Declare {
                name: name.to_string(),
                is_mut: *is_mut,
            },
            pos: node.pos.clone(),
            tp: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(right.0),
            pos: node.pos.clone(),
            tp: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: name.to_string(),
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::Void).unwrap().clone()),
        });

        self.namespace
            .insert(name.clone(), (right.1, BindingTags { is_mut: *is_mut }));

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::Void).unwrap().clone(),
        )
    }

    fn generate_load(&mut self, node: &Node) -> MirResult<'a> {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        if self.namespace.get(name).is_none() {
            let fmt: String = format!("Binding '{}' not found in scope.", name);
            raise_error(&fmt, ErrorType::BindingNotFound, &node.pos, &self.info);
        }

        let tp = self.namespace.get(name).unwrap().0.clone();

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Load(name.to_string()),
            pos: node.pos.clone(),
            tp: Some(tp.clone()),
        });

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Copy(self.instructions.len() - 1),
            pos: node.pos.clone(),
            tp: Some(tp.clone()),
        });

        (
            self.instructions.len() - 1,
            self.namespace.get(name).unwrap().0.clone(),
        )
    }

    fn generate_store(&mut self, node: &Node) -> MirResult<'a> {
        let storenode = node.data.get_data();
        let name = storenode.raw.get("name").unwrap();
        let expr = storenode.nodes.get("expr").unwrap();
        let right = self.generate_expr(expr);

        if self.namespace.get(name).is_none() {
            let fmt: String = format!("Binding '{}' not found in scope.", name);
            raise_error(&fmt, ErrorType::BindingNotFound, &node.pos, &self.info);
        }

        let binding = self.namespace.get(name).unwrap();

        if right.1 != binding.0 {
            raise_error(
                &format!(
                    "Expected '{}', got '{}'",
                    binding.0.qualname(),
                    right.1.qualname()
                ),
                ErrorType::TypeMismatch,
                &expr.pos,
                &self.info,
            );
        }

        if !binding.1.is_mut {
            raise_error(
                &format!(
                    "Binding '{}' is not mutable, so it cannot be assigned to.",
                    name
                ),
                ErrorType::BindingNotMutable,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: name.to_string(),
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::Void).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.namespace.get(name).unwrap().0.clone(),
        )
    }

    fn generate_reference(&mut self, node: &Node) -> MirResult<'a> {
        let referencenode = node.data.get_data();
        let mut expr = self.generate_expr(referencenode.nodes.get("expr").unwrap());

        expr.1.ref_n += 1;

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Reference(expr.0),
            pos: node.pos.clone(),
            tp: Some(expr.1.clone()),
        });

        (self.instructions.len() - 1, expr.1.clone())
    }
}
