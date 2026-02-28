mod ast;
mod lexer;
mod parser;
mod codegen;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::codegen::CodeGen;
use inkwell::context::Context;
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: orbitron <file.ot> [output]");
        process::exit(1);
    }

    let src = fs::read_to_string(&args[1])
        .unwrap_or_else(|e| { eprintln!("Cannot read '{}': {}", args[1], e); process::exit(1) })
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    // Lex
    let tokens = Lexer::tokenize(&src)
        .unwrap_or_else(|e| { eprintln!("Lexer error: {}", e); process::exit(1) });

    // Parse
    let program = Parser::new(tokens)
        .parse_program()
        .unwrap_or_else(|e| { eprintln!("Parse error: {}", e); process::exit(1) });

    // Compile to LLVM IR + binary
    let ctx = Context::create();
    let mut cg = CodeGen::new("orbitron", &ctx);
    cg.generate_program(&program);

    let out = args.get(2).map(String::as_str).unwrap_or("orbitron");
    if let Err(e) = cg.save_and_compile(out) {
        eprintln!("Codegen error: {}", e);
        process::exit(1);
    }
}
