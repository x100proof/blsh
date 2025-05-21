

use std::collections::HashMap;
use rustyline::Editor;
use rustyline::history::FileHistory;

mod ast;
mod eval;
mod parser;
mod types;

use crate::ast::{Statement, Command};
use crate::eval::eval_expr;
use crate::parser::parse_statement;
use crate::types::{Value, EvalError, is_builtin, builtin_env};

fn main() {
    let mut rl = Editor::<(), FileHistory>::new().unwrap();
    let _ = rl.load_history("blsh-history.txt");

    let mut env: HashMap<String, Value> = builtin_env();

    loop {
        let readline = rl.readline("blsh> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }
                match parse_statement(line) {
                    Ok(stmt) => match stmt {
                        Statement::Let(name, expr) => {
                            if is_builtin(&name) {
                                println!("Error: {}", EvalError::CannotRedefineBuiltin);
                                continue;
                            }
                            match eval_expr(expr, &env) {
                                Ok(value) => {
                                    env.insert(name.clone(), value.clone());
                                    println!("{} = {}", name, value);
                                }
                                Err(e) => println!("Eval error: {}", e),
                            }
                        }
                        Statement::Command(cmd) => match cmd {
                            ast::Command::Info(expr) => match eval_expr(expr, &env) {
                                Ok(val) => {
                                    println!("{}", val.describe());
                                },
                                Err(e) => println!("Eval error: {}", e),
                            },
                        }
                        Statement::Expr(expr) => match eval_expr(expr, &env) {
                            Ok(result) => println!("{}", result),
                            Err(e) => println!("Eval error: {}", e),
                        },
                    },
                    Err(e) => println!("Parse error: {}", e),
                }
            }
            Err(_) => break,
        }
    }

    let _ = rl.save_history("blsh-history.txt");
}