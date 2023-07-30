use std::{fmt::Display, fs::File, io::Write};

use crate::{
    errors::{raise_error, ErrorType},
    parser::nodes::{Node, NodeType},
    utils::{FileInfo, Position},
};

mod check;

pub struct Mir<'a> {
    pub info: FileInfo<'a>,
    instructions: Vec<MirInstruction>,
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
pub struct MirInstruction {
    instruction: RawMirInstruction,
    pos: Position,
}

impl Display for RawMirInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawMirInstruction::Add { left, right } => {
                writeln!(f, "add {left} {right}")
            }
            RawMirInstruction::Declare(name) => {
                writeln!(f, "declare {name}")
            }
            RawMirInstruction::I32(value) => {
                writeln!(f, "i32 {value}")
            }
            RawMirInstruction::Load(name) => {
                writeln!(f, "load {name}")
            }
            RawMirInstruction::Own(result) => {
                writeln!(f, "own {result}")
            }
            RawMirInstruction::Store { name, right } => {
                writeln!(f, "store {name} {right}")
            }
        }
    }
}

pub fn new(info: FileInfo<'_>) -> Mir<'_> {
    Mir {
        info,
        instructions: Vec::new(),
    }
}

pub fn check(instructions: Vec<MirInstruction>, info: FileInfo<'_>) {
    check::check(instructions, info)
}

impl<'a> Mir<'a> {
    pub fn generate(&mut self, ast: &Vec<Node>) -> Vec<MirInstruction> {
        for node in ast {
            self.generate_expr(node);
        }

        let mut out = String::new();
        for (i, instruction) in self.instructions.iter().enumerate() {
            out.push_str(&format!("{:<5}", format!("{}:", i)));
            out.push_str(&instruction.instruction.to_string());
        }
        let mut f = File::create("a.mir").expect("Unable to create MIR output file.");
        f.write_all(out.as_bytes()).expect("Unable to write MIR.");

        self.instructions.clone()
    }

    fn generate_expr(&mut self, node: &Node) -> usize {
        match node.tp {
            NodeType::I32 => self.generate_i32(node),
            NodeType::Binary => self.generate_binary(node),
            NodeType::Let => self.generate_let(node),
            NodeType::Identifier => self.generate_load(node),
        }
    }
}

impl<'a> Mir<'a> {
    fn generate_i32(&mut self, node: &Node) -> usize {
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
        });
        self.instructions.len() - 1
    }

    fn generate_binary(&mut self, node: &Node) -> usize {
        let binary = node.data.get_data();
        let left = self.generate_expr(binary.nodes.get("left").unwrap());
        let right = self.generate_expr(binary.nodes.get("right").unwrap());

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Add { left, right },
            pos: node.pos.clone(),
        });
        self.instructions.len() - 1
    }

    fn generate_let(&mut self, node: &Node) -> usize {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.generate_expr(letnode.nodes.get("expr").unwrap());

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Declare(name.to_string()),
            pos: node.pos.clone(),
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(right),
            pos: node.pos.clone(),
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: name.to_string(),
                right,
            },
            pos: node.pos.clone(),
        });
        self.instructions.len() - 1
    }

    fn generate_load(&mut self, node: &Node) -> usize {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Load(name.to_string()),
            pos: node.pos.clone(),
        });
        self.instructions.len() - 1
    }
}
