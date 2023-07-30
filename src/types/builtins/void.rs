use std::collections::HashMap;

use crate::{
    codegen::CodeGen,
    types::{BasicType, Lifetime, Trait, TraitType, Type},
};

pub fn init_void(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::Void,
        traits: HashMap::from([(TraitType::Copy, Trait::Copy)]),
        qualname: "std::void".into(),
        lifetime: Lifetime::Static,
    };
    codegen.builtins.insert(BasicType::Void, tp);
}
