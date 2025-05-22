use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use rustyline::Editor;
use rustyline::history::FileHistory;

mod ast;
mod eval;
mod parser;
mod types;

use crate::ast::{Statement, Command};
use crate::eval::eval;
use crate::parser::parse_program;
use crate::types::{Value, EvalError, is_builtin, builtin_env};

fn main() {
    let history_path = get_history_path();
    let mut rl = Editor::<(), FileHistory>::new().unwrap();
    rl.load_history(&history_path).ok();

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
                match parse_program(line) {
                    Ok(stmt) => match stmt {
                        Statement::Let(name, expr) => {
                            if is_builtin(&name) {
                                println!("Error: {}", EvalError::CannotRedefineBuiltin);
                                continue;
                            }
                            match eval(expr, &env) {
                                Ok(value) => {
                                    env.insert(name.clone(), value.clone());
                                    println!("{} = {}", name, value);
                                }
                                Err(e) => println!("Eval error: {}", e),
                            }
                        }
                        Statement::Command(cmd) => match cmd {
                            Command::Info(expr) => match eval(expr, &env) {
                                Ok(val) => {
                                    println!("{}", val.describe());
                                },
                                Err(e) => println!("Eval error: {}", e),
                            },
                        }
                        Statement::Expr(expr) => match eval(expr, &env) {
                            Ok(result) => println!("{}", result),
                            Err(e) => println!("{}", e),
                        },
                    },
                    Err(e) => println!("{}", e),
                }
            }
            Err(_) => break,
        }
    }

    rl.save_history(&history_path).ok();
}

fn get_history_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".blsh");
    fs::create_dir_all(&path).expect("could not create ~/.blsh directory");
    path.push("blsh_history");
    path
}