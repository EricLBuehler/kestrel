use std::{collections::HashMap, fs::OpenOptions, io::Write};

use indexmap::IndexMap;

use crate::{
    codegen::{BindingTags, CodegenFunctions},
    errors::{raise_error, raise_error_multi, ErrorType},
    parser::nodes::{Node, NodeType, OpType},
    types::{implements_trait, BasicType, BuiltinTypes, Lifetime, Trait, TraitType, Type},
    utils::{FileInfo, Position},
};

use self::mirxplore::explore;

mod check;
mod mirxplore;

#[allow(dead_code)]
pub struct Mir<'a> {
    pub info: FileInfo<'a>,
    fn_name: String,
    fn_pos: Position,
    instructions: Vec<MirInstruction<'a>>,
    pub builtins: BuiltinTypes<'a>,
    functions: CodegenFunctions<'a>,
    debug_mir: bool,
    cur_block: usize,
    blocks: Vec<Block<'a>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Block<'a> {
    namespace_check: MirNamespace,
    parents: Vec<usize>,
    blockid: usize,
    namespace: HashMap<String, (Type<'a>, BindingTags)>,
    instructions: Option<Vec<MirInstruction<'a>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockName {
    name: String,
    blockid: usize,
}

#[derive(Clone, Debug)]
pub enum RawMirInstruction<'a> {
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
    Add {
        left: usize,
        right: usize,
    },
    Declare {
        name: BlockName,
        is_mut: bool,
    },
    Store {
        name: BlockName,
        right: usize,
    },
    Own(usize),
    Load(BlockName),
    Reference(usize),
    Copy(usize),
    Bool(bool),
    Return(usize),
    CallFunction(String),
    Eq {
        left: usize,
        right: usize,
    },
    Ne {
        left: usize,
        right: usize,
    },
    Deref(usize),
    IfCondition {
        code: Vec<MirInstruction<'a>>,
        check_n: usize,
        right: Option<usize>,
        offset: usize,
    },
}

#[derive(Clone, Debug)]
pub struct MirInstruction<'a> {
    instruction: RawMirInstruction<'a>,
    pos: Position,
    tp: Option<Type<'a>>,
    last_use: Option<String>,
}

type MirResult<'a> = (usize, Type<'a>);

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum ReferenceType {
    Immutable,
}

#[derive(Debug, Clone)]
pub struct MirTag {
    is_owned: bool,
    is_mut: bool,
    owner: Option<(usize, usize)>, //i, blockid
    lifetime: Lifetime,
}

type MirNamespace = HashMap<String, (Option<usize>, Option<usize>, MirTag)>; //(declaration, right, tag)
type MirReference = (usize, ReferenceType, Lifetime, ReferenceBase); //(right, type, lifetime, referred)

#[derive(Debug, Eq, PartialOrd, Ord, Clone)]
pub enum ReferenceBase {
    Literal(Lifetime),
    Load { name: BlockName },
    Reference(Lifetime),
}

impl PartialEq for ReferenceBase {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReferenceBase::Literal(life1), ReferenceBase::Literal(life2)) => life1 == life2,
            (ReferenceBase::Load { name: _ }, ReferenceBase::Load { name: _ }) => true,
            (ReferenceBase::Reference(life1), ReferenceBase::Reference(life2)) => life1 == life2,
            _ => false,
        }
    }
}

