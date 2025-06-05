use ansi_term::{Colour, Style};
use atty::Stream;
use env_logger::Builder;
use log::{debug, LevelFilter};
use miette::{self, Result};
use rustyline::Editor;
use rustyline::history::FileHistory;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod ast;
mod eval;
mod eval_error;
mod parser;
mod types;


use crate::ast::{Command, SpanInfo, Statement};
use crate::eval::eval;
use crate::eval_error::EvalError;
use crate::parser::parse_program;
use crate::types::{builtin_env, is_reserved_word, pretty_let_to_string, Value};



// Debug configuration
struct DebugConfig {
    enabled: bool,
}

impl DebugConfig {
    fn new() -> Self {
        // Check environment variable for debug mode
        let debug_env = std::env::var("BLSH_DEBUG").unwrap_or_else(|_| "0".to_string());
        let enabled = debug_env == "1" || debug_env.to_lowercase() == "true";

        // Initialize logger if debug is enabled
        if enabled {
            let mut builder = Builder::new();
            builder
                .format(|buf, record| {
                    writeln!(
                        buf,
                        "[{}] {} - {}",
                        record.level(),
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.args()
                    )
                })
                .filter(Some("blsh"), LevelFilter::Debug) // default to just `blsh` crate
                .parse_default_env() // Use RUST_LOG to override
                .target(env_logger::Target::Stderr)
                .init();

            debug!("Debug logging enabled");
        }

        Self { enabled }
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

fn main() -> Result<()> {
    // Initialize miette
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new()
        .context_lines(1)
        .tab_width(2)
        .unicode(true)
        .build()))).unwrap();

    // Initialize debug configuration
    let debug_config = DebugConfig::new();

    let history_path = get_history_path();
    let mut rl = Editor::<(), FileHistory>::new().unwrap();
    rl.load_history(&history_path).ok();

    let mut env: HashMap<String, Value> = builtin_env();
    let prompt = colour_with(Colour::Green.bold(), "blsh> ".to_string());

    if debug_config.is_enabled() {
        debug!("BLSH started with debug logging enabled");
    }

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
                let input = line.to_string();
                match parse_program(line) {
                    Ok(stmt) => match stmt {
                        Statement::Let(name, expr, span_info) => handle_let(input.clone(), name, expr, span_info, &mut env),
                        Statement::Command(cmd) => handle_command(input.clone(), cmd, &env),
                        Statement::Expr(expr) => handle_expr(input.clone(), expr, &env),
                    },
                    Err(e) => handle_parse_err(input, e),
                }
            }
            Err(_) => break,
        }
    }

    rl.save_history(&history_path).ok();

    Ok(())
}


fn colour_with(style: Style, s: String) -> String {
    if supports_colour() {
        style.paint(s).to_string()
    } else {
        s
    }
}

fn handle_let(input: String, name: String, expr: ast::Expr, ident_meta: SpanInfo, env: &mut HashMap<String, Value>) {
    if is_reserved_word(&name) {
          eprintln!("{:?}",
             miette::Report::new(EvalError::CannotRedefineBuiltin{ ident: ident_meta.miette_span()}).with_source_code(input));
        return;
    }
    match eval(&expr, env) {
        Ok(value) => {
            env.insert(name.clone(), value.clone());
            let output = pretty_let_to_string(&name, &value);
            println!("{}", output);
        }
        Err(e) => handle_eval_err(input, e),
    }
}

fn handle_command(input: String, cmd: Command, env: &HashMap<String, Value>) {
    match cmd {
        Command::Info(expr) => match eval(&expr, env) {
            Ok(val) => println!("{}", val.describe()),
            Err(e) => handle_eval_err(input, e),
        }
    }
}

fn handle_expr(input: String, expr: ast::Expr, env: &HashMap<String, Value>) {
    debug!("Evaluating expression: {:?}", expr);
    match eval(&expr, env) {
        Ok(result) => println!("{}", result.to_string()),
        Err(e) => handle_eval_err(input, e),
    }
}

fn handle_eval_err(input: String, e: EvalError) {
    eprintln!("{:?}", miette::Report::new(e).with_source_code(input));
}

fn handle_parse_err(input: String, e: parser::ParseError) {
    eprintln!("{:?}", miette::Report::new(e).with_source_code(input));
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
