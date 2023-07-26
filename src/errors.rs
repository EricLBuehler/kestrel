use colored::Colorize;

#[derive(Clone)]
pub enum ErrorType {
    InvalidTok,
    InvalidLiteralForRadix,
    InvalidFlag,
    TypeMismatch,
    BindingNotFound,
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
    println!("{}", header.red().bold());
    println!("{}", location.red());

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
    println!("{} | {}", linestr, snippet);
    println!("{} | {}", " ".repeat(linestr.len()), arrows.green());
    std::process::exit(1);
}

pub fn raise_error_no_pos(error: &str, errtp: ErrorType) -> ! {
    let header: String = format!("error[E{:0>3}]: {}", errtp as u8 + 1, error);
    println!("{}", header.red().bold());
    std::process::exit(1);
}
