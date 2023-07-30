use std::collections::HashMap;

use crate::{
    codegen::CodeGen,
    types::{BasicType, Type},
};

pub fn init_void(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::Void,
        traits: HashMap::new(),
        qualname: "std::void".into(),
    };
    codegen.builtins.insert(BasicType::Void, tp);
}
