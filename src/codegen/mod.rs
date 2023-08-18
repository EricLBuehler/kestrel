use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    debug_info::{DWARFEmissionKind, DWARFSourceLanguage},
    module::FlagBehavior,
    module::Module,
    passes::PassManagerSubType,
    types::{AnyTypeEnum, BasicMetadataTypeEnum, FunctionType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use std::{collections::HashMap, error::Error, fs::OpenOptions};

use crate::{
    errors::{raise_error, raise_error_multi, ErrorType},
    mir,
    parser::nodes::{Node, NodeType, OpType},
    types::{
        builtins::init_builtins, init_extern_fns, BasicType, BuiltinTypes, Trait, TraitType, Type,
    },
    utils::{FileInfo, Position},
    Flags,
};

pub struct BindingTags {
    pub is_mut: bool,
}

pub struct Namespace<'a> {
    bindings: HashMap<String, (Option<PointerValue<'a>>, Type<'a>, BindingTags)>,
}

#[derive(Clone)]
pub struct CurFunctionState<'a> {
    pub cur_block: Option<BasicBlock<'a>>,
    pub returned: bool,
    pub rettp: Type<'a>,
}

pub type CodegenFunctions<'a> =
    HashMap<String, (Node, (Vec<Type<'a>>, Type<'a>), Option<FunctionValue<'a>>)>; //(args, (code, (args, rettp), function)

pub struct CodeGen<'a> {
    pub context: &'a Context,
    pub module: Module<'a>,
    pub builder: Builder<'a>,
    pub info: &'a FileInfo<'a>,
    dibuilder: inkwell::debug_info::DebugInfoBuilder<'a>,
    pub block: Option<BasicBlock<'a>>,

    pub cur_fnstate: Option<CurFunctionState<'a>>,
    pub cur_fn: Option<FunctionValue<'a>>,

    pub builtins: BuiltinTypes<'a>,
    pub extern_fns: HashMap<String, FunctionValue<'a>>,
    pub functions: CodegenFunctions<'a>, //(args, (code, (args, rettp))
    namespaces: HashMap<FunctionValue<'a>, Namespace<'a>>,

    pub flags: Vec<Flags>,
    pub optimized: bool,
    pub debug_mir: bool,
}

#[derive(Debug)]
pub struct Data<'a> {
    pub data: Option<BasicValueEnum<'a>>,
    pub tp: Type<'a>,
}

enum RefOptions {
    Normal,
    Deref,
    Ref,
}

struct ExprFlags {
    ref_opt: RefOptions,
}

impl<'a> CodeGen<'a> {
    fn compile(&mut self, ast: Vec<Node>) {
        //Hoist definitions
        for node in ast.clone() {
            match node.tp {
                NodeType::Fn => {
                    self.hoist_fn_def(node);
                }
                _ => {
                    raise_error(
                        "Only function definitions are allowed at the module level.",
                        ErrorType::NonModuleLevelStatement,
                        &node.pos,
                        self.info,
                    );
                }
            }
        }

        if !self.functions.contains_key("main") {
            self.add_main_skeleton();
        }

        for node in &ast {
            match node.tp {
                NodeType::Fn => {
                    self.create_fn(node);
                }
                _ => {
                    unreachable!()
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
            res = self.compile_expr(
                node,
                ExprFlags {
                    ref_opt: RefOptions::Normal,
                },
            );
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
                raise_error(
                    "Nested function definitions are disallowed.",
                    ErrorType::NestedFnDef,
                    &node.pos,
                    self.info,
                );
            }
            NodeType::Return => self.compile_return(node, flags),
            NodeType::Call => self.compile_call(node, flags),
            NodeType::Deref => self.compile_deref(node, flags),
            NodeType::If => self.compile_if(node, flags),
        }
    }

    fn add_attrs(&mut self, function: FunctionValue) {
        let mut attr: inkwell::attributes::Attribute = self.context.create_enum_attribute(
            inkwell::attributes::Attribute::get_named_enum_kind_id("noinline"),
            0,
        );
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

        attr = self.context.create_enum_attribute(
            inkwell::attributes::Attribute::get_named_enum_kind_id("norecurse"),
            0,
        );
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

        if !self.optimized {
            attr = self.context.create_enum_attribute(
                inkwell::attributes::Attribute::get_named_enum_kind_id("optnone"),
                0,
            );
            function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
        }

        //TODO: Ensure this is true
        attr = self.context.create_enum_attribute(
            inkwell::attributes::Attribute::get_named_enum_kind_id("willreturn"),
            0,
        );
        function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

        for flag in &self.flags {
            if flag == &Flags::Sanitize {
                let mut attr = self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_address"),
                    0,
                );
                function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

                attr = self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_memory"),
                    0,
                );
                function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);

                attr = self.context.create_enum_attribute(
                    inkwell::attributes::Attribute::get_named_enum_kind_id("sanitize_thread"),
                    0,
                );
                function.add_attribute(inkwell::attributes::AttributeLoc::Function, attr);
            }
        }
    }
}