impl<'a> RawMirInstruction<'a> {
    fn fmt(&self, f: &mut String, blocks: Vec<Block>, info: &FileInfo) {
        f.push_str(&match self {
            RawMirInstruction::Add { left, right } => {
                format!("add .{left} .{right}")
            }
            RawMirInstruction::Declare { name, is_mut } => {
                format!("declare {}{}", if *is_mut { "mut " } else { "" }, name.name)
            }
            RawMirInstruction::I8(value) => {
                format!("i8 {value}")
            }
            RawMirInstruction::I16(value) => {
                format!("i16 {value}")
            }
            RawMirInstruction::I32(value) => {
                format!("i32 {value}")
            }
            RawMirInstruction::I64(value) => {
                format!("i64 {value}")
            }
            RawMirInstruction::I128(value) => {
                format!("i128 {value}")
            }
            RawMirInstruction::Load(name) => {
                format!("load {}", name.name)
            }
            RawMirInstruction::Own(result) => {
                format!("own .{result}")
            }
            RawMirInstruction::Store { name, right } => {
                format!("store {} .{right}", name.name)
            }
            RawMirInstruction::Reference(right) => {
                format!("ref .{right}")
            }
            RawMirInstruction::Copy(right) => {
                format!("copy .{right}")
            }
            RawMirInstruction::Bool(value) => {
                format!("bool {value}")
            }
            RawMirInstruction::U8(value) => {
                format!("u8 {value}")
            }
            RawMirInstruction::U16(value) => {
                format!("u16 {value}")
            }
            RawMirInstruction::U32(value) => {
                format!("u32 {value}")
            }
            RawMirInstruction::U64(value) => {
                format!("u64 {value}")
            }
            RawMirInstruction::U128(value) => {
                format!("u128 {value}")
            }
            RawMirInstruction::Return(right) => {
                format!("return .{right}")
            }
            RawMirInstruction::CallFunction(name) => {
                format!("call fn {name}")
            }
            RawMirInstruction::Eq { left, right } => {
                format!("eq .{left} .{right}")
            }
            RawMirInstruction::Ne { left, right } => {
                format!("ne .{left} .{right}")
            }
            RawMirInstruction::Deref(right) => {
                format!("deref .{right}")
            }
            RawMirInstruction::IfCondition {
                code,
                check_n,
                right,
                offset,
            } => {
                let mut out = String::new();
                output_mir(&code[*offset..], &mut out, &0, info, blocks);
                out = out
                    .split('\n')
                    .map(|x| String::from("    ") + x)
                    .collect::<Vec<String>>()
                    .join("\n");
                if right.is_some() {
                    format!("ifcondition #{check_n} .{} {{\n{out}}}", right.unwrap())
                } else {
                    format!("ifcondition #{check_n} {{\n{out}}}")
                }
            }
        })
    }
}

pub fn new<'a>(
    info: FileInfo<'a>,
    builtins: BuiltinTypes<'a>,
    functions: CodegenFunctions<'a>,
    fn_name: String,
    fn_pos: Position,
    debug_mir: bool,
) -> Mir<'a> {
    let cur = Block {
        namespace_check: HashMap::new(),
        namespace: HashMap::new(),
        parents: vec![0],
        blockid: 0,
        instructions: None,
    };
    Mir {
        info,
        fn_name,
        fn_pos,
        instructions: Vec::new(),
        builtins,
        functions,
        debug_mir,
        cur_block: 0,
        blocks: vec![cur],
    }
}

pub fn check<'a>(this: &mut Mir<'a>, instructions: &mut Vec<MirInstruction<'a>>, head: bool) {
    let references = check::generate_lifetimes(this, instructions);
    check::check_references(this, instructions, &references);
    check::check_return(this, instructions);
    if head {
        if !this.debug_mir {
            write_mir(
                this,
                instructions.clone(),
                this.blocks.first().unwrap().clone(),
                &references,
            );
        } else {
            explore(
                this,
                instructions,
                this.blocks.first().unwrap().clone(),
                references.clone(),
                this.info.clone(),
            );
        }
    }
}

