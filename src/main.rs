use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use rustyline::Editor;
use rustyline::history::FileHistory;
use ansi_term::Colour::{Green, Yellow};
use atty::Stream;


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
    let color_enabled = supports_color();
    let prompt = if color_enabled {
        format!("{}", Green.bold().paint("blsh> "))
    } else {
        "blsh> ".to_string()
    };

    loop {
        let readline = rl.readline(&prompt);
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
                        Statement::Let(name, expr) => handle_let(name, expr, &mut env, color_enabled),
                        Statement::Command(cmd) => handle_command(cmd, &env, color_enabled),
                        Statement::Expr(expr) => handle_expr(expr, &env, color_enabled),
                    },
                    Err(e) => println!("{}", e),
                }
            }
            Err(_) => break,
        }
    }

    rl.save_history(&history_path).ok();
}

fn handle_let(name: String, expr: ast::Expr, env: &mut HashMap<String, Value>, color_enabled: bool) {
    if is_builtin(&name) {
        println!("Error: {}", EvalError::CannotRedefineBuiltin);
        return;
    }
    match eval(expr, env) {
        Ok(value) => {
            env.insert(name.clone(), value.clone());
            let output = if color_enabled {
                Yellow.paint(value.to_string()).to_string()
            } else {
                value.to_string()
            };
            println!("{} = {}", name, output);
        }
        Err(e) => println!("{}", e),
    }
}

fn handle_command(cmd: Command, env: &HashMap<String, Value>, color_enabled: bool) {
    match cmd {
        Command::Info(expr) => match eval(expr, env) {
            Ok(val) => {
                let desc = val.describe();
                if color_enabled {
                    println!("{}", Yellow.paint(desc));
                } else {
                    println!("{}", desc);
                }
            }
            Err(e) => println!("{}", e),
        }
    }
}

fn handle_expr(expr: ast::Expr, env: &HashMap<String, Value>, color_enabled: bool) {
    match eval(expr, env) {
        Ok(result) => {
            let output = if color_enabled {
                Yellow.paint(result.to_string()).to_string()
            } else {
                result.to_string()
            };
            println!("{}", output);
        }
        Err(e) => println!("{}", e),
    }
}

fn supports_color() -> bool {
    atty::is(Stream::Stdout)
}

fn get_history_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".blsh");
    fs::create_dir_all(&path).expect("could not create ~/.blsh directory");
    path.push("blsh_history");
    path
}