use clap::Parser;
use inkwell::{context::Context, intrinsics::Intrinsic, AddressSpace};

//Version: major.minor
#[derive(Parser, Debug)]
#[command(author, version = "0.1.0", about, long_about = None)]
struct Args {
    /// File to execute
    #[arg(required = true, name = "file")]
    file: String,
}

fn main() {
    let sadd_i32_intrinsic = Intrinsic::find("llvm.sadd.with.overflow.i8").unwrap();
    let expect_i1 = Intrinsic::find("llvm.expect.i1").unwrap();

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let fn_type = context.i32_type().fn_type(&[], false);
    let fn_value = module.add_function("main", fn_type, None);
    let entry = context.append_basic_block(fn_value, "entry");

    builder.position_at_end(entry);

    let sadd_i8_intrinsic_function = sadd_i32_intrinsic
        .get_declaration(
            &module,
            &[context.i8_type().into(), context.i8_type().into()],
        )
        .unwrap();

    let expect_i1_function = expect_i1
        .get_declaration(
            &module,
            &[context.bool_type().into(), context.bool_type().into()],
        )
        .unwrap();

    let x = context.i8_type().const_int(127, false).into();
    let y = context.i8_type().const_int(1, false).into();
    let res = builder
        .build_call(sadd_i8_intrinsic_function, &[x, y], "sadd1")
        .try_as_basic_value()
        .left();

    let _result = builder.build_extract_value(res.unwrap().into_struct_value(), 0, "result");
    let overflow = builder.build_extract_value(res.unwrap().into_struct_value(), 1, "overflow");

    let end_block: inkwell::basic_block::BasicBlock = context.append_basic_block(fn_value, "end");
    let overflow_block: inkwell::basic_block::BasicBlock =
        context.append_basic_block(fn_value, "else");

    let res = builder
        .build_call(
            expect_i1_function,
            &[
                overflow.unwrap().into(),
                context.bool_type().const_int(0, true).into(),
            ],
            "sadd1",
        )
        .try_as_basic_value()
        .left();

    builder.build_conditional_branch(res.unwrap().into_int_value(), overflow_block, end_block);

    builder.position_at_end(overflow_block);

    let printftp = context.i32_type().fn_type(
        &[context.i8_type().ptr_type(AddressSpace::from(0)).into()],
        false,
    );
    let printf = module.add_function("printf", printftp, Some(inkwell::module::Linkage::External));

    let message = "Error: i8 operation overflow!\n";

    let mut arrv = Vec::new();
    for c in message.bytes() {
        arrv.push(context.i8_type().const_int(c as u64, false));
    }
    arrv.push(context.i8_type().const_zero());

    let str = context.i8_type().const_array(&arrv[..]);

    let strct = context.struct_type(&[str.get_type().into()], false);
    let mem = builder.build_alloca(strct, "string");

    let ptr = builder
        .build_struct_gep(mem, 0_u32, "ptr")
        .expect("GEP error");
    builder.build_store(ptr, str);
    let ptr = unsafe {
        builder.build_gep(
            ptr,
            &[
                context.i32_type().const_zero(),
                context.i32_type().const_zero(),
            ],
            "ptr",
        )
    };

    builder.build_call(printf, &[ptr.into()], "printf_call");
    builder.build_return(Some(&context.i32_type().const_int(1, false)));

    builder.position_at_end(end_block);
    builder.build_return(Some(&context.i32_type().const_int(0, false)));

    overflow_block.move_after(entry).unwrap();
    end_block.move_after(overflow_block).unwrap();

    module
        .print_to_file("a.ll")
        .expect(".ll file saving failed.");

    let mut res: std::process::Output = std::process::Command::new("llc")
        .arg("a.ll")
        .output()
        .expect("Failed to execute llc");
    if !res.status.success() {
        println!(
            "Stderr:\n{}\n\nStdout:{}",
            std::str::from_utf8(&res.stderr[..]).expect("Unable to convert for stderr (llc)"),
            std::str::from_utf8(&res.stdout[..]).expect("Unable to convert for stdout (llc)")
        );
        panic!("Failed to run llc (exit code {})", res.status);
    }

    res = std::process::Command::new("gcc")
        .arg("a.s")
        .arg("-oa.o")
        .arg("-c")
        .output()
        .expect("Failed to execute gcc");
    if !res.status.success() {
        println!(
            "Stderr:\n{}\n\nStdout:{}",
            std::str::from_utf8(&res.stderr[..]).expect("Unable to convert for stderr (gcc)"),
            std::str::from_utf8(&res.stdout[..]).expect("Unable to convert for stdout (gcc)")
        );
        panic!("Failed to run gcc (exit code {})", res.status);
    }

    res = std::process::Command::new("gcc")
        .arg("a.s")
        .arg("-oa.out")
        .arg("-no-pie")
        .output()
        .expect("Failed to execute gcc");
    if !res.status.success() {
        println!(
            "Stderr:\n{}\n\nStdout:{}",
            std::str::from_utf8(&res.stderr[..]).expect("Unable to convert for stderr (gcc)"),
            std::str::from_utf8(&res.stdout[..]).expect("Unable to convert for stdout (gcc)")
        );
        panic!("Failed to run gcc (exit code {})", res.status);
    }
}
