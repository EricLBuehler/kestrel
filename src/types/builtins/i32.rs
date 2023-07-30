use std::collections::HashMap;

use inkwell::intrinsics::Intrinsic;

use crate::{
    codegen::{CodeGen, Data},
    errors::{raise_error, ErrorType},
    mir::Mir,
    types::{BasicType, Lifetime, Trait, TraitType, Type},
    utils::{print_string, Position},
    Flags,
};

fn i32_add<'a>(
    codegen: &mut CodeGen<'a>,
    pos: &Position,
    this: Data<'a>,
    other: Data<'a>,
) -> Data<'a> {
    if !codegen.flags.contains(&Flags::NoOUChecks) {
        let sadd_i32_intrinsic = Intrinsic::find("llvm.sadd.with.overflow.i32").unwrap();
        let expect_i1 = Intrinsic::find("llvm.expect.i1").unwrap();

        let sadd_i32_function = sadd_i32_intrinsic
            .get_declaration(
                &codegen.module,
                &[
                    codegen.context.i32_type().into(),
                    codegen.context.i32_type().into(),
                ],
            )
            .unwrap();

        let expect_i1_function = expect_i1
            .get_declaration(
                &codegen.module,
                &[
                    codegen.context.bool_type().into(),
                    codegen.context.bool_type().into(),
                ],
            )
            .unwrap();

        let res = codegen
            .builder
            .build_call(
                sadd_i32_function,
                &[this.data.unwrap(), other.data.unwrap()],
                "",
            )
            .try_as_basic_value()
            .left();

        let result = codegen
            .builder
            .build_extract_value(res.unwrap().into_struct_value(), 0, "");
        let overflow = codegen
            .builder
            .build_extract_value(res.unwrap().into_struct_value(), 1, "");

        let overflow_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "");
        let end_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "");

        let done_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "");

        let res = codegen
            .builder
            .build_call(
                expect_i1_function,
                &[
                    overflow.unwrap().into(),
                    codegen.context.bool_type().const_int(0, true).into(),
                ],
                "",
            )
            .try_as_basic_value()
            .left();

        codegen.builder.build_conditional_branch(
            res.unwrap().into_int_value(),
            overflow_block,
            end_block,
        );

        codegen.builder.position_at_end(overflow_block);

        print_string(
            codegen,
            &format!(
                "Error: i32 addition overflow!\n    {}:{}:{}\n",
                codegen.info.name,
                pos.line + 1,
                pos.startcol + 1
            ),
        );

        codegen.builder.build_unconditional_branch(done_block);

        codegen.builder.position_at_end(end_block);
        codegen.builder.build_unconditional_branch(done_block);

        overflow_block
            .move_after(codegen.cur_block.unwrap())
            .unwrap();
        end_block.move_after(overflow_block).unwrap();
        done_block.move_after(done_block).unwrap();

        codegen.builder.position_at_end(done_block);

        let phi = codegen
            .builder
            .build_phi(this.data.unwrap().into_int_value().get_type(), "");

        phi.add_incoming(&[(&result.unwrap(), end_block)]);
        phi.add_incoming(&[(&codegen.context.i32_type().get_undef(), overflow_block)]);

        codegen.cur_block = Some(done_block);

        Data {
            data: Some(phi.as_basic_value().into()),
            tp: this.tp,
        }
    } else {
        let res = codegen.builder.build_int_add(
            this.data.unwrap().into_int_value(),
            other.data.unwrap().into_int_value(),
            "",
        );

        Data {
            data: Some(res.into()),
            tp: this.tp,
        }
    }
}

fn i32_add_skeleton<'a>(
    mir: &mut Mir,
    pos: &Position,
    this: Type<'a>,
    other: Type<'a>,
) -> Type<'a> {
    if this != other {
        raise_error(
            &format!("Expected 'i32', got '{}'", other.basictype),
            ErrorType::TypeMismatch,
            pos,
            &mir.info,
        );
    }
    this
}

pub fn init_i32(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::I32,
        traits: HashMap::from([
            (
                TraitType::Add,
                Trait::Add {
                    code: i32_add,
                    skeleton: i32_add_skeleton,
                },
            ),
            (TraitType::Copy, Trait::Copy),
        ]),
        qualname: "std::i32".into(),
        lifetime: Lifetime::Static,
    };
    codegen.builtins.insert(BasicType::I32, tp);
}
