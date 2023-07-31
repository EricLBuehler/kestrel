use clap::{ArgAction, Parser};
use codegen::generate_code;
use errors::{raise_error_no_pos, ErrorType};
use utils::FileInfo;

mod errors;
mod types;
mod utils;

mod lexer;

mod parser;

mod codegen;

mod mir;

//Version: major.minor
#[derive(Parser, Debug)]
#[command(author, version = "0.1.0", about, long_about = None)]
struct Args {
    /// File to execute
    #[clap(name = "name", required = true)]
    name: String,

    /// Flags to exclude, no-ou-checks (over and underflow runtime checkss) or sanitize (sanitize address, thread, and memory)
    #[clap(use_value_delimiter=true, value_delimiter=' ', action=ArgAction::Append, long, short)]
    flags: Option<Vec<String>>,

    #[clap(long, short, action)]
    optimize: bool,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum Flags {
    NoOUChecks,
    Sanitize,
}

fn main() {
    let args = Args::parse();

    let mut flags = Vec::new();

    if args.flags.is_some() {
        for flag in args.flags.unwrap() {
            if flag == "no-ou-checks" {
                if flags.contains(&Flags::NoOUChecks) {
                    raise_error_no_pos(
                        &format!("'{flag}' was specified multiple times"),
                        ErrorType::DuplicateFlag,
                    );
                }
                flags.push(Flags::NoOUChecks);
            } else if flag == "sanitize" {
                if flags.contains(&Flags::Sanitize) {
                    raise_error_no_pos(
                        &format!("'{flag}' was specified multiple times"),
                        ErrorType::DuplicateFlag,
                    );
                }
                flags.push(Flags::Sanitize);
            } else {
                raise_error_no_pos(
                    &format!("'{flag}' was not recognized as a valid flag"),
                    ErrorType::InvalidFlag,
                );
            }
        }
    }

    let res = std::fs::read_to_string(&args.name);
    let file_data = match res {
        Ok(_) => res.unwrap(),
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", args.name);
            return;
        }
    };

    let data = file_data.chars();

    let mut file_info = FileInfo {
        data: data.clone(),
        name: args.name.clone(),
        dir: String::from("."),
    };

    let keywords = vec!["let".into(), "mut".into()];
    let mut lexer = lexer::new(&mut file_info);
    let (_, tokens) = lexer::generate_tokens(&mut lexer, &keywords);

    let mut parser = parser::Parser::new(tokens, &file_info);
    let ast = parser.generate_ast();

    generate_code(
        &args.name,
        &args.name,
        ast,
        &file_info,
        flags,
        args.optimize,
    )
    .expect("Code generation error.");
}
