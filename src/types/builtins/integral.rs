use std::collections::HashMap;

use inkwell::{intrinsics::Intrinsic, types::BasicTypeEnum};

use crate::{
    codegen::{CodeGen, Data},
    errors::{raise_error, ErrorType},
    mir::Mir,
    types::{BasicType, Lifetime, Trait, TraitType, Type},
    utils::{print_string, Position},
    Flags,
};

fn integral_add<'a>(
    codegen: &mut CodeGen<'a>,
    pos: &Position,
    this: Data<'a>,
    other: Data<'a>,
) -> Data<'a> {
    let tp = this.data.as_ref().unwrap().get_type();
    let tpname = this.tp.basictype.to_string();
    if !codegen.flags.contains(&Flags::NoOUChecks) {
        let sadd_intrinsic = Intrinsic::find(&format!("llvm.sadd.with.overflow.{}", tpname)).unwrap();
        let expect_i1 = Intrinsic::find("llvm.expect.i1").unwrap();

        let sadd_function = sadd_intrinsic
            .get_declaration(
                &codegen.module,
                &[
                    tp.into(),
                    tp.into(),
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
                sadd_function,
                &[this.data.unwrap().into(), other.data.unwrap().into()],
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
                "Error: {} addition overflow!\n    {}:{}:{}\n",
                this.tp.qualname,
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
        if let BasicTypeEnum::IntType(tp) = tp {
            phi.add_incoming(&[(&tp.get_undef(), overflow_block)]);
        }

        codegen.cur_block = Some(done_block);

        Data {
            data: Some(phi.as_basic_value()),
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

fn integral_add_skeleton<'a>(
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

pub fn init_integral(codegen: &mut CodeGen) {
    for basictype in vec![BasicType::I8,BasicType::I16,BasicType::I32,BasicType::I64,BasicType::I128,BasicType::Bool] {
        let tp = Type {
            basictype: basictype.clone(),
            traits: HashMap::from([
                (
                    TraitType::Add,
                    Trait::Add {
                        code: integral_add,
                        skeleton: integral_add_skeleton,
                    },
                ),
                (TraitType::Copy, Trait::Copy),
            ]),
            qualname: format!("std::{basictype}"),
            lifetime: Lifetime::Static,
            ref_n: 0,
        };
        codegen.builtins.insert(basictype, tp);
    }
}
