use crate::codegen::CodeGen;

use self::i32::init_i32;
use self::void::init_void;

mod i32;
mod void;

pub fn init_builtins(codegen: &mut CodeGen) {
    init_i32(codegen);
    init_void(codegen);
}
