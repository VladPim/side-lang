mod ast;
mod codegen;
mod parser;
mod token;

use logos::Logos;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.sd>", args[0]);
        std::process::exit(1);
    }

    let source_path = &args[1];
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|_| panic!("Could not read source file: {}", source_path));

    // ---------- Лексер ----------
    let lexer = token::Token::lexer(&source);
    let tokens: Vec<(token::Token, std::ops::Range<usize>)> = lexer.spanned().collect();

    // Проверка на лексические ошибки
    if tokens.iter().any(|(t, _)| matches!(t, token::Token::Error)) {
        eprintln!("Lexical error: unexpected characters in source");
        std::process::exit(1);
    }

    // ---------- Парсер ----------
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    // ---------- Генерация C-кода ----------
    let c_code = codegen::generate(&program);
    println!("Generated C code:\n{}", c_code);

    let c_path = "temp_output.c";
    fs::write(c_path, &c_code).expect("Could not write C file");

    // ---------- Компиляция C в .exe ----------
    let output_name = if cfg!(windows) { "hello.exe" } else { "hello" };
    let status = Command::new("gcc")
        .args(&[c_path, "-o", output_name])
        .status()
        .expect("Failed to run gcc. Is GCC installed and in PATH?");

    if !status.success() {
        eprintln!("GCC compilation failed.");
        std::process::exit(1);
    }

    println!("\x1b[1;32m✅ Compilation successful! Run ./{}\x1b[0m", output_name);
}