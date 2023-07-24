use std::collections::HashMap;

use crate::{
    codegen::CodeGen,
    types::{BasicType, Type},
};

pub fn init_void(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::I32,
        traits: HashMap::new(),
    };
    codegen.builtins.insert(BasicType::Void, tp);
}
