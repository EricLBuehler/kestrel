use std::{collections::HashMap, fmt::Display};

use inkwell::{AddressSpace, module::Linkage, values::BasicValue};

use crate::{
    codegen::{CodeGen, Data},
    utils::Position,
};

pub mod builtins;

pub type BuiltinTypes<'a> = HashMap<BasicType, Type<'a>>;
pub type Traits<'a> = HashMap<TraitType, Trait<'a>>;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum Trait<'a> {
    Add(fn(&mut CodeGen<'a>, &Position, Data<'a>, Data<'a>) -> Data<'a>),
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum TraitType {
    Add,
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum BasicType {
    I32,
    Void,
}

impl Display for BasicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            BasicType::I32 => {
                write!(f, "i32")
            }
            BasicType::Void => {
                write!(f, "void")
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Type<'a> {
    pub basictype: BasicType,
    pub traits: Traits<'a>,
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

pub fn print_panic(codegen: &mut CodeGen, message: &str) {
    let str = codegen.context.const_string(&message.as_bytes()[..], true);

    let global = codegen.module.add_global(str.get_type(), Some(AddressSpace::from(0u16)), "");
    global.set_constant(true);
    global.set_linkage(Linkage::Private);
    global.set_initializer(&str.as_basic_value_enum());

    let ptr = unsafe {
        codegen.builder.build_gep(
            global.as_pointer_value(),
            &[
                codegen.context.i32_type().const_zero(),
                codegen.context.i32_type().const_zero(),
            ],
            "",
        )
    };

    codegen.builder.build_call(
        *codegen.extern_fns.get("printf").unwrap(),
        &[ptr.into()],
        "",
    );
}
