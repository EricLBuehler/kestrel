use crate::codegen::CodeGen;

use self::integral::init_integral;
use self::void::init_void;

mod integral;
mod void;

pub fn init_builtins(codegen: &mut CodeGen) {
    init_integral(codegen);
    init_void(codegen);
}
