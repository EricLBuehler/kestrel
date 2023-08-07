use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    debug_info::{DWARFEmissionKind, DWARFSourceLanguage},
    module::FlagBehavior,
    module::Module,
    passes::PassManagerSubType,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};
use std::{collections::HashMap, error::Error, fs::OpenOptions};

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

pub struct BindingTags {
    pub is_mut: bool,
}

pub struct Namespace<'a> {
    bindings: HashMap<String, (Option<PointerValue<'a>>, Type<'a>, BindingTags)>,
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
    pub functions: HashMap<String, Node>, //(args, code)
    namespaces: HashMap<FunctionValue<'a>, Namespace<'a>>,

    pub flags: Vec<Flags>,
    pub optimized: bool,
}

#[derive(Debug)]
pub struct Data<'a> {
    pub data: Option<BasicValueEnum<'a>>,
    pub tp: Type<'a>,
}

struct ExprFlags {
    get_ref: bool,
}

impl<'a> CodeGen<'a> {
    fn compile(&mut self, ast: Vec<Node>) {
        for node in ast {
            match node.tp {
                NodeType::Fn => {
                    self.create_fn(node);
                }
                _ => {

                }
            }
        }
    }

    fn compile_statements(&mut self, ast: &Vec<Node>) -> Data<'a> {
        let mut res = Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        };

        for node in ast {
            res = self.compile_expr(node, ExprFlags { get_ref: false });
        }

        res
    }

    fn compile_expr(&mut self, node: &Node, flags: ExprFlags) -> Data<'a> {
        match node.tp {
            NodeType::Binary => self.compile_binary(node, flags),
            NodeType::I32 => self.compile_i32(node, flags),
            NodeType::Identifier => self.compile_load(node, flags),
            NodeType::Let => self.compile_let(node, flags),
            NodeType::Store => self.compile_store(node, flags),
            NodeType::Reference => self.compile_reference(node, flags),
            NodeType::I8 => self.compile_i8(node, flags),
            NodeType::I16 => self.compile_i16(node, flags),
            NodeType::I64 => self.compile_i64(node, flags),
            NodeType::I128 => self.compile_i128(node, flags),
            NodeType::Bool => self.compile_bool(node, flags),
            NodeType::U8 => self.compile_u8(node, flags),
            NodeType::U16 => self.compile_u16(node, flags),
            NodeType::U32 => self.compile_u32(node, flags),
            NodeType::U64 => self.compile_u64(node, flags),
            NodeType::U128 => self.compile_u128(node, flags),
            NodeType::Fn => {
                raise_error("Nested function definitions are disallowed.", ErrorType::NestedFnDef, &node.pos, &self.info);
            }
        }
    }
}

