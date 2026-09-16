#![forbid(unsafe_code)]

mod ast;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod source;
mod token;
mod type_checker;
mod typed_ast;
mod types;
mod value;

use error::KarmaError;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use type_checker::TypeChecker;

const VERSION: &str = "0.2.0";
const MAX_SOURCE_BYTES: u64 = 8 * 1024 * 1024;

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("karma: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_cli() -> Result<(), KarmaError> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        print_help();
        return Ok(());
    }
    if args[0] == "--version" || args[0] == "-V" {
        println!("Karma {VERSION}");
        return Ok(());
    }

    if args[0] == "--check" {
        if args.len() != 2 {
            return Err(KarmaError::runtime("usage: karma --check <file.kr>"));
        }
        check_file(&args[1])?;
        println!("Karma check: OK");
        return Ok(());
    }

    if args.len() != 1 {
        return Err(KarmaError::runtime("usage: karma <file.kr>"));
    }

    run_file(&args[0])
}

fn read_source(path: &str) -> Result<String, KarmaError> {
    let path = Path::new(path);
    if path.extension().and_then(|s| s.to_str()) != Some("kr") {
        return Err(KarmaError::runtime(
            "Karma source files must use the .kr extension",
        ));
    }

    let metadata = fs::metadata(path)
        .map_err(|e| KarmaError::runtime(format!("cannot read '{}': {e}", path.display())))?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(KarmaError::runtime(format!(
            "source file exceeds v0.2 safety limit of {} MiB",
            MAX_SOURCE_BYTES / 1024 / 1024
        )));
    }

    fs::read_to_string(path)
        .map_err(|e| KarmaError::runtime(format!("cannot read '{}': {e}", path.display())))
}

fn compile_frontend(source: &str) -> Result<typed_ast::TypedProgram, KarmaError> {
    let tokens = Lexer::new(source).scan_tokens()?;
    let program = Parser::new(tokens).parse()?;
    TypeChecker::new().check(&program)
}

fn check_file(path: &str) -> Result<(), KarmaError> {
    let source = read_source(path)?;
    compile_frontend(&source)?;
    Ok(())
}

fn run_file(path: &str) -> Result<(), KarmaError> {
    let source = read_source(path)?;
    let typed_program = compile_frontend(&source)?;
    let output = Interpreter::new().run(&typed_program)?;
    for line in output {
        println!("{line}");
    }
    Ok(())
}

fn print_help() {
    println!("Karma Programming Language {VERSION}");
    println!();
    println!("Usage:");
    println!("  karma <file.kr>           Type-check and run a Karma source file");
    println!("  karma --check <file.kr>   Type-check without executing");
    println!("  karma --version           Print version");
    println!("  karma --help              Show this help");
}