impl<'a> CodeGen<'a> {
    fn kestrel_to_inkwell_tp(context: &'a Context, tp: &Type<'a>) -> AnyTypeEnum<'a> {
        match tp.basictype {
            BasicType::Bool => {
                let inkwell_tp = context.bool_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::I128 | BasicType::U128 => {
                let inkwell_tp = context.i128_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::I64 | BasicType::U64 => {
                let inkwell_tp = context.i64_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::I32 | BasicType::U32 => {
                let inkwell_tp = context.i32_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::I16 | BasicType::U16 => {
                let inkwell_tp = context.i16_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::I8 | BasicType::U8 => {
                let inkwell_tp = context.i8_type();
                if tp.ref_n > 0 {
                    let mut inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    for _ in 1..tp.ref_n {
                        inkwell_tp = inkwell_tp.ptr_type(AddressSpace::from(0u16));
                    }
                    inkwell_tp.into()
                } else {
                    inkwell_tp.into()
                }
            }
            BasicType::Void => context.void_type().into(),
        }
    }
    fn create_fn_tp(
        context: &'a Context,
        args: &[Type<'a>],
        return_type: &Type<'a>,
    ) -> FunctionType<'a> {
        let args: Vec<BasicMetadataTypeEnum> = args
            .iter()
            .map(|x| Self::kestrel_to_inkwell_tp(context, x))
            .filter(|x: &AnyTypeEnum<'_>| !x.is_void_type() && !x.is_function_type())
            .map(|x| match x {
                AnyTypeEnum::ArrayType(tp) => tp.into(),
                AnyTypeEnum::FloatType(tp) => tp.into(),
                AnyTypeEnum::FunctionType(_) => {
                    unreachable!()
                }
                AnyTypeEnum::IntType(tp) => tp.into(),
                AnyTypeEnum::PointerType(tp) => tp.into(),
                AnyTypeEnum::StructType(tp) => tp.into(),
                AnyTypeEnum::VectorType(tp) => tp.into(),
                AnyTypeEnum::VoidType(_) => {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();

        match Self::kestrel_to_inkwell_tp(context, return_type) {
            AnyTypeEnum::ArrayType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::FloatType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::IntType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::PointerType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::StructType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::VectorType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::VoidType(tp) => tp.fn_type(&args[..], false),
            AnyTypeEnum::FunctionType(_) => unreachable!(),
        }
    }

    fn resolve_type(builtins: &BuiltinTypes<'a>, info: &FileInfo<'a>, name: &Node) -> Type<'a> {
        assert!(name.tp == NodeType::Identifier);
        let data = name.data.get_data();
        let name_str = data.raw.get("value").unwrap();

        for basictype in vec![
            BasicType::I8,
            BasicType::I16,
            BasicType::I32,
            BasicType::I64,
            BasicType::I128,
            BasicType::Bool,
            BasicType::U8,
            BasicType::U16,
            BasicType::U32,
            BasicType::U64,
            BasicType::U128,
            BasicType::Void,
        ] {
            if name_str == &basictype.to_string() {
                return builtins.get(&basictype).unwrap().clone();
            }
        }

        let fmt: String = format!("Type '{}' not found.", name_str);
        raise_error(&fmt, ErrorType::TypeNotFound, &name.pos, info);
    }
}

impl<'a> CodeGen<'a> {
    fn compile_i8(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::I8).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::I8).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i16(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::I16).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::I16).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i32(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::I32).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::I32).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i64(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::I64).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::I64).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_i128(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::I128).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::I128).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u8(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::U8).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::U8).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u16(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::U16).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::U16).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u32(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::U32).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::U32).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u64(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::U64).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::U64).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_u128(&self, node: &Node, flags: ExprFlags) -> Data<'a> {
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
            if matches!(flags.ref_opt, RefOptions::Ref) {
                let ptr = self.builder.build_alloca(int.get_type(), "");
                let mut tp = self.builtins.get(&BasicType::U128).unwrap().clone();
                tp.ref_n += 1;
                Data {
                    data: Some(ptr.into()),
                    tp,
                }
            } else {
                Data {
                    data: Some(int.into()),
                    tp: self.builtins.get(&BasicType::U128).unwrap().clone(),
                }
            }
        } else {
            unimplemented!();
        }
    }

    fn compile_bool(&self, node: &Node, _flags: ExprFlags) -> Data<'a> {
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
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
        );
        let right = self.compile_expr(
            binary.nodes.get("right").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
        );

        let traittp = match binary.op.unwrap() {
            OpType::Add => TraitType::Add,
            OpType::Eq => TraitType::Eq,
            OpType::Ne => TraitType::Ne,
        };

        let t = left.tp.traits.get(&traittp);

        if let Some(Trait::Add {
            code,
            skeleton: _,
            ref_n: _,
        }) = t
        {
            code(self, &node.pos, left, right)
        } else if let Some(Trait::Eq {
            code,
            skeleton: _,
            ref_n: _,
        }) = t
        {
            code(self, &node.pos, left, right)
        } else if let Some(Trait::Ne {
            code,
            skeleton: _,
            ref_n: _,
        }) = t
        {
            code(self, &node.pos, left, right)
        } else {
            unreachable!()
        }
    }

    fn compile_let(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let letnode = node.data.get_data();
        let name = letnode.raw.get("name").unwrap();
        let right = self.compile_expr(
            letnode.nodes.get("expr").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
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

        let binding = binding.unwrap();

        if matches!(flags.ref_opt, RefOptions::Ref) {
            let mut tp = binding.1.clone();
            tp.ref_n += 1;
            Data {
                data: if binding.0.is_some() {
                    Some(binding.0.unwrap().into())
                } else {
                    None
                },
                tp,
            }
        } else if matches!(flags.ref_opt, RefOptions::Deref) {
            let mut tp = binding.1.clone();
            tp.ref_n -= 1;
            Data {
                data: if binding.0.is_some() {
                    Some(
                        self.builder.build_load(
                            self.builder
                                .build_load(binding.0.unwrap(), "")
                                .into_pointer_value(),
                            "",
                        ),
                    )
                } else {
                    None
                },
                tp: binding.1.clone(),
            }
        } else {
            Data {
                data: if binding.0.is_some() {
                    Some(self.builder.build_load(binding.0.unwrap(), ""))
                } else {
                    None
                },
                tp: binding.1.clone(),
            }
        }
    }

    fn compile_store(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let storenode = node.data.get_data();
        let name = storenode.raw.get("name").unwrap();
        let expr = storenode.nodes.get("expr").unwrap();
        let right = self.compile_expr(
            expr,
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
        );

        let binding = self
            .namespaces
            .get_mut(&self.cur_fn.unwrap())
            .unwrap()
            .bindings
            .get(name);

        let binding = binding.unwrap();

        if right.data.is_some() {
            debug_assert!(binding.0.is_some());
            self.builder
                .build_store(binding.0.unwrap(), right.data.unwrap());
        }

        Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        }
    }

    fn compile_reference(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let referencenode = node.data.get_data();
        let expr = self.compile_expr(
            referencenode.nodes.get("expr").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Ref,
            },
        );

        expr
    }

    fn compile_return(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let returnnode = node.data.get_data();
        let expr = self.compile_expr(
            returnnode.nodes.get("expr").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
        );

        if self.cur_fnstate.as_ref().unwrap().rettp != expr.tp {
            raise_error(
                &format!(
                    "Expected '{}', got '{}'",
                    self.cur_fnstate.as_ref().unwrap().rettp.qualname(),
                    expr.tp.qualname()
                ),
                ErrorType::TypeMismatch,
                &node.pos,
                self.info,
            );
        }

        if expr.data.is_some() {
            self.builder.build_return(Some(expr.data.as_ref().unwrap()));
        } else {
            self.builder.build_return(None);
        }

        self.cur_fnstate.as_mut().unwrap().returned = true;

        Data {
            data: None,
            tp: self.builtins.get(&BasicType::Void).unwrap().clone(),
        }
    }

    fn compile_call(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let callnode = node.data.get_data();
        let name = callnode.raw.get("name").unwrap().clone();

        let mut func = self.functions.get(&name).unwrap().clone();

        let func_rettp = func.1 .1.clone();
        let args = func.1 .0.clone();

        if func.2.is_none() {
            let fnnode = func.0.data.get_data();

            let fn_tp = Self::create_fn_tp(self.context, &args, &func_rettp);

            let fn_real = self.module.add_function(&name, fn_tp, None);

            func.2 = Some(fn_real);
            self.functions.insert(name.clone(), func.clone());

            let basic_block = self.context.append_basic_block(fn_real, "");

            // Mir check
            let mut mir = mir::new(
                self.info.clone(),
                self.builtins.clone(),
                self.functions.clone(),
                name.clone(),
                node.pos.clone(),
                self.debug_mir,
            );
            let mut instructions = mir.generate(fnnode.nodearr.unwrap());
            mir::check(&mut mir, &mut instructions, true, &mut HashMap::new());
            //

            self.namespaces.insert(
                fn_real,
                Namespace {
                    bindings: HashMap::new(),
                },
            );

            let old_block = self.block;

            self.builder.position_at_end(basic_block);
            self.block = Some(basic_block);

            let old_state = self.cur_fnstate.clone();
            self.cur_fnstate = Some(CurFunctionState {
                cur_block: Some(basic_block),
                returned: false,
                rettp: func_rettp.clone(),
            });

            let old_fn = self.cur_fn;
            self.cur_fn = Some(fn_real);

            //

            //Compile code
            self.compile_statements(fnnode.nodearr.unwrap());

            if !self.cur_fnstate.as_ref().unwrap().returned
                && func_rettp.basictype == BasicType::Void
            {
                self.builder.build_return(None);
            } else if !self.cur_fnstate.as_ref().unwrap().returned
                && func_rettp.basictype != BasicType::Void
            {
                raise_error(
                    &format!("Expected 'void', got '{}'", func_rettp.qualname()),
                    ErrorType::TypeMismatch,
                    &node.pos,
                    self.info,
                );
            }
            //

            self.cur_fn = old_fn;
            self.cur_fnstate = old_state;
            self.block = old_block;

            self.builder.position_at_end(self.block.unwrap());
        }

        Data {
            data: Some(
                self.builder
                    .build_call(func.2.unwrap(), &[], "")
                    .try_as_basic_value()
                    .unwrap_left(),
            ),
            tp: func_rettp,
        }
    }

    fn compile_deref(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let derefnode = node.data.get_data();
        let expr = self.compile_expr(
            derefnode.nodes.get("expr").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Deref,
            },
        );

        expr
    }

    fn compile_if(&mut self, node: &Node, _flags: ExprFlags) -> Data<'a> {
        let ifnode = node.data.get_data();
        let expr = self.compile_expr(
            ifnode.nodes.get("expr").unwrap(),
            ExprFlags {
                ref_opt: RefOptions::Normal,
            },
        );

        let if_block = self.context.append_basic_block(self.cur_fn.unwrap(), "");

        let done_block = self.context.append_basic_block(self.cur_fn.unwrap(), "");

        if_block
            .move_after(self.cur_fnstate.as_ref().unwrap().cur_block.unwrap())
            .unwrap();

        self.builder.build_conditional_branch(
            expr.data.unwrap().into_int_value(),
            if_block,
            done_block,
        );

        self.builder.position_at_end(if_block);

        self.compile_statements(&ifnode.nodearr.unwrap());
        self.builder.build_unconditional_branch(done_block);

        self.builder.position_at_end(done_block);

        expr
    }
}

