use std::{fmt::Display, fs::File, io::Write};

use crate::{
    errors::{raise_error, ErrorType},
    parser::nodes::{Node, NodeType},
    utils::{FileInfo, Position}, types::{BuiltinTypes, BasicType, Type},
};

mod check;

pub struct Mir<'a> {
    pub info: FileInfo<'a>,
    instructions: Vec<MirInstruction<'a>>,
    builtins: BuiltinTypes<'a>,
}

#[derive(Clone)]
pub enum RawMirInstruction {
    I32(String),
    Add { left: usize, right: usize },
    Declare(String),
    Store { name: String, right: usize },
    Own(usize),
    Load(String),
}

#[derive(Clone)]
pub struct MirInstruction<'a> {
    instruction: RawMirInstruction,
    pos: Position,
    tp: Type<'a>,
}

type MirResult<'a> = (usize, Type<'a>);

impl Display for RawMirInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawMirInstruction::Add { left, right } => {
                write!(f, "add {left} {right}")
            }
            RawMirInstruction::Declare(name) => {
                write!(f, "declare {name}")
            }
            RawMirInstruction::I32(value) => {
                write!(f, "i32 {value}")
            }
            RawMirInstruction::Load(name) => {
                write!(f, "load {name}")
            }
            RawMirInstruction::Own(result) => {
                write!(f, "own {result}")
            }
            RawMirInstruction::Store { name, right } => {
                write!(f, "store {name} {right}")
            }
        }
    }
}

pub fn new<'a>(info: FileInfo<'a>, builtins: BuiltinTypes<'a>) -> Mir<'a> {
    Mir {
        info,
        instructions: Vec::new(),
        builtins,
    }
}

pub fn check(instructions: Vec<MirInstruction>, info: FileInfo<'_>) {
    check::check(instructions, info)
}

impl<'a> Mir<'a> {
    pub fn generate(&mut self, ast: &Vec<Node>) -> Vec<MirInstruction<'a>> {
        for node in ast {
            self.generate_expr(node);
        }

        let mut out = String::new();
        for (i, instruction) in self.instructions.iter().enumerate() {
            out.push_str(&format!("{:<5}", format!("{}:", i)));
            out.push_str(&instruction.instruction.to_string());
            out.push_str(&format!(" -> {}\n", instruction.tp.qualname));
        }
        let mut f = File::create("a.mir").expect("Unable to create MIR output file.");
        f.write_all(out.as_bytes()).expect("Unable to write MIR.");

        self.instructions.clone()
    }

    fn generate_expr(&mut self, node: &Node) -> MirResult<'a> {
        match node.tp {
            NodeType::I32 => self.generate_i32(node),
            NodeType::Binary => self.generate_binary(node),
            NodeType::Let => self.generate_let(node),
            NodeType::Identifier => self.generate_load(node),
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
            tp: self.builtins.get(&BasicType::I32).unwrap().clone(),
        });
        
        (self.instructions.len() - 1, self.builtins.get(&BasicType::I32).unwrap().clone())
    }

    fn generate_binary(&mut self, node: &Node) -> MirResult<'a> {
        let binary = node.data.get_data();
        let left = self.generate_expr(binary.nodes.get("left").unwrap());
        let right = self.generate_expr(binary.nodes.get("right").unwrap());

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Add { left: left.0, right: right.0 },
            pos: node.pos.clone(),
            tp: self.builtins.get(&BasicType::I32).unwrap().clone(),
        });
        
        (self.instructions.len() - 1, self.builtins.get(&BasicType::I32).unwrap().clone())
    }

    fn generate_let(&mut self, node: &Node) -> MirResult<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.generate_expr(letnode.nodes.get("expr").unwrap());

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Declare(name.to_string()),
            pos: node.pos.clone(),
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(right.0),
            pos: node.pos.clone(),
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: name.to_string(),
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        });
        
        (self.instructions.len() - 1, self.builtins.get(&BasicType::I32).unwrap().clone())
    }

    fn generate_load(&mut self, node: &Node) -> MirResult<'a> {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Load(name.to_string()),
            pos: node.pos.clone(),
            tp: self.builtins.get(&BasicType::I32).unwrap().clone(),
        });
        
        (self.instructions.len() - 1, self.builtins.get(&BasicType::I32).unwrap().clone())
    }
}
