#![forbid(unsafe_code)]

mod ast;
mod environment;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod value;

use error::KarmaError;
use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const VERSION: &str = "0.1.0";
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
    if args.len() != 1 {
        return Err(KarmaError::runtime("usage: karma <file.kr>"));
    }

    run_file(&args[0])
}

fn run_file(path: &str) -> Result<(), KarmaError> {
    let path = Path::new(path);
    if path.extension().and_then(|s| s.to_str()) != Some("kr") {
        return Err(KarmaError::runtime("Karma source files must use the .kr extension"));
    }

    let metadata = fs::metadata(path)
        .map_err(|e| KarmaError::runtime(format!("cannot read '{}': {e}", path.display())))?;
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(KarmaError::runtime(format!(
            "source file exceeds v0.1 safety limit of {} MiB",
            MAX_SOURCE_BYTES / 1024 / 1024
        )));
    }

    let source = fs::read_to_string(path)
        .map_err(|e| KarmaError::runtime(format!("cannot read '{}': {e}", path.display())))?;
    let tokens = Lexer::new(&source).scan_tokens()?;
    let program = Parser::new(tokens).parse()?;
    let output = Interpreter::new().run(&program)?;
    for line in output {
        println!("{line}");
    }
    Ok(())
}

fn print_help() {
    println!("Karma Programming Language {VERSION}");
    println!();
    println!("Usage:");
    println!("  karma <file.kr>   Run a Karma source file");
    println!("  karma --version   Print version");
    println!("  karma --help      Show this help");
}
