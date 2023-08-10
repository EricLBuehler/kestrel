use std::str::Chars;

use inkwell::{module::Linkage, values::BasicValue, AddressSpace};

use crate::codegen::CodeGen;

#[derive(Clone, Debug)]
pub struct FileInfo<'a> {
    pub data: Chars<'a>,
    pub name: String,
    pub dir: String,
}

#[derive(Clone, Debug)]
pub struct Position {
    pub line: usize,
    pub startcol: usize, //Inclusive
    pub endcol: usize,   //Exclusive
    pub opcol: Option<usize>,
}

pub fn print_string(codegen: &CodeGen, message: &str) {
    let str = codegen.context.const_string(message.as_bytes(), true);

    let global = codegen
        .module
        .add_global(str.get_type(), Some(AddressSpace::from(0u16)), "");
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
