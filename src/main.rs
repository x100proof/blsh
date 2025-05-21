mod ast;
mod eval;
mod parser;
mod types;

use rustyline::Editor;
use rustyline::history::FileHistory;
use std::collections::HashMap;
use crate::parser::parse_statement;
use crate::eval::eval_expr;
use crate::ast::{Statement, Command};
use crate::types::{Value, ParseError};

fn main() {
    let mut rl = Editor::<(), FileHistory>::new().unwrap();
    let _ = rl.load_history("blsh-history.txt");
    let mut env: HashMap<String, Value> = HashMap::new();

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
                            match eval_expr(expr, &env) {
                                Ok(val) => {
                                    env.insert(name.clone(), val.clone());
                                    println!("{} = {}", name, val);
                                }
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                        Statement::Command(Command::Info(expr)) => {
                            match eval_expr(expr, &env) {
                                Ok(val) => {
                                    // Custom print function
                                    println!("Detailed info: {}", val); // FIXME: Add more
                                }
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                        Statement::Expr(expr) => {
                            match eval_expr(expr, &env) {
                                Ok(val) => println!("{}", val),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                    },
                    Err(e) => println!("Parse error: {}", e),
                }
            }
            Err(_) => break,
        }
    }
    let _ = rl.save_history("blsh-history.txt");
}