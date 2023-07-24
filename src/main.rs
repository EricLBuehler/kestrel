use clap::Parser;
use codegen::generate_code;
use errors::{raise_error_no_pos, ErrorType};
use utils::FileInfo;

mod errors;
mod types;
mod utils;

mod lexer;

mod parser;

mod codegen;

//Version: major.minor
#[derive(Parser, Debug)]
#[command(author, version = "0.1.0", about, long_about = None)]
struct Args {
    /// File to execute
    #[arg(required = true, name = "file")]
    file: String,

    /// File to execute
    #[arg(default_value = "", name = "N")]
    no: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NoFlag {
    OUChecks,
}

fn main() {
    let args = Args::parse();

    let tmp_no_flags = args.no.split("--N").collect::<Vec<_>>();
    let no_flags: Vec<_> = if tmp_no_flags.len() != 1 {
        tmp_no_flags[1]
            .split(' ')
            .map(|x| {
                if x == "ou-checks" {
                    NoFlag::OUChecks
                } else {
                    raise_error_no_pos(
                        &format!("'{x}' was not recognized as a valid 'no flag'"),
                        ErrorType::InvalidNoFlag,
                    );
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let res = std::fs::read_to_string(&args.file);
    let file_data = match res {
        Ok(_) => res.unwrap(),
        Err(_) => {
            println!("File '{}' is unable to be opened or read.", args.file);
            return;
        }
    };

    let data = file_data.chars();

    let mut file_info = FileInfo {
        data: data.clone(),
        name: args.file.clone(),
        dir: String::from("."),
    };

    let keywords = vec!["let".into()];
    let mut lexer = lexer::new(&mut file_info);
    let (len, tokens) = lexer::generate_tokens(&mut lexer, &keywords);
    lexer::print_tokens(len, &tokens);

    let mut parser = parser::Parser::new(tokens, &file_info);
    let ast = parser.generate_ast();

    generate_code(&args.file, &args.file, ast, &file_info, no_flags)
        .expect("Code generation error.");
}
