use std::{collections::HashMap, fmt::Display, fs::OpenOptions, io::Write};

use indexmap::IndexMap;

use crate::{
    codegen::{BindingTags, CodegenFunctions},
    errors::{raise_error, ErrorType},
    parser::nodes::{Node, NodeType, OpType},
    types::{BasicType, BuiltinTypes, Lifetime, Trait, TraitType, Type},
    utils::{FileInfo, Position},
};

mod check;

#[allow(dead_code)]
pub struct Mir<'a> {
    pub info: FileInfo<'a>,
    fn_name: String,
    fn_pos: Position,
    instructions: Vec<MirInstruction<'a>>,
    builtins: BuiltinTypes<'a>,
    functions: CodegenFunctions<'a>,
    namespace: HashMap<String, (Type<'a>, BindingTags)>,
}

#[derive(Clone)]
pub enum RawMirInstruction {
    I8(String),
    I16(String),
    I32(String),
    I64(String),
    I128(String),
    U8(String),
    U16(String),
    U32(String),
    U64(String),
    U128(String),
    Add { left: usize, right: usize },
    Declare { name: String, is_mut: bool },
    Store { name: String, right: usize },
    Own(usize),
    Load(String),
    Reference(usize),
    Copy(usize),
    DropBinding(String, usize),
    Bool(bool),
    Return(usize),
    CallFunction(String),
    Eq { left: usize, right: usize },
    Ne { left: usize, right: usize },
}

#[derive(Clone)]
pub struct MirInstruction<'a> {
    instruction: RawMirInstruction,
    pos: Position,
    tp: Option<Type<'a>>,
}

type MirResult<'a> = (usize, Type<'a>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum ReferenceType {
    Immutable,
}

#[derive(Debug)]
pub struct MirTag {
    is_owned: bool,
    is_mut: bool,
    owner: Option<usize>,
    lifetime: Lifetime,
}

type MirNamespace = HashMap<String, (Option<usize>, Option<usize>, MirTag)>; //(declaration, right, tag)
type MirReference = (usize, ReferenceType, Lifetime, ReferenceBase); //(right, type, lifetime, referred)

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReferenceBase {
    I32(Lifetime),
    Load {
        borrowed_life: Lifetime,
        value_life: Lifetime,
    },
    Reference(Lifetime),
}

impl Display for RawMirInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawMirInstruction::Add { left, right } => {
                write!(f, "add .{left} .{right}")
            }
            RawMirInstruction::Declare { name, is_mut } => {
                write!(f, "declare {}{}", if *is_mut { "mut " } else { "" }, name)
            }
            RawMirInstruction::I8(value) => {
                write!(f, "i8 {value}")
            }
            RawMirInstruction::I16(value) => {
                write!(f, "i16 {value}")
            }
            RawMirInstruction::I32(value) => {
                write!(f, "i32 {value}")
            }
            RawMirInstruction::I64(value) => {
                write!(f, "i64 {value}")
            }
            RawMirInstruction::I128(value) => {
                write!(f, "i128 {value}")
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
            RawMirInstruction::Bool(value) => {
                write!(f, "bool {value}")
            }
            RawMirInstruction::U8(value) => {
                write!(f, "u8 {value}")
            }
            RawMirInstruction::U16(value) => {
                write!(f, "u16 {value}")
            }
            RawMirInstruction::U32(value) => {
                write!(f, "u32 {value}")
            }
            RawMirInstruction::U64(value) => {
                write!(f, "u64 {value}")
            }
            RawMirInstruction::U128(value) => {
                write!(f, "u128 {value}")
            }
            RawMirInstruction::Return(right) => {
                write!(f, "return .{right}")
            }
            RawMirInstruction::CallFunction(name) => {
                write!(f, "call fn {name}")
            }
            RawMirInstruction::Eq { left, right } => {
                write!(f, "eq .{left} .{right}")
            }
            RawMirInstruction::Ne { left, right } => {
                write!(f, "ne .{left} .{right}")
            }
        }
    }
}

pub fn new<'a>(
    info: FileInfo<'a>,
    builtins: BuiltinTypes<'a>,
    functions: CodegenFunctions<'a>,
    fn_name: String,
    fn_pos: Position,
) -> Mir<'a> {
    Mir {
        info,
        fn_name,
        fn_pos,
        instructions: Vec::new(),
        builtins,
        functions,
        namespace: HashMap::new(),
    }
}

