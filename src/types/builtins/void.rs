use std::collections::HashMap;

use crate::{
    codegen::CodeGen,
    types::{BasicType, Type, Lifetime},
};

pub fn init_void(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::Void,
        traits: HashMap::new(),
        qualname: "std::void".into(),
        lifetime: Lifetime::Static,
    };
    codegen.builtins.insert(BasicType::Void, tp);
}
