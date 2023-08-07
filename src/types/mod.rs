use std::{collections::HashMap, fmt::Display};

use inkwell::AddressSpace;

use crate::{
    codegen::{CodeGen, Data},
    mir::Mir,
    utils::Position,
};

pub mod builtins;

pub type BuiltinTypes<'a> = HashMap<BasicType, Type<'a>>;
pub type Traits<'a> = HashMap<TraitType, Trait<'a>>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Trait<'a> {
    Add {
        code: fn(&mut CodeGen<'a>, &Position, Data<'a>, Data<'a>) -> Data<'a>,
        skeleton: fn(&mut Mir, &Position, Type<'a>, Type<'a>) -> Type<'a>,
    },
    Copy,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum TraitType {
    Add,
    Copy,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum BasicType {
    I8,
    I16,
    I32,
    I64,
    I128,
    Void,
    Bool,
}

impl Display for BasicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BasicType::I8 => {
                write!(f, "i8")
            }
            BasicType::I16 => {
                write!(f, "i16")
            }
            BasicType::I32 => {
                write!(f, "i32")
            }
            BasicType::I64 => {
                write!(f, "i64")
            }
            BasicType::I128 => {
                write!(f, "i128")
            }
            BasicType::Void => {
                write!(f, "void")
            }
            BasicType::Bool => {
                write!(f, "bool")
            }
        }
    }
}

#[derive(Eq, Clone, Debug)]
pub struct Type<'a> {
    pub basictype: BasicType,
    pub traits: Traits<'a>,
    qualname: String,
    pub lifetime: Lifetime,
    pub ref_n: usize,
}

impl<'a> Type<'a> {
    pub fn qualname(&self) -> String {
        if self.ref_n > 0 {
            "&".to_string().repeat(self.ref_n) + &self.qualname
        } else {
            self.qualname.clone()
        }
    }
}

impl<'a> PartialEq for Type<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.basictype == other.basictype
            && self.traits == other.traits
            && self.qualname == other.qualname
            && self.ref_n == other.ref_n
    }
}

#[derive(PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub enum Lifetime {
    Static,
    ImplicitLifetime {
        name: String,
        start_mir: usize,
        end_mir: usize,
    },
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lifetime::Static => {
                write!(f, "['static]")
            }
            Lifetime::ImplicitLifetime {
                name,
                start_mir,
                end_mir,
            } => {
                write!(f, "['{} .{} => .{}]", name, start_mir, end_mir)
            }
        }
    }
}

pub fn init_extern_fns(codegen: &mut CodeGen) {
    let printftp = codegen.context.i32_type().fn_type(
        &[codegen
            .context
            .i8_type()
            .ptr_type(AddressSpace::from(0))
            .into()],
        false,
    );
    let printf =
        codegen
            .module
            .add_function("printf", printftp, Some(inkwell::module::Linkage::External));

    codegen.extern_fns.insert(String::from("printf"), printf);
}
