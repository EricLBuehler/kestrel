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

pub fn implements_trait(tp: &Type<'_>, trait_tp: TraitType) -> bool {
    let trait_opt = tp.traits.get(&trait_tp);
    trait_opt.is_some()
        && match trait_opt.unwrap() {
            Trait::Add {
                code: _,
                skeleton: _,
                ref_n,
            } => tp.ref_n == *ref_n,
            Trait::Eq {
                code: _,
                skeleton: _,
                ref_n,
            } => tp.ref_n == *ref_n,
            Trait::Ne {
                code: _,
                skeleton: _,
                ref_n,
            } => tp.ref_n == *ref_n,
            Trait::Copy { ref_n } => tp.ref_n == *ref_n,
        }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Trait<'a> {
    Add {
        code: fn(&mut CodeGen<'a>, &Position, Data<'a>, Data<'a>) -> Data<'a>,
        skeleton: fn(&mut Mir, &Position, Type<'a>, Type<'a>) -> Type<'a>,
        ref_n: usize,
    },
    Eq {
        code: fn(&mut CodeGen<'a>, &Position, Data<'a>, Data<'a>) -> Data<'a>,
        skeleton: fn(&mut Mir, &Position, Type<'a>, Type<'a>) -> Type<'a>,
        ref_n: usize,
    },
    Ne {
        code: fn(&mut CodeGen<'a>, &Position, Data<'a>, Data<'a>) -> Data<'a>,
        skeleton: fn(&mut Mir, &Position, Type<'a>, Type<'a>) -> Type<'a>,
        ref_n: usize,
    },
    Copy {
        ref_n: usize,
    },
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum TraitType {
    Add,
    Copy,
    Eq,
    Ne,
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
    U8,
    U16,
    U32,
    U64,
    U128,
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
            BasicType::U8 => {
                write!(f, "u8")
            }
            BasicType::U16 => {
                write!(f, "u16")
            }
            BasicType::U32 => {
                write!(f, "u32")
            }
            BasicType::U64 => {
                write!(f, "u64")
            }
            BasicType::U128 => {
                write!(f, "u128")
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
