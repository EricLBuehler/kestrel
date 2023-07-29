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

#[derive(Clone, Debug)]
pub enum MirInstruction {
    I32(String, Position),
    Add {
        left: usize,
        right: usize,
        pos: Position,
    },
    Declare(String, Position),
    Store {
        name: String,
        right: usize,
        pos: Position,
    },
    Own(usize, Position),
    Load(String, Position),
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

        self.instructions.push(MirInstruction::I32(
            node.data.get_data().raw.get("value").unwrap().to_string(),
            node.pos.clone(),
        ));
        self.instructions.len() - 1
    }

    fn generate_binary(&mut self, node: &Node) -> usize {
        let binary = node.data.get_data();
        let left = self.generate_expr(binary.nodes.get("left").unwrap());
        let right = self.generate_expr(binary.nodes.get("right").unwrap());

        self.instructions.push(MirInstruction::Add {
            left,
            right,
            pos: node.pos.clone(),
        });
        self.instructions.len() - 1
    }

    fn generate_let(&mut self, node: &Node) -> usize {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.generate_expr(letnode.nodes.get("expr").unwrap());

        self.instructions
            .push(MirInstruction::Declare(name.to_string(), node.pos.clone()));
        self.instructions
            .push(MirInstruction::Own(right, node.pos.clone()));
        self.instructions.push(MirInstruction::Store {
            name: name.to_string(),
            right,
            pos: node.pos.clone(),
        });
        self.instructions.len() - 1
    }

    fn generate_load(&mut self, node: &Node) -> usize {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        self.instructions
            .push(MirInstruction::Load(name.to_string(), node.pos.clone()));
        self.instructions.len() - 1
    }
}