pub fn output_mir(
    instructions: &[MirInstruction<'_>],
    out: &mut String,
    start: &usize,
    info: &FileInfo,
    blocks: Vec<Block>,
) {
    let mut cur_line = None;

    for (i, instruction) in instructions.iter().enumerate() {
        if Some(instruction.pos.line) != cur_line {
            cur_line = Some(instruction.pos.line);
            out.push_str("    ");
            out.push_str(&format!("{}:{}\n", info.name, instruction.pos.line + 1));
        }

        out.push_str("    ");
        out.push_str(&format!(".{:<5}", format!("{}:", i + start)));
        instruction.instruction.fmt(out, blocks.clone(), info);

        if let RawMirInstruction::Declare { name, is_mut: _ } = &instruction.instruction {
            out.push_str(
                &blocks
                    .get(name.blockid)
                    .unwrap()
                    .namespace_check
                    .get(&name.name)
                    .unwrap()
                    .2
                    .lifetime
                    .to_string(),
            );
        }

        if instruction.tp.is_some() {
            out.push_str(&format!(
                " -> {}",
                instruction.tp.as_ref().unwrap().qualname()
            ));
            out.push_str(&format!("{}", instruction.tp.as_ref().unwrap().lifetime));
        }

        if instruction.last_use.is_some() {
            out.push_str("  dropbinding ");
            out.push_str(instruction.last_use.as_ref().unwrap());
        }

        out.push('\n');
    }
}

pub fn write_mir(
    this: &mut Mir,
    instructions: Vec<MirInstruction<'_>>,
    _namespace: Block,
    references: &IndexMap<usize, MirReference>,
) {
    let mut out = String::new();

    out.push_str(&format!(
        "fn {}: {} {{\n",
        this.fn_name,
        this.functions.get(&this.fn_name).unwrap().1 .1.qualname()
    ));

    output_mir(&instructions, &mut out, &0, &this.info, this.blocks.clone());

    out.push('\n');

    for (i, (_right, _reftype, life, _)) in references {
        out.push_str("    ");
        out.push_str(&format!(
            "{} ref .{} {life}",
            "&".repeat(
                instructions
                    .get(*i)
                    .as_ref()
                    .unwrap()
                    .tp
                    .as_ref()
                    .unwrap()
                    .ref_n
            ),
            i
        ));
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
        let n = self.blocks.len() - 1;
        for node in ast {
            self.generate_expr(node);
        }
        self.blocks.get_mut(n).unwrap().instructions = Some(self.instructions.clone());

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
            NodeType::Deref => self.generate_deref(node),
            NodeType::Conditional => self.generate_if(node),
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            last_use: None,
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
            OpType::Add => (TraitType::Add, "Add"),
            OpType::Eq => (TraitType::Eq, "Eq"),
            OpType::Ne => (TraitType::Ne, "Ne"),
        };

        let t = left.1.traits.get(&traittp);

        let res = if let Some(Trait::Add {
            code: _,
            skeleton,
            ref_n: _,
        }) = t
        {
            skeleton(self, &node.pos, left.1, right.1)
        } else if let Some(Trait::Eq {
            code: _,
            skeleton,
            ref_n: _,
        }) = t
        {
            skeleton(self, &node.pos, left.1, right.1)
        } else if let Some(Trait::Ne {
            code: _,
            skeleton,
            ref_n: _,
        }) = t
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
            TraitType::Add => RawMirInstruction::Add {
                left: left.0,
                right: right.0,
            },
            TraitType::Eq => RawMirInstruction::Eq {
                left: left.0,
                right: right.0,
            },
            TraitType::Ne => RawMirInstruction::Ne {
                left: left.0,
                right: right.0,
            },
            _ => {
                unreachable!();
            }
        };

        self.instructions.push(MirInstruction {
            instruction,
            pos: node.pos.clone(),
            tp: Some(res.clone()),
            last_use: None,
        });

        (self.instructions.len() - 1, res)
    }

    fn generate_let(&mut self, node: &Node) -> MirResult<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let is_mut = letnode.booleans.get("is_mut").unwrap();

        let blockname = BlockName {
            name: name.clone(),
            blockid: self.cur_block,
        };

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Declare {
                name: blockname.clone(),
                is_mut: *is_mut,
            },
            pos: node.pos.clone(),
            tp: None,
            last_use: None,
        });

        let right = self.generate_expr(letnode.nodes.get("expr").unwrap());

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(right.0),
            pos: node.pos.clone(),
            tp: None,
            last_use: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: blockname.clone(),
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::Void).unwrap().clone()),
            last_use: None,
        });

        let mut get = self.blocks.get_mut(self.cur_block);
        let block = get.as_mut().unwrap();
        block
            .namespace
            .insert(name.clone(), (right.1, BindingTags { is_mut: *is_mut }));

        (
            self.instructions.len() - 1,
            self.builtins.get(&BasicType::Void).unwrap().clone(),
        )
    }

    fn generate_load(&mut self, node: &Node) -> MirResult<'a> {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        for blockid in self
            .blocks
            .get(self.cur_block)
            .unwrap()
            .parents
            .iter()
            .rev()
        {
            let block = self.blocks.get(*blockid).unwrap();
            if block.namespace.get(name).is_none() {
                continue;
            }

            let tp = block.namespace.get(name).unwrap().0.clone();

            let blockname = BlockName {
                name: name.clone(),
                blockid: block.blockid,
            };

            self.instructions.push(MirInstruction {
                instruction: RawMirInstruction::Load(blockname),
                pos: node.pos.clone(),
                tp: Some(tp.clone()),
                last_use: None,
            });

            if implements_trait(&tp, TraitType::Copy) {
                self.instructions.push(MirInstruction {
                    instruction: RawMirInstruction::Copy(self.instructions.len() - 1),
                    pos: node.pos.clone(),
                    tp: Some(tp.clone()),
                    last_use: None,
                });
            }

            return (
                self.instructions.len() - 1,
                block.namespace.get(name).unwrap().0.clone(),
            );
        }

        let fmt: String = format!("Binding '{}' not found in scope.", name);
        raise_error(&fmt, ErrorType::BindingNotFound, &node.pos, &self.info);
    }

    fn generate_store(&mut self, node: &Node) -> MirResult<'a> {
        let storenode = node.data.get_data();
        let name = storenode.raw.get("name").unwrap();
        let expr = storenode.nodes.get("expr").unwrap();
        let right = self.generate_expr(expr);

        let block = self.blocks.get(self.cur_block).unwrap();

        if block.namespace.get(name).is_none() {
            let fmt: String = format!("Binding '{}' not found in scope.", name);
            raise_error(&fmt, ErrorType::BindingNotFound, &node.pos, &self.info);
        }

        let binding = block.namespace.get(name).unwrap();

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

        let blockname = BlockName {
            name: name.clone(),
            blockid: self.cur_block,
        };

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(right.0),
            pos: node.pos.clone(),
            tp: None,
            last_use: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Store {
                name: blockname,
                right: right.0,
            },
            pos: node.pos.clone(),
            tp: Some(self.builtins.get(&BasicType::Void).unwrap().clone()),
            last_use: None,
        });

        (
            self.instructions.len() - 1,
            block.namespace.get(name).unwrap().0.clone(),
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
            last_use: None,
        });

        (self.instructions.len() - 1, expr.1.clone())
    }

    fn generate_return(&mut self, node: &Node) -> MirResult<'a> {
        let returnnode = node.data.get_data();
        let expr = self.generate_expr(returnnode.nodes.get("expr").unwrap());

        //TODO: Actual lifetime check
        if expr.1.ref_n != 0 {
            raise_error(
                "Cannot return reference.",
                ErrorType::ReturnReference,
                &node.pos,
                &self.info,
            );
        }

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Own(expr.0),
            pos: node.pos.clone(),
            tp: None,
            last_use: None,
        });
        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Return(expr.0),
            pos: node.pos.clone(),
            tp: Some(expr.1.clone()),
            last_use: None,
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
                    last_use: None,
                });
            }
            None => {
                let fmt: String = format!("Function '{}' not found.", name);
                raise_error(&fmt, ErrorType::FunctionNotFound, &node.pos, &self.info);
            }
        }

        (self.instructions.len() - 1, func.unwrap().1 .1.clone())
    }

    fn generate_deref(&mut self, node: &Node) -> MirResult<'a> {
        let derefnode = node.data.get_data();
        let mut expr = self.generate_expr(derefnode.nodes.get("expr").unwrap());

        if expr.1.ref_n == 0 {
            let fmt: String = format!("Cannot deref non-reference type '{}'.", expr.1.qualname());
            raise_error(&fmt, ErrorType::DerefNonref, &node.pos, &self.info);
        }

        expr.1.ref_n -= 1;

        self.instructions.push(MirInstruction {
            instruction: RawMirInstruction::Deref(expr.0),
            pos: node.pos.clone(),
            tp: Some(expr.1.clone()),
            last_use: None,
        });

        (self.instructions.len() - 1, expr.1.clone())
    }

    fn generate_if(&mut self, node: &Node) -> MirResult<'a> {
        let ifnode = node.data.get_data();
        let codes = ifnode.nodearr_codes.unwrap().clone();
        let exprs = ifnode.nodearr.unwrap();

        let mut finaltp: Option<(Type<'_>, Position)> = None;
        let mut check_n = 0;

        for (code, expr) in std::iter::zip(codes, exprs) {
            let expr = self.generate_expr(expr);

            if expr.1.basictype != BasicType::Bool {
                raise_error(
                    &format!("Expected 'std::bool', got '{}'", expr.1.qualname()),
                    ErrorType::TypeMismatch,
                    &node.pos,
                    &self.info,
                );
            }
            let block = self.blocks.get(self.cur_block).unwrap().clone();

            let mut parents = block.parents.clone();
            parents.push(self.blocks.len());
            let cur_block = Block {
                namespace_check: HashMap::new(),
                parents,
                blockid: self.blocks.len(),
                namespace: HashMap::new(),
                instructions: None,
            };

            self.blocks.push(cur_block.clone());

            let old_block = self.cur_block;
            self.cur_block = cur_block.blockid;

            let len = self.instructions.len();
            let instructions = self.generate(&code);

            self.cur_block = old_block;

            let tp_cur = instructions
                .iter()
                .map(|x| {
                    x.tp.as_ref()
                        .unwrap_or(self.builtins.get(&BasicType::Void).unwrap())
                        .clone()
                })
                .last()
                .unwrap_or(self.builtins.get(&BasicType::Void).unwrap().clone());

            let pos_cur = instructions
                .iter()
                .map(|x| x.pos.clone())
                .last()
                .unwrap_or(node.pos.clone());

            match finaltp {
                Some(ref tp) => {
                    if tp.0 != tp_cur {
                        raise_error_multi(
                            vec![
                                format!(
                                    "Expected '{}', got '{}'",
                                    tp.0.qualname(),
                                    tp_cur.qualname()
                                ),
                                format!("Original type:"),
                            ],
                            ErrorType::TypeMismatch,
                            vec![&pos_cur, &tp.1],
                            &self.info,
                        );
                    }
                }
                None => {
                    finaltp = Some((tp_cur.clone(), pos_cur));
                }
            }

            self.instructions.push(MirInstruction {
                instruction: RawMirInstruction::IfCondition {
                    code: instructions.clone(),
                    check_n,
                    right: Some(expr.0),
                    offset: len,
                },
                pos: node.pos.clone(),
                tp: Some(tp_cur),
                last_use: None,
            });
            check_n += 1;
        }

        if ifnode.nodearr_else.is_some() {
            let code = ifnode.nodearr_else.as_ref().unwrap().clone();

            let block = self.blocks.get(self.cur_block).unwrap().clone();

            let mut parents = block.parents.clone();
            parents.push(self.blocks.len());
            let cur_block = Block {
                namespace_check: HashMap::new(),
                parents,
                blockid: self.blocks.len(),
                namespace: HashMap::new(),
                instructions: None,
            };

            self.blocks.push(cur_block.clone());

            let old_block = self.cur_block;
            self.cur_block = cur_block.blockid;

            let len = self.instructions.len();
            let instructions = self.generate(&code);

            self.cur_block = old_block;

            let tp_cur = instructions
                .iter()
                .map(|x| {
                    x.tp.as_ref()
                        .unwrap_or(self.builtins.get(&BasicType::Void).unwrap())
                        .clone()
                })
                .last()
                .unwrap_or(self.builtins.get(&BasicType::Void).unwrap().clone());

            let pos_cur = instructions
                .iter()
                .map(|x| x.pos.clone())
                .last()
                .unwrap_or(node.pos.clone());

            match finaltp {
                Some(ref tp) => {
                    if tp.0 != tp_cur {
                        raise_error_multi(
                            vec![
                                format!(
                                    "Expected '{}', got '{}'",
                                    tp.0.qualname(),
                                    tp_cur.qualname()
                                ),
                                format!("Original type:"),
                            ],
                            ErrorType::TypeMismatch,
                            vec![&pos_cur, &tp.1],
                            &self.info,
                        );
                    }
                }
                None => {
                    finaltp = Some((tp_cur.clone(), pos_cur));
                }
            }

            self.instructions.push(MirInstruction {
                instruction: RawMirInstruction::IfCondition {
                    code: instructions.clone(),
                    check_n,
                    right: None,
                    offset: len,
                },
                pos: node.pos.clone(),
                tp: Some(tp_cur),
                last_use: None,
            });
        }

        (self.instructions.len() - 1, finaltp.unwrap().0)
    }
}
