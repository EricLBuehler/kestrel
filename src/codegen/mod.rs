use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    debug_info::{DWARFEmissionKind, DWARFSourceLanguage},
    module::FlagBehavior,
    module::Module,
    passes::PassManagerSubType,
    values::{BasicMetadataValueEnum, BasicValue, FunctionValue, PointerValue},
};
use std::{collections::HashMap, error::Error};

use crate::{
    errors::{raise_error, ErrorType},
    mir,
    parser::nodes::{Node, NodeType, OpType},
    types::{
        builtins::init_builtins, init_extern_fns, BasicType, BuiltinTypes, Trait, TraitType, Type,
    },
    utils::FileInfo,
    Flags,
};

pub struct Namespace<'a> {
    bindings: HashMap<String, (PointerValue<'a>, Type<'a>)>,
}

pub struct CodeGen<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub info: &'a FileInfo<'a>,
    dibuilder: inkwell::debug_info::DebugInfoBuilder<'a>,

    pub cur_block: Option<BasicBlock<'a>>,
    pub cur_fn: Option<FunctionValue<'a>>,

    pub builtins: BuiltinTypes<'a>,
    pub extern_fns: HashMap<String, FunctionValue<'a>>,
    namespaces: HashMap<FunctionValue<'a>, Namespace<'a>>,

    pub flags: Vec<Flags>,
}

#[derive(Debug)]
pub struct Data<'a> {
    pub data: Option<BasicMetadataValueEnum<'a>>,
    pub tp: Type<'a>,
}

impl<'a> CodeGen<'a> {
    fn compile(&mut self, ast: Vec<Node>) -> Data<'a> {
        let mut res = Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        };

        for node in ast {
            res = self.compile_expr(&node);
        }

        res
    }

    fn compile_expr(&mut self, node: &Node) -> Data<'a> {
        match node.tp {
            NodeType::Binary => self.compile_binary(node),
            NodeType::I32 => self.compile_i32(node),
            NodeType::Identifier => self.compile_load(node),
            NodeType::Let => self.compile_let(node),
        }
    }
}

impl<'a> CodeGen<'a> {
    fn compile_i32(&mut self, node: &Node) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i32>()
            .is_err()
        {
            let fmt: String = format!(
                "i32 literal in radix 10 out of bounds ({} to {}).",
                i32::MAX,
                i32::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i32_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::I32).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_binary(&mut self, node: &Node) -> Data<'a> {
        let binary = node.data.get_data();
        let left = self.compile_expr(binary.nodes.get("left").unwrap());
        let right = self.compile_expr(binary.nodes.get("right").unwrap());

        match binary.op.unwrap() {
            OpType::Add => {
                if let Some(Trait::Add { code, skeleton: _ }) = left.tp.traits.get(&TraitType::Add)
                {
                    code(self, &node.pos, left, right)
                } else {
                    raise_error(
                        &format!("Type '{}' does not implement Add.", left.tp.qualname),
                        ErrorType::TypeMismatch,
                        &node.pos,
                        &self.info,
                    );
                }
            }
        }
    }

    fn compile_let(&mut self, node: &Node) -> Data<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.compile_expr(letnode.nodes.get("expr").unwrap());

        let alloc = self
            .builder
            .build_alloca(right.data.unwrap().into_int_value().get_type(), "");

        self.builder.build_store(
            alloc,
            right.data.unwrap().into_int_value().as_basic_value_enum(),
        );

        self.namespaces
            .get_mut(&self.cur_fn.unwrap())
            .unwrap()
            .bindings
            .insert(name.clone(), (alloc, right.tp));

        Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        }
    }

    fn compile_load(&mut self, node: &Node) -> Data<'a> {
        let identifiernode = node.data.get_data();
        let name = identifiernode.raw.get("value").unwrap();

        let binding = self
            .namespaces
            .get_mut(&self.cur_fn.unwrap())
            .unwrap()
            .bindings
            .get(name);

        if binding.is_none() {
            let fmt: String = format!("Binding '{}' not found in scope.", name);
            raise_error(&fmt, ErrorType::BindingNotFound, &node.pos, self.info);
        }

        let binding = binding.unwrap();

        Data {
            data: Some(self.builder.build_load(binding.0, "").into()),
            tp: binding.1.clone(),
        }
    }
}

