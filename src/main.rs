use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use rustyline::Editor;
use rustyline::history::FileHistory;
use ansi_term::{Colour, Style};
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
    let prompt = colour_with(Colour::Green.bold(), "blsh> ".to_string());

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
                        Statement::Let(name, expr) => handle_let(name, expr, &mut env),
                        Statement::Command(cmd) => handle_command(cmd, &env),
                        Statement::Expr(expr) => handle_expr(expr, &env),
                    },
                    Err(e) => println!("{}", colour_with(Colour::Red.bold(), e.to_string())),
                }
            }
            Err(_) => break,
        }
    }

    rl.save_history(&history_path).ok();
}


fn colour_with(style: Style, s: String) -> String {
    if supports_colour() {
        style.paint(s).to_string()
    } else {
        s
    }
}

fn handle_let(name: String, expr: ast::Expr, env: &mut HashMap<String, Value>) {
    if is_builtin(&name) {
        println!("Error: {}", EvalError::CannotRedefineBuiltin);
        return;
    }
    match eval(expr, env) {
        Ok(value) => {
            env.insert(name.clone(), value.clone());
            let output = format!("{} = {}", name, value);
            println!("{}", colour_with(Colour::Yellow.normal(), output)); // FIXME: Pretty print
        }
        Err(e) => handle_eval_err(e),
    }
}

fn handle_command(cmd: Command, env: &HashMap<String, Value>) {
    match cmd {
        Command::Info(expr) => match eval(expr, env) {
            Ok(val) => println!("{}", colour_with(Colour::Yellow.normal(), val.describe())),
            Err(e) => handle_eval_err(e),
        }
    }
}

fn handle_expr(expr: ast::Expr, env: &HashMap<String, Value>) {
    match eval(expr, env) {
        Ok(result) => println!("{}", colour_with(Colour::Yellow.normal(), result.to_string())),
        Err(e) => handle_eval_err(e),
    }
}

fn handle_eval_err(e: EvalError) {
    println!("{}", colour_with(Colour::Red.bold(), e.to_string()))
}

fn supports_colour() -> bool {
    atty::is(Stream::Stdout)
}

fn get_history_path() -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".blsh");
    fs::create_dir_all(&path).expect("could not create ~/.blsh directory");
    path.push("blsh_history");
    path
}