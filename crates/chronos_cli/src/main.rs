mod repl;

use std::env;
use std::fs;
use chronos_lexer::{Lexer, TokenKind};
use chronos_parser::Parser;
use chronos_analyzer::Analyzer;
use chronos_vm::{Compiler, VM};

const VERSION: &str = "0.1.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_banner();
        print_usage();
        return;
    }

    match args[1].as_str() {
        "--help" | "-h" => {
            print_banner();
            print_usage();
        }
        "--version" | "-v" => {
            println!("chronos {}", VERSION);
        }
        "--repl" | "repl" => {
            print_banner();
            repl::start_repl();
        }
        "--lex" => {
            if args.len() < 3 { eprintln!("Error: --lex requires a filename"); return; }
            print_banner();
            run_lexer(&read_file(&args[2]), &args[2]);
        }
        "--parse" => {
            if args.len() < 3 { eprintln!("Error: --parse requires a filename"); return; }
            print_banner();
            run_parser(&read_file(&args[2]), &args[2]);
        }
        "--analyze" => {
            if args.len() < 3 { eprintln!("Error: --analyze requires a filename"); return; }
            print_banner();
            run_analyzer(&read_file(&args[2]), &args[2]);
        }
        "--bytecode" => {
            if args.len() < 3 { eprintln!("Error: --bytecode requires a filename"); return; }
            print_banner();
            run_bytecode(&read_file(&args[2]), &args[2]);
        }
        "--run" => {
            if args.len() < 3 { eprintln!("Error: --run requires a filename"); return; }
            print_banner();
            run_full(&read_file(&args[2]), &args[2]);
        }
        filename => {
            print_banner();
            run_full(&read_file(filename), filename);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Banner & Usage
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn print_banner() {
    println!();
    println!("  ╔══════════════════════════════════════════╗");
    println!("  ║   CHRONOS Programming Language v{}    ║", VERSION);
    println!("  ║   Explicit. Ceremonial. Absolute.        ║");
    println!("  ╚══════════════════════════════════════════╝");
    println!();
}

fn print_usage() {
    println!("  USAGE:");
    println!("    chronos <file.chrn>               Run a CHRONOS program");
    println!("    chronos --run <file.chrn>          Run a CHRONOS program");
    println!("    chronos --repl                     Start interactive REPL");
    println!();
    println!("  PIPELINE STAGES:");
    println!("    chronos --lex <file.chrn>          Show lexer token output");
    println!("    chronos --parse <file.chrn>        Show parser AST output");
    println!("    chronos --analyze <file.chrn>      Show semantic analysis");
    println!("    chronos --bytecode <file.chrn>     Show compiled bytecode");
    println!();
    println!("  OPTIONS:");
    println!("    --help, -h                         Show this help");
    println!("    --version, -v                      Show version");
    println!();
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  File Reader
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn read_file(filename: &str) -> String {
    match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  Error: Cannot read file '{}': {}", filename, e);
            std::process::exit(1);
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Pipeline Helpers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn lex_source(source: &str) -> Option<Vec<chronos_lexer::Token>> {
    match Lexer::new(source).tokenize() {
        Ok(t) => Some(t),
        Err(errors) => {
            eprintln!("  LEXER FAILED [{} error(s)]", errors.len());
            for err in &errors { eprintln!("    ✗ {}", err); }
            None
        }
    }
}

fn parse_tokens(tokens: Vec<chronos_lexer::Token>) -> Option<chronos_parser::Program> {
    match Parser::new(tokens).parse() {
        Ok(p) => Some(p),
        Err(errors) => {
            eprintln!("  PARSER FAILED [{} error(s)]", errors.len());
            for err in &errors { eprintln!("    ✗ {}", err); }
            None
        }
    }
}

fn analyze_program(program: &chronos_parser::Program) -> Option<chronos_analyzer::AnalysisResult> {
    let result = Analyzer::new().analyze(program);
    if result.has_errors() {
        eprintln!("  ANALYZER FAILED [{} error(s)]", result.error_count());
        for err in &result.errors { eprintln!("    ✗ {}", err); }
        None
    } else {
        Some(result)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  Modes
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

fn run_lexer(source: &str, filename: &str) {
    match Lexer::new(source).tokenize() {
        Ok(tokens) => {
            let count = tokens.len();
            println!("  Source : {}", filename);
            println!("  Tokens : {}", count);
            println!();
            println!("    {:<10} {:<30} {}", "LINE:COL", "TOKEN", "LEXEME");
            println!("    {}", "─".repeat(62));
            for token in &tokens {
                if matches!(token.kind, TokenKind::Comment(_)) { continue; }
                let kind_str = format!("{:?}", token.kind);
                let kind_display = if kind_str.len() > 28 {
                    format!("{}...", &kind_str[..25])
                } else {
                    kind_str
                };
                println!("    {:>4}:{:<4}  {:<30} \"{}\"",
                    token.span.line, token.span.column,
                    kind_display, token.lexeme.escape_default());
            }
            println!();
            println!("  ✓ {} tokens generated", count);
        }
        Err(errors) => {
            eprintln!("  LEXER FAILED [{} error(s)]", errors.len());
            for err in &errors { eprintln!("    ✗ {}", err); }
        }
    }
}

fn run_parser(source: &str, filename: &str) {
    let tokens = match lex_source(source) { Some(t) => t, None => return };
    let program = match parse_tokens(tokens) { Some(p) => p, None => return };

    println!("  Source      : {}", filename);
    println!("  Directives  : {}", program.module_directives.len());
    println!("  Requires    : {}", program.require_statements.len());
    println!("  Declarations: {}", program.declarations.len());
    println!();

    for dir in &program.module_directives {
        println!("    ModuleDirective: {}({})", dir.path.join("::"), dir.value);
    }
    for req in &program.require_statements {
        println!("    Require: {} :: [{}]", req.module_path.join("::"), req.imports.join(", "));
    }
    for decl in &program.declarations {
        match decl {
            chronos_parser::Declaration::Contract(c) => {
                println!("    Contract: {} :: [{}] ({} members)",
                    c.name, c.traits.join(", "), c.members.len());
            }
            chronos_parser::Declaration::Function(f) => {
                println!("    Function: {}", f.name);
            }
            chronos_parser::Declaration::Enumeration(e) => {
                println!("    Enum: {} ({} variants)", e.name, e.variants.len());
            }
        }
    }
    println!();
    println!("  ✓ AST generated successfully");
}

fn run_analyzer(source: &str, filename: &str) {
    let tokens = match lex_source(source) { Some(t) => t, None => return };
    let program = match parse_tokens(tokens) { Some(p) => p, None => return };
    let result = Analyzer::new().analyze(&program);

    println!("  Source: {}", filename);
    println!();

    // Contracts
    if !result.symbol_table.contracts.is_empty() {
        println!("  ┌─ Registered Contracts ──────────────────────");
        for (name, info) in &result.symbol_table.contracts {
            println!("  │  contract {} :: [{}]", name, info.traits.join(", "));
            for (fname, ftype) in &info.fields {
                println!("  │    field {}: {}", fname, ftype);
            }
            for method in &info.methods {
                let params: String = method.params.iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect::<Vec<_>>().join(", ");
                println!("  │    fn {}({}) -> {}", method.name, params, method.return_type);
            }
        }
        println!("  └──────────────────────────────────────────────");
        println!();
    }

    if result.has_errors() {
        println!("  ┌─ Errors ({}) ─────────────────────────────────", result.error_count());
        for err in &result.errors { println!("  │  ✗ {}", err); }
        println!("  └──────────────────────────────────────────────");
        println!();
        println!("  ✗ FAILED — {} error(s)", result.error_count());
    } else {
        println!("  ┌─ Analysis ────────────────────────────────────");
        println!("  │  ✓ All types verified");
        println!("  │  ✓ All variables resolved");
        println!("  │  ✓ All scopes valid");
        println!("  │  ✓ Return types checked");
        println!("  └──────────────────────────────────────────────");
        println!();
        println!("  ✓ Semantic analysis passed — 0 errors");
    }
}

fn run_bytecode(source: &str, filename: &str) {
    let tokens = match lex_source(source) { Some(t) => t, None => return };
    let program = match parse_tokens(tokens) { Some(p) => p, None => return };
    let _result = match analyze_program(&program) { Some(r) => r, None => return };

    let compiled = Compiler::new().compile(&program);

    println!("  Source : {}", filename);
    println!("  Chunks : {}", compiled.chunks.len());
    println!("  Entry  : {}", compiled.entry_point.as_deref().unwrap_or("(none)"));
    println!();

    for chunk in &compiled.chunks {
        println!("  ┌─ Chunk: {} ({} ops) ─────────────", chunk.name, chunk.code.len());
        for (i, op) in chunk.code.iter().enumerate() {
            let line = chunk.lines.get(i).unwrap_or(&0);
            println!("  │  {:04}  L{:<4}  {:?}", i, line, op);
        }
        println!("  └────────────────────────────────────────");
        println!();
    }

    println!("  ✓ Bytecode generated — {} chunk(s)", compiled.chunks.len());
}

fn run_full(source: &str, filename: &str) {
    let tokens = match lex_source(source) { Some(t) => t, None => return };
    let program = match parse_tokens(tokens) { Some(p) => p, None => return };
    let _result = match analyze_program(&program) { Some(r) => r, None => return };

    let compiled = Compiler::new().compile(&program);

    println!("  ┌─ Executing: {} ──────────────────────", filename);
    println!("  │");

    let mut vm = VM::new();
    match vm.run(&compiled) {
        Ok(return_val) => {
            // VM output
            let output = vm.get_output();
            if output.is_empty() {
                println!("  │  (no output)");
            } else {
                for line in output {
                    println!("  │  {}", line);
                }
            }
            println!("  │");
            println!("  │  Exit: {}", return_val);
            println!("  └──────────────────────────────────────────");
            println!();
            println!("  ✓ Program executed successfully");
        }
        Err(err) => {
            println!("  │");
            println!("  │  RUNTIME ERROR: {}", err);
            println!("  └──────────────────────────────────────────");
            println!();
            println!("  ✗ Program failed");
        }
    }
}
