use std::collections::HashMap;

use inkwell::intrinsics::Intrinsic;

use crate::{
    codegen::{CodeGen, Data},
    errors::{raise_error, ErrorType},
    types::{print_panic, BasicType, Trait, TraitType, Type},
    utils::Position,
    NoFlag,
};

fn i32_add<'a>(
    codegen: &mut CodeGen<'a>,
    pos: &Position,
    this: Data<'a>,
    other: Data<'a>,
) -> Data<'a> {
    if this.tp != other.tp {
        raise_error(
            &format!("Expected 'i32', got '{}'", other.tp.basictype),
            ErrorType::TypeMismatch,
            pos,
            codegen.info,
        );
    }

    if !codegen.no_flags.contains(&NoFlag::OUChecks) {
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
                "i32_sadd",
            )
            .try_as_basic_value()
            .left();

        let result =
            codegen
                .builder
                .build_extract_value(res.unwrap().into_struct_value(), 0, "result");
        let overflow =
            codegen
                .builder
                .build_extract_value(res.unwrap().into_struct_value(), 1, "overflow");

        let end_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "end");

        let overflow_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "else");
        let done_block: inkwell::basic_block::BasicBlock = codegen
            .context
            .append_basic_block(codegen.cur_fn.unwrap(), "done");

        let res = codegen
            .builder
            .build_call(
                expect_i1_function,
                &[
                    overflow.unwrap().into(),
                    codegen.context.bool_type().const_int(0, true).into(),
                ],
                "sadd1",
            )
            .try_as_basic_value()
            .left();

        codegen.builder.build_conditional_branch(
            res.unwrap().into_int_value(),
            overflow_block,
            end_block,
        );

        codegen.builder.position_at_end(overflow_block);

        print_panic(codegen, &format!("Error: i32 addition overflow!\n    {}:{}:{}\n", codegen.info.name, pos.line + 1, pos.startcol + 1));

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
            .build_phi(this.data.unwrap().into_int_value().get_type(), "phi");

        phi.add_incoming(&[(&result.unwrap(), end_block)]);
        phi.add_incoming(&[(
            &codegen.context.i32_type().const_int(u32::MAX.into(), false),
            overflow_block,
        )]);

        codegen.cur_block = Some(done_block);

        Data {
            data: Some(phi.as_basic_value().into()),
            tp: this.tp,
        }
    } else {
        let res = codegen.builder.build_int_add(
            this.data.unwrap().into_int_value(),
            other.data.unwrap().into_int_value(),
            "i32sum",
        );

        Data {
            data: Some(res.into()),
            tp: this.tp,
        }
    }
}

pub fn init_i32(codegen: &mut CodeGen) {
    let tp = Type {
        basictype: BasicType::I32,
        traits: HashMap::from([(TraitType::Add, Trait::Add(i32_add))]),
    };
    codegen.builtins.insert(BasicType::I32, tp);
}