impl<'a> CodeGen<'a> {
    fn compile_i8(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i8>()
            .is_err()
        {
            let fmt: String = format!(
                "i8 literal in radix 10 out of bounds ({} to {}).",
                i8::MAX,
                i8::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i8_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::I8).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i16(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i16>()
            .is_err()
        {
            let fmt: String = format!(
                "i16 literal in radix 10 out of bounds ({} to {}).",
                i16::MAX,
                i16::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i16_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::I16).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i32(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
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

    fn compile_i64(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i64>()
            .is_err()
        {
            let fmt: String = format!(
                "i64 literal in radix 10 out of bounds ({} to {}).",
                i64::MAX,
                i64::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i64_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::I64).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i128(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<i128>()
            .is_err()
        {
            let fmt: String = format!(
                "i128 literal in radix 10 out of bounds ({} to {}).",
                i128::MAX,
                i128::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i128_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::I128).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u8(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u8>()
            .is_err()
        {
            let fmt: String = format!(
                "u8 literal in radix 10 out of bounds ({} to {}).",
                u8::MAX,
                u8::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i8_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::U8).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u16(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u16>()
            .is_err()
        {
            let fmt: String = format!(
                "u16 literal in radix 10 out of bounds ({} to {}).",
                u16::MAX,
                u16::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i16_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::U16).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u32(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u32>()
            .is_err()
        {
            let fmt: String = format!(
                "u32 literal in radix 10 out of bounds ({} to {}).",
                u32::MAX,
                u32::MIN
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
                tp: self.builtins.get(&BasicType::U32).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u64(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u64>()
            .is_err()
        {
            let fmt: String = format!(
                "u64 literal in radix 10 out of bounds ({} to {}).",
                u64::MAX,
                u64::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i64_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::U64).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u128(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        if node
            .data
            .get_data()
            .raw
            .get("value")
            .unwrap()
            .parse::<u128>()
            .is_err()
        {
            let fmt: String = format!(
                "u128 literal in radix 10 out of bounds ({} to {}).",
                u128::MAX,
                u128::MIN
            );
            raise_error(
                &fmt,
                ErrorType::InvalidLiteralForRadix,
                &node.pos,
                self.info,
            );
        }

        let res = self.context.i128_type().const_int_from_string(
            node.data.get_data().raw.get("value").unwrap(),
            inkwell::types::StringRadix::Decimal,
        );

        if let Some(int) = res {
            Data {
                data: Some(int.into()),
                tp: self.builtins.get(&BasicType::U128).unwrap().clone(),
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_bool(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        match node.data.get_data().booleans.get("value").unwrap() {
            true => {
                let res = self.context.bool_type().const_int(1, false);
                Data {
                    data: Some(res.into()),
                    tp: self.builtins.get(&BasicType::Bool).unwrap().clone(),
                }
            }
            false => {
                let res = self.context.bool_type().const_int(0, false);
                Data {
                    data: Some(res.into()),
                    tp: self.builtins.get(&BasicType::Bool).unwrap().clone(),
                }
            }
        }
    }

    fn compile_binary(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let binary = node.data.get_data();
        let left = self.compile_expr(
            binary.nodes.get("left").unwrap(),
            ExprFlags { get_ref: false },
        );
        let right = self.compile_expr(
            binary.nodes.get("right").unwrap(),
            ExprFlags { get_ref: false },
        );

        match binary.op.unwrap() {
            OpType::Add => {
                if let Some(Trait::Add { code, skeleton: _ }) = left.tp.traits.get(&TraitType::Add)
                {
                    code(self, &node.pos, left, right)
                } else {
                    raise_error(
                        &format!("Type '{}' does not implement Add.", left.tp.qualname()),
                        ErrorType::TraitNotImplemented,
                        &node.pos,
                        self.info,
                    );
                }
            }
        }
    }

    fn compile_let(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.compile_expr(
            letnode.nodes.get("expr").unwrap(),
            ExprFlags { get_ref: false },
        );
        let is_mut = letnode.booleans.get("is_mut").unwrap();

        if right.data.is_some() {
            let alloc = self
                .builder
                .build_alloca(right.data.unwrap().get_type(), "");

            self.builder.build_store(alloc, right.data.unwrap());
            self.namespaces
                .get_mut(&self.cur_fn.unwrap())
                .unwrap()
                .bindings
                .insert(
                    name.clone(),
                    (Some(alloc), right.tp, BindingTags { is_mut: *is_mut }),
                );
        } else {
            self.namespaces
                .get_mut(&self.cur_fn.unwrap())
                .unwrap()
                .bindings
                .insert(
                    name.clone(),
                    (None, right.tp, BindingTags { is_mut: *is_mut }),
                );
        }

        Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        }
    }

    fn compile_load(&mut self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            data: if binding.0.is_some() {
                Some(if flags.get_ref {
                    binding.0.unwrap().into()
                } else {
                    self.builder.build_load(binding.0.unwrap(), "")
                })
            } else {
                None
            },
            tp: binding.1.clone(),
        }
    }

    fn compile_store(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let storenode = node.data.get_data();
        let name = storenode.raw.get("name").unwrap();
        let expr = storenode.nodes.get("expr").unwrap();
        let right = self.compile_expr(expr, ExprFlags { get_ref: false });

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

        if right.tp != binding.1 {
            raise_error(
                &format!(
                    "Expected '{}', got '{}'",
                    binding.1.qualname(),
                    right.tp.qualname()
                ),
                ErrorType::TypeMismatch,
                &expr.pos,
                self.info,
            );
        }

        if !binding.2.is_mut {
            raise_error(
                &format!(
                    "Binding '{}' is not mutable, so it cannot be assigned to.",
                    name
                ),
                ErrorType::BindingNotMutable,
                &node.pos,
                self.info,
            );
        }

        if right.data.is_some() && right.data.unwrap().is_pointer_value() {
            debug_assert!(binding.0.is_some());
            self.builder
                .build_store(binding.0.unwrap(), right.data.unwrap());
        } else if right.data.is_some() {
            let alloc = self
                .builder
                .build_alloca(right.data.unwrap().get_type(), "");
            self.builder.build_store(alloc, right.data.unwrap());
        }

        Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        }
    }

    fn compile_reference(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let referencenode = node.data.get_data();
        let mut expr = self.compile_expr(
            referencenode.nodes.get("expr").unwrap(),
            ExprFlags { get_ref: true },
        );

        expr.tp.ref_n += 1;

        expr
    }
}

impl<'a> CodeGen<'a> {
    fn create_fn(&mut self, node: Node) {
        let fnnode = node.data.get_data();
        let name = fnnode.raw.get("name").unwrap();

        if name == "main" {
            let main_tp: inkwell::types::FunctionType = self.context.i32_type().fn_type(
                &[
                    inkwell::types::BasicMetadataTypeEnum::IntType(self.context.i32_type()),
                    inkwell::types::BasicMetadataTypeEnum::PointerType(
                        self
                            .context
                            .i32_type()
                            .ptr_type(inkwell::AddressSpace::from(0u16))
                            .ptr_type(inkwell::AddressSpace::from(0u16)),
                    ),
                ],
                false,
            );
            let realmain = self.module.add_function("main", main_tp, None);
            let basic_block = self.context.append_basic_block(realmain, "");
        
            // Mir check
            let mut mir = mir::new(self.info.clone(), self.builtins.clone(), name.into());
            let mut instructions = mir.generate(&fnnode.nodearr.unwrap());
            mir::check(&mut mir, &mut instructions);
            //
        
            self.namespaces.insert(
                realmain,
                Namespace {
                    bindings: HashMap::new(),
                },
            );
        
            let mut attr: inkwell::attributes::Attribute = self.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("noinline"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        
            attr = self.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("norecurse"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        
            if !self.optimized {
                attr = self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("optnone"),
                    0,
                );
                realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
            }
        
            //TODO: Ensure this is true
            attr = self.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("willreturn"),
                0,
            );
            realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        
            for flag in &self.flags {
                if flag == &Flags::Sanitize {
                    let mut attr = self.context.create_enum_attribute(
                        inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_address"),
                        0,
                    );
                    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        
                    attr = self.context.create_enum_attribute(
                        inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_memory"),
                        0,
                    );
                    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        
                    attr = self.context.create_enum_attribute(
                        inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_thread"),
                        0,
                    );
                    realmain.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
                }
            }
        
            self.builder.position_at_end(basic_block);
            self.cur_block = Some(basic_block);
            self.cur_fn = Some(realmain);
        
            //
        
            //Compile code
            self.compile_statements(fnnode.nodearr.unwrap());
        
            self
                .builder
                .build_return(Some(&self.context.i32_type().const_int(0, false)));
        
            //
        }
        else {
            self.functions.insert(name.clone(), node);
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
        "kestrel",
        optimize,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        "",
        "kestrel",
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
        functions: HashMap::new(),
        namespaces: HashMap::new(),
        flags: flags.clone(),
        optimized: optimize,
    };

    let f = OpenOptions::new().write(true).append(true).open("a.mir").expect("Unable to create MIR output file.");
    f.set_len(0).expect("Unable to truncate MIR output file.");

    init_builtins(&mut codegen);
    init_extern_fns(&mut codegen);

    //Pass manager (optimizer)
    let pass_manager_builder: inkwell::passes::PassManagerBuilder =
        inkwell::passes::PassManagerBuilder::create();
    pass_manager_builder.set_optimization_level(inkwell::OptimizationLevel::Aggressive);
    let manager: inkwell::passes::PassManager<Module> = inkwell::passes::PassManager::create(());
    pass_manager_builder.populate_module_pass_manager(&manager);

    //

    codegen.compile(ast);
    

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