pub fn check(this: &mut Mir, instructions: &mut Vec<MirInstruction>) {
    let (mut namespace, references, bindings_drop) = check::generate_lifetimes(this, instructions);
    check::check_references(this, instructions, &mut namespace, &references);
    check::check_return(this, instructions);
    write_mir(this, bindings_drop, instructions.clone(), &mut namespace);
}

pub fn write_mir<'a>(
    this: &mut Mir,
    binding_drops: IndexMap<usize, MirInstruction<'a>>,
    mut instructions: Vec<MirInstruction<'a>>,
    namespace: &mut MirNamespace,
) {
    for (k, v) in binding_drops {
        instructions.insert(k, v);
    }

    let mut out = String::new();

    out.push_str(&format!("fn {} {{\n", this.fn_name));
    for (i, instruction) in instructions.iter().enumerate() {
        out.push_str("    ");
        out.push_str(&format!(".{:<5}", format!("{}:", i)));
        out.push_str(&instruction.instruction.to_string());
        if let RawMirInstruction::Declare { name, is_mut: _ } = &instruction.instruction {
            out.push_str(&namespace.get(name).unwrap().2.lifetime.to_string());
        }
        if let RawMirInstruction::DropBinding(_, _) = &instruction.instruction {
        } else if instruction.tp.is_some() {
            out.push_str(&format!(
                " -> {}",
                instruction.tp.as_ref().unwrap().qualname()
            ));
            out.push_str(&format!("{}", instruction.tp.as_ref().unwrap().lifetime));
        }
        out.push('\n');
    }
    out.push('}');

    let mut f = OpenOptions::new()
        .write(true)
        .append(true)
        .open("a.mir")
        .expect("Unable to open MIR output file.");

    if f.metadata().unwrap().len() > 0 {
        f.write_all("\n\n".as_bytes())
            .expect("Unable to write MIR.");
    }
    f.write_all(out.as_bytes()).expect("Unable to write MIR.");
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
            NodeType::I8 => self.generate_i8(node),
            NodeType::I16 => self.generate_i16(node),
            NodeType::I32 => self.generate_i32(node),
            NodeType::I64 => self.generate_i64(node),
            NodeType::I128 => self.generate_i128(node),
            NodeType::Binary => self.generate_binary(node),
            NodeType::Let => self.generate_let(node),
            NodeType::Identifier => self.generate_load(node),
            NodeType::Store => self.generate_store(node),
            NodeType::Reference => self.generate_reference(node),
            NodeType::Bool => self.generate_bool(node),
            NodeType::U8 => self.generate_u8(node),
            NodeType::U16 => self.generate_u16(node),
            NodeType::U32 => self.generate_u32(node),
            NodeType::U64 => self.generate_u64(node),
            NodeType::U128 => self.generate_u128(node),
            NodeType::Return => self.generate_return(node),
            NodeType::Fn => unreachable!(),
            NodeType::Call => self.generate_call(node),
        }
    }
}

impl<'a> Mir<'a> {
    fn generate_i8(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i8>()
            .is_err()
        {
            let fmt: String = format!(
                "i8 literal in radix 10 out of bounds ({} to {}).",
                i8::MAX,
                i8::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::I8(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::I8).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I8).unwrap().clone(),
        )
    }

    fn generate_i16(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i16>()
            .is_err()
        {
            let fmt: String = format!(
                "i16 literal in radix 10 out of bounds ({} to {}).",
                i16::MAX,
                i16::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::I16(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::I16).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I16).unwrap().clone(),
        )
    }

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

    fn generate_i64(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i64>()
            .is_err()
        {
            let fmt: String = format!(
                "i64 literal in radix 10 out of bounds ({} to {}).",
                i64::MAX,
                i64::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::I64(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::I64).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I64).unwrap().clone(),
        )
    }

    fn generate_i128(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i128>()
            .is_err()
        {
            let fmt: String = format!(
                "i128 literal in radix 10 out of bounds ({} to {}).",
                i128::MAX,
                i128::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::I128(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::I128).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::I128).unwrap().clone(),
        )
    }

    fn generate_u8(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u8>()
            .is_err()
        {
            let fmt: String = format!(
                "u8 literal in radix 10 out of bounds ({} to {}).",
                u8::MAX,
                u8::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::U8(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::U8).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::U8).unwrap().clone(),
        )
    }