impl<'a> CodeGen<'a> {
    fn hoist_fn_def(&mut self, node: Node) {
        let fnnode = node.data.get_data();
        let name = fnnode.raw.get("name").unwrap();

        if self.functions.get(name).is_some() {
            raise_error_multi(
                vec![
                    format!("Function {} is defined multiple times.", name),
                    "First definition here:".into(),
                ],
                ErrorType::MultipleFunctionDefinitions,
                vec![&node.pos, &self.functions.get(name).as_ref().unwrap().0.pos],
                self.info,
            );
        }

        let rettp = if let Some(ref v) = fnnode.tp {
            Self::resolve_type(&self.builtins, self.info, v)
        } else {
            self.builtins.get(&BasicType::Void).unwrap().clone()
        };

        self.functions
            .insert(name.clone(), (node, (vec![], rettp), None));
    }

    fn create_fn(&mut self, node: &Node) {
        let fnnode = node.data.get_data();
        let name = fnnode.raw.get("name").unwrap();

        if name == "main" {
            let main_tp: inkwell::types::FunctionType = self.context.i32_type().fn_type(
                &[
                    inkwell::types::BasicMetadataTypeEnum::IntType(self.context.i32_type()),
                    inkwell::types::BasicMetadataTypeEnum::PointerType(
                        self.context
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
            let mut mir = mir::new(
                self.info.clone(),
                self.builtins.clone(),
                self.functions.clone(),
                name.into(),
                node.pos.clone(),
                self.debug_mir,
            );
            let mut instructions = mir.generate(fnnode.nodearr.unwrap());
            mir::check(&mut mir, &mut instructions, true, &mut HashMap::new());
            //

            self.namespaces.insert(
                realmain,
                Namespace {
                    bindings: HashMap::new(),
                },
            );

            self.add_attrs(realmain);

            self.builder.position_at_end(basic_block);
            self.block = Some(basic_block);

            self.cur_fnstate = Some(CurFunctionState {
                cur_block: Some(basic_block),
                returned: false,
                rettp: self.builtins.get(&BasicType::I32).unwrap().clone(),
            });
            self.cur_fn = Some(realmain);

            //

            //Compile code
            self.compile_statements(fnnode.nodearr.unwrap());

            if !self.cur_fnstate.as_ref().unwrap().returned {
                self.builder
                    .build_return(Some(&self.context.i32_type().const_int(0, false)));
            }

            //
        }
    }

    fn add_main_skeleton(&mut self) {
        let main_tp: inkwell::types::FunctionType = self.context.i32_type().fn_type(
            &[
                inkwell::types::BasicMetadataTypeEnum::IntType(self.context.i32_type()),
                inkwell::types::BasicMetadataTypeEnum::PointerType(
                    self.context
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
        let mut mir = mir::new(
            self.info.clone(),
            self.builtins.clone(),
            self.functions.clone(),
            "main".into(),
            Position {
                line: 0,
                startcol: 0,
                endcol: 0,
                opcol: None,
            },
            self.debug_mir,
        );
        let mut instructions = mir.generate(&vec![]);
        mir::check(&mut mir, &mut instructions, true, &mut HashMap::new());
        //

        self.namespaces.insert(
            realmain,
            Namespace {
                bindings: HashMap::new(),
            },
        );

        self.add_attrs(realmain);

        self.builder.position_at_end(basic_block);
        self.block = Some(basic_block);

        self.cur_fnstate = Some(CurFunctionState {
            cur_block: Some(basic_block),
            returned: false,
            rettp: self.builtins.get(&BasicType::I32).unwrap().clone(),
        });
        self.cur_fn = Some(realmain);

        //

        if !self.cur_fnstate.as_ref().unwrap().returned {
            self.builder
                .build_return(Some(&self.context.i32_type().const_int(0, false)));
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
    debug_mir: bool,
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
        block: None,
        info,
        dibuilder,
        cur_fnstate: None,
        cur_fn: None,
        builtins: HashMap::new(),
        extern_fns: HashMap::new(),
        functions: HashMap::new(),
        namespaces: HashMap::new(),
        flags: flags.clone(),
        optimized: optimize,
        debug_mir,
    };

    let f = OpenOptions::new()
        .write(true)
        .append(true)
        .open("a.mir")
        .expect("Unable to create MIR output file.");
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
