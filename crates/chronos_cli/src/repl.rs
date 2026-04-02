use std::io::{self, Write};
use chronos_lexer::Lexer;
use chronos_parser::Parser;
use chronos_analyzer::Analyzer;
use chronos_vm::{Compiler, VM};

pub fn start_repl() {
    println!("  CHRONOS REPL v0.1.0");
    println!("  Type CHRONOS expressions. Type ':help' for commands.");
    println!("  Type ':quit' to exit.");
    println!();

    let mut history: Vec<String> = Vec::new();
    let mut line_num: usize = 1;

    loop {
        print!("  chronos[{}]> ", line_num);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let input = input.trim().to_string();
        if input.is_empty() {
            continue;
        }

        // REPL komutları
        if input.starts_with(':') {
            match input.as_str() {
                ":quit" | ":q" | ":exit" => {
                    println!("  Goodbye.");
                    break;
                }
                ":help" | ":h" => {
                    print_repl_help();
                    continue;
                }
                ":history" => {
                    println!();
                    for (i, line) in history.iter().enumerate() {
                        println!("  [{}] {}", i + 1, line);
                    }
                    println!();
                    continue;
                }
                ":clear" => {
                    history.clear();
                    line_num = 1;
                    println!("  History cleared.");
                    continue;
                }
                ":tokens" => {
                    if let Some(last) = history.last() {
                        show_tokens(last);
                    } else {
                        println!("  No previous input.");
                    }
                    continue;
                }
                ":ast" => {
                    if let Some(last) = history.last() {
                        show_ast(last);
                    } else {
                        println!("  No previous input.");
                    }
                    continue;
                }
                _ => {
                    println!("  Unknown command: {}", input);
                    println!("  Type ':help' for available commands.");
                    continue;
                }
            }
        }

        // CHRONOS kodu çalıştır
        history.push(input.clone());
        execute_repl_input(&input, line_num);
        line_num += 1;
    }
}

fn execute_repl_input(input: &str, _line_num: usize) {
    // Girdiyi bir contract'a sar
    let wrapped = format!(
        r#"
        #![module::entry(main)]
        contract Main :: EntryPoint {{
            fn main() -> Void {{
                {}
            }}
        }}
        "#,
        input
    );

    // Lex
    let tokens = match Lexer::new(&wrapped).tokenize() {
        Ok(t) => t,
        Err(errors) => {
            for err in &errors {
                println!("  LEX ERROR: {}", err);
            }
            return;
        }
    };

    // Parse
    let program = match Parser::new(tokens).parse() {
        Ok(p) => p,
        Err(errors) => {
            for err in &errors {
                println!("  PARSE ERROR: {}", err);
            }
            return;
        }
    };

    // Analyze
    let result = Analyzer::new().analyze(&program);
    if result.has_errors() {
        for err in &result.errors {
            println!("  ANALYSIS ERROR: {}", err);
        }
        return;
    }

    // Compile
    let compiled = Compiler::new().compile(&program);

    // Run
    let mut vm = VM::new();
    match vm.run(&compiled) {
        Ok(_val) => {
            // Output varsa göster
            let output = vm.get_output();
            for line in output {
                println!("  {}", line);
            }
        }
        Err(err) => {
            println!("  RUNTIME ERROR: {}", err);
        }
    }
}

fn show_tokens(input: &str) {
    let wrapped = format!(
        r#"#![module::entry(main)]
        contract Main :: EntryPoint {{
            fn main() -> Void {{
                {}
            }}
        }}"#,
        input
    );

    match Lexer::new(&wrapped).tokenize() {
        Ok(tokens) => {
            println!();
            for token in &tokens {
                if matches!(token.kind, chronos_lexer::TokenKind::Comment(_) | chronos_lexer::TokenKind::EOF) {
                    continue;
                }
                println!("  {:>4}:{:<4}  {:?}", token.span.line, token.span.column, token.kind);
            }
            println!();
        }
        Err(errors) => {
            for err in &errors {
                println!("  ERROR: {}", err);
            }
        }
    }
}

fn show_ast(input: &str) {
    let wrapped = format!(
        r#"#![module::entry(main)]
        contract Main :: EntryPoint {{
            fn main() -> Void {{
                {}
            }}
        }}"#,
        input
    );

    let tokens = match Lexer::new(&wrapped).tokenize() {
        Ok(t) => t,
        Err(errors) => {
            for err in &errors { println!("  ERROR: {}", err); }
            return;
        }
    };

    match Parser::new(tokens).parse() {
        Ok(program) => {
            println!();
            for decl in &program.declarations {
                println!("  {:?}", decl);
            }
            println!();
        }
        Err(errors) => {
            for err in &errors { println!("  ERROR: {}", err); }
        }
    }
}

fn print_repl_help() {
    println!();
    println!("  ┌─ CHRONOS REPL Commands ─────────────────────");
    println!("  │  :help      Show this help message");
    println!("  │  :quit      Exit the REPL");
    println!("  │  :history   Show input history");
    println!("  │  :clear     Clear history");
    println!("  │  :tokens    Show tokens of last input");
    println!("  │  :ast       Show AST of last input");
    println!("  └──────────────────────────────────────────────");
    println!();
    println!("  Examples:");
    println!("    let x: Int32 = 42i32;");
    println!("    writer.emit(payload: \"Hello!\");");
    println!();
}