    fn generate_u16(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u16>()
            .is_err()
        {
            let fmt: String = format!(
                "u16 literal in radix 10 out of bounds ({} to {}).",
                u16::MAX,
                u16::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::U16(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::U16).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::U16).unwrap().clone(),
        )
    }

    fn generate_u32(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u32>()
            .is_err()
        {
            let fmt: String = format!(
                "u32 literal in radix 10 out of bounds ({} to {}).",
                u32::MAX,
                u32::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::U32(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::U32).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::U32).unwrap().clone(),
        )
    }

    fn generate_u64(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u64>()
            .is_err()
        {
            let fmt: String = format!(
                "u64 literal in radix 10 out of bounds ({} to {}).",
                u64::MAX,
                u64::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::U64(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::U64).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::U64).unwrap().clone(),
        )
    }

    fn generate_u128(&mut self, node: &Node) -> MirResult<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u128>()
            .is_err()
        {
            let fmt: String = format!(
                "u128 literal in radix 10 out of bounds ({} to {}).",
                u128::MAX,
                u128::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::U128(
                node.data.get_data().raw.get("value").unwrap().to_string(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::U128).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::U128).unwrap().clone(),
        )
    }

    fn generate_bool(&mut self, node: &Node) -> MirResult<'a> {
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Bool(
                *node.data.get_data().booleans.get("value").unwrap(),
            ),
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::Bool).unwrap().clone()),
        });

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::Bool).unwrap().clone(),
        )
    }

    fn generate_binary(&mut self, node: &Node) -> MirResult<'a> {
        let binary = node.data.get_data();
        let left = self.generate_expr(binary.nodes.get("left").unwrap());
        let right = self.generate_expr(binary.nodes.get("right").unwrap());

        let (traittp, name) = match binary.op.unwrap() {
            OpType::Add => {
                (TraitType::Add, "Add")
            }
            OpType::Eq => {
                (TraitType::Eq, "Eq")
            }
            OpType::Ne => {
                (TraitType::Ne, "Ne")
            }
        };

        let t = left.1.traits.get(&traittp);
        
        let res = if let Some(Trait::Add { code: _, skeleton }) = t
        {
            skeleton(self, &node.pos, left.1, right.1)
        }
        else if let Some(Trait::Eq { code: _, skeleton }) = t
        {
            skeleton(self, &node.pos, left.1, right.1)
        }
        else if let Some(Trait::Ne { code: _, skeleton }) = t
        {
            skeleton(self, &node.pos, left.1, right.1)
        } else {
            raise_error(
                &format!("Type '{}' does not implement '{name}'.", left.1.qualname()),
                ErrorType::TraitNotImplemented,
                &node.pos,
                &self.info,
            );
        };

        let instruction = match traittp {
            TraitType::Add => {
                RawMirInstruction::Add {
                    left: left.0,
                    right: right.0,
                }
            }
            TraitType::Eq => {
                RawMirInstruction::Eq {
                    left: left.0,
                    right: right.0,
                }
            }
            TraitType::Ne => {
                RawMirInstruction::Ne {
                    left: left.0,
                    right: right.0,
                }
            }
            _ => {
                unreachable!();
            }
        };

        self.instructions.push(MirInstruction {
            instruction,
            pos: node.pos.clone(),
            tp: Some(res.clone()),
        });

        (self.instructions.len() - 1, res)
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

    fn generate_return(&mut self, node: &Node) -> MirResult<'a> {
        let returnnode = node.data.get_data();
        let mut expr = self.generate_expr(returnnode.nodes.get("expr").unwrap());

        expr.1.ref_n += 1;

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(expr.0),
            pos: node.pos.clone(),
            tp: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Return(expr.0),
            pos: node.pos.clone(),
            tp: Some(expr.1.clone()),
        });

        (self.instructions.len() - 1, expr.1.clone())
    }

    fn generate_call(&mut self, node: &Node) -> MirResult<'a> {
        let callnode = node.data.get_data();
        let name = callnode.raw.get("name").unwrap().clone();

        let func = self.functions.get(&name);

        match func {
            Some(func) => {
                self.instructions.push(MirInstruction {
                    instruction: RawMirInstruction::CallFunction(name),
                    pos: node.pos.clone(),
                    tp: Some(func.1 .1.clone()),
                });
            }
            None => {
                let fmt: String = format!("Function '{}' not found.", name);
                raise_error(&fmt, ErrorType::FunctionNotFound, &node.pos, &self.info);
            }
        }

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::Void).unwrap().clone(),
        )
    }
}
