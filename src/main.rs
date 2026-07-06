mod ast;
mod codegen;
mod parser;
mod token;

use logos::Logos;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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

    let lexer = token::Token::lexer(&source);
    let tokens: Vec<(token::Token, std::ops::Range<usize>)> = lexer.spanned().collect();

    if tokens.iter().any(|(t, _)| matches!(t, token::Token::Error)) {
        eprintln!("Lexical error: unexpected characters in source");
        std::process::exit(1);
    }

    // 👇 Передаём source в парсер
    let mut parser = parser::Parser::new(tokens, source.clone());
    let mut program = parser.parse_program().unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    let base_dir: PathBuf = PathBuf::from(source_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut loaded = HashSet::new();

    process_imports(&mut program, &base_dir, &mut loaded)
        .unwrap_or_else(|e| {
            eprintln!("Import error: {}", e);
            std::process::exit(1);
        });

    let c_code = codegen::generate(&program).unwrap_or_else(|e| {
        eprintln!("Compilation error: {}", e);
        std::process::exit(1);
    });

    println!("Generated C code:\n{}", c_code);

    let c_path = "temp_output.c";
    fs::write(c_path, &c_code).expect("Could not write C file");

    let output_name = if cfg!(windows) { "hello.exe" } else { "hello" };
    let status = Command::new("gcc")
        .args(&[c_path, "-o", output_name, "-lm"])
        .status()
        .expect("Failed to run gcc. Is GCC installed and in PATH?");

    if !status.success() {
        eprintln!("GCC compilation failed.");
        std::process::exit(1);
    }

    println!("\x1b[1;32m✅ Compilation successful! Run ./{}\x1b[0m", output_name);
}

fn process_imports(
    program: &mut ast::Program,
    base_dir: &Path,
    loaded: &mut HashSet<String>,
) -> Result<(), String> {
    let imports = std::mem::take(&mut program.imports);
    for import_path in imports {
        if !loaded.insert(import_path.clone()) {
            continue;
        }

        let full_path = base_dir.join(&import_path);
        let source = fs::read_to_string(&full_path)
            .map_err(|e| format!("Cannot read import '{}': {}", import_path, e))?;

        let lexer = token::Token::lexer(&source);
        let tokens: Vec<(token::Token, std::ops::Range<usize>)> = lexer.spanned().collect();

        if tokens.iter().any(|(t, _)| matches!(t, token::Token::Error)) {
            return Err(format!("Lexical error in import '{}'", import_path));
        }

        // 👇 Передаём source в парсер для импорта
        let mut parser = parser::Parser::new(tokens, source.clone());
        let mut imported = parser.parse_program()
            .map_err(|e| format!("Parse error in import '{}': {}", import_path, e))?;

        let imported_base = full_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        process_imports(&mut imported, imported_base, loaded)?;

        merge_programs(program, imported)?;
    }
    Ok(())
}

fn merge_programs(target: &mut ast::Program, imported: ast::Program) -> Result<(), String> {
    for s in imported.structs {
        if target.structs.iter().any(|existing| existing.name == s.name) {
            return Err(format!("Struct '{}' already defined (import conflict)", s.name));
        }
        target.structs.push(s);
    }
    for c in imported.constants {
        if target.constants.iter().any(|existing| existing.name == c.name) {
            return Err(format!("Constant '{}' already defined (import conflict)", c.name));
        }
        target.constants.push(c);
    }
    for f in imported.functions {
        if target.functions.iter().any(|existing| existing.name == f.name) {
            return Err(format!("Function '{}' already defined (import conflict)", f.name));
        }
        target.functions.push(f);
    }
    Ok(())
}