pub fn generate_code(
    module_name: &str,
    source_name: &str,
    ast: Vec<Node>,
    info: &FileInfo,
    flags: Vec<Flags>,
    optimize: bool,
) -> Result<(), Box<dyn Error>> {
    let context: inkwell::context::Context = Context::create();
    let module: inkwell::module::Module = context.create_module(module_name);

    let mut triple: String = String::from("");
    guess_host_triple::guess_host_triple()
        .map(|t| triple = String::from(t))
        .unwrap_or_else(|| triple = String::from("unknown-unknown-unknown"));

    module.set_triple(&inkwell::targets::TargetTriple::create(triple.as_str()));
    module.set_source_file_name(source_name);

    //Setup debug info
    module.add_basic_value_flag(
        "Debug Info Version",
        FlagBehavior::Error,
        context.i32_type().const_int(3, false),
    );
    let (dibuilder, _) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        &info.name,
        &info.dir,
        "xpl",
        optimize,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        "",
        "xpl",
    );

    let mut codegen = CodeGen {
        context: &context,
        module,
        builder: context.create_builder(),
        info,
        dibuilder,
        cur_block: None,
        cur_fn: None,
        builtins: HashMap::new(),
        extern_fns: HashMap::new(),
        namespaces: HashMap::new(),
        flags: flags.clone(),
    };

    init_builtins(&mut codegen);
    init_extern_fns(&mut codegen);

    //Pass manager (optimizer)
    let pass_manager_builder: inkwell::passes::PassManagerBuilder =
        inkwell::passes::PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(inkwell::OptimizationLevel::Aggressive);
    let manager: inkwell::passes::PassManager<Module> = inkwell::passes::PassManager::create(());
    pass_manager_builder.populate_module_pass_manager(&manager);

    //

    let main_tp: inkwell::types::FunctionType = codegen.context.i32_type().fn_type(
        &[
            inkwell::types::BasicMetadataTypeEnum::IntType(codegen.context.i32_type()),
            inkwell::types::BasicMetadataTypeEnum::PointerType(
                codegen
                    .context
                    .i32_type()
                    .ptr_type(inkwell::AddressSpace::from(0u16))
                    .ptr_type(inkwell::AddressSpace::from(0u16)),
            ),
        ],
        false,
    );
    let realmain = codegen.module.add_function("main", main_tp, None);
    let basic_block = codegen.context.append_basic_block(realmain, "");

    // Mir check
    let mut mir = mir::new(info.clone(), codegen.builtins.clone());
    let instructions = mir.generate(&ast);
    mir::check(instructions, codegen.info.clone());
    //

    codegen.namespaces.insert(
        realmain,
        Namespace {
            bindings: HashMap::new(),
        },
    );

    let mut attr: inkwell::attributes::Attribute = codegen.context.create_enum_attribute(
        inkwell::attributes::Attribute::get_named_enum_kind_id("noinline"),
        0,
    );
    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

    attr = codegen.context.create_enum_attribute(
        inkwell::attributes::Attribute::get_named_enum_kind_id("norecurse"),
        0,
    );
    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

    if !optimize {
        attr = codegen.context.create_enum_attribute(
            inkwell::attributes::Attribute::get_named_enum_kind_id("optnone"),
            0,
        );
        realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
    }

    //TODO: Ensure this is true
    attr = codegen.context.create_enum_attribute(
        inkwell::attributes::Attribute::get_named_enum_kind_id("willreturn"),
        0,
    );
    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

    for flag in flags {
        if flag == Flags::Sanitize {
            let mut attr = codegen.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_address"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

            attr = codegen.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_memory"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

            attr = codegen.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_thread"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        }
    }

    codegen.builder.position_at_end(basic_block);
    codegen.cur_block = Some(basic_block);
    codegen.cur_fn = Some(realmain);

    //

    //Compile code
    codegen.compile(ast);

    codegen
        .builder
        .build_return(Some(&codegen.context.i32_type().const_int(0, false)));

    //

    //Generate debug info
    codegen.dibuilder.finalize();

    //Optimize
    unsafe { codegen.module.run_in_pass_manager(&manager) };

    codegen.module.print_to_file(std::path::Path::new("a.ll"))?;

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

    Ok(())
}
