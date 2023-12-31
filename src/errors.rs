use colored::Colorize;

use crate::utils::{FileInfo, Position};

#[derive(Clone)]
pub enum ErrorType {
    InvalidTok,
    InvalidLiteralForRadix,
    InvalidFlag,
    TypeMismatch,
    BindingNotFound,
    DuplicateFlag,
    MovedBinding,
    BindingNotMutable,
    MultipleImmutableReferences,
    TraitNotImplemented,
    InvalidSpecifiedNumericType,
    NestedFnDef,
    MultipleFunctionDefinitions,
    NonModuleLevelStatement,
    FunctionNotFound,
    TypeNotFound,
    ReturnReference,
    DerefNonref,
    CannotMoveOutOfBinding,
    CannotMoveOutOfNonCopy,
    FloatingElse,
    FloatingElif,
    ValueNotLiveEnough,
    MissingElseClause,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", repr_err(self.clone()))
    }
}

pub fn repr_err(tp: ErrorType) -> &'static str {
    match tp {
        ErrorType::InvalidTok => "invalid token encountered",
        ErrorType::InvalidLiteralForRadix => "invalid literal for radix provided",
        ErrorType::InvalidFlag => "invalid flag passed",
        ErrorType::TypeMismatch => "type mismatch",
        ErrorType::BindingNotFound => "binding not found",
        ErrorType::DuplicateFlag => "duplicate flag passed",
        ErrorType::MovedBinding => "binding was moved",
        ErrorType::BindingNotMutable => "binding not mutable",
        ErrorType::MultipleImmutableReferences => "multiple immutable references",
        ErrorType::TraitNotImplemented => "trait not implemented",
        ErrorType::InvalidSpecifiedNumericType => "invalid specified numeric type",
        ErrorType::NestedFnDef => "nested function definitions are disallowed",
        ErrorType::MultipleFunctionDefinitions => "multiple function definitions are disallowed",
        ErrorType::NonModuleLevelStatement => "unexpected module level statement",
        ErrorType::FunctionNotFound => "function not found",
        ErrorType::TypeNotFound => "type not found",
        ErrorType::ReturnReference => "references cannot be returned",
        ErrorType::DerefNonref => "references cannot be dereferenced",
        ErrorType::CannotMoveOutOfBinding => "cannot move non Copy-able type out of binding",
        ErrorType::CannotMoveOutOfNonCopy => "cannot move out of non Copy-able type",
        ErrorType::FloatingElse => "floating else is not allowed here",
        ErrorType::FloatingElif => "floating elif is not allowed here",
        ErrorType::ValueNotLiveEnough => "value does not live long enough",
        ErrorType::MissingElseClause => "missing else clause",
    }
}

#[derive(Clone)]
pub enum WarningType {}

impl std::fmt::Display for WarningType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", repr_warn(self.clone()))
    }
}

pub fn repr_warn(tp: WarningType) -> &'static str {
    match tp {}
}

pub fn raise_error(
    error: &str,
    errtp: ErrorType,
    pos: &crate::utils::Position,
    info: &crate::utils::FileInfo,
) -> ! {
    let header: String = format!("error[E{:0>3}]: {}", errtp as u8 + 1, error);
    let location: String = format!("{}:{}:{}", info.name, pos.line + 1, pos.startcol + 1);
    eprintln!("{}", header.red().bold());
    eprintln!("{}", location.red());

    let collected = info.data.clone().collect::<Vec<_>>();
    let lines = Vec::from_iter(collected.split(|num| *num == '\n'));

    let snippet: String = format!(
        "{}",
        String::from_iter(lines.get(pos.line).unwrap().to_vec()).blue()
    );

    let mut arrows: String = String::new();
    for idx in 0..snippet.len() {
        if idx >= pos.startcol && idx < pos.endcol {
            arrows += "^";
        } else {
            arrows += " ";
        }
    }
    let linestr = (pos.line + 1).to_string().blue().bold();
    eprintln!("{} | {}", linestr, snippet);
    eprintln!("{} | {}", " ".repeat(linestr.len()), arrows.green());
    std::process::exit(1);
}

pub fn raise_error_no_pos(error: &str, errtp: ErrorType) -> ! {
    let header: String = format!("error[E{:0>3}]: {}", errtp as u8 + 1, error);
    println!("{}", header.red().bold());
    std::process::exit(1);
}

pub fn raise_error_multi(
    err: Vec<String>,
    errtp: ErrorType,
    pos: Vec<Option<&Position>>,
    info: &FileInfo,
) -> ! {
    for (i, (error, pos)) in std::iter::zip(&err, pos).enumerate() {
        if pos.is_none() {
            if i != 0 {
                let header: String = error.to_string();
                eprintln!("{}", header.yellow());
                continue;
            } else {
                unreachable!();
            }
        }
        let pos = pos.unwrap();

        let location: String = format!("{}:{}:{}", info.name, pos.line + 1, pos.startcol + 1);
        if i == 0 {
            let header: String = format!("error[E{:0>3}]: {}", errtp.clone() as u8 + 1, error);
            eprintln!("{}", header.red().bold());
        } else {
            let header: String = error.to_string();
            eprintln!("{}", header.yellow());
        }
        eprintln!("{}", location.red());

        let collected = info.data.clone().collect::<Vec<_>>();
        let lines = Vec::from_iter(collected.split(|num| *num == '\n'));

        let snippet: String = format!(
            "{}",
            String::from_iter(lines.get(pos.line).unwrap().to_vec()).blue()
        );

        let mut arrows: String = String::new();
        for idx in 0..snippet.len() {
            if idx >= pos.startcol && idx < pos.endcol {
                arrows += "^";
            } else {
                arrows += " ";
            }
        }
        let linestr = (pos.line + 1).to_string().blue().bold();
        eprintln!("{} | {}", linestr, snippet);
        eprintln!("{} | {}", " ".repeat(linestr.len()), arrows.green());
    }
    std::process::exit(1);
}
