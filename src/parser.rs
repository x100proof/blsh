use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::ast::{Expr, Statement, Command};

#[derive(Parser)]
#[grammar = "blsh.pest"]
pub struct BlshParser;

pub fn parse_statement(input: &str) -> Result<Statement, String> {
    let mut pairs = BlshParser::parse(Rule::program, input)
        .map_err(|e| format!("Parse error:\n{}", strip_line_column_prefix(&e.to_string())))?;
    let stmt = pairs.next().unwrap().into_inner().next().unwrap();
    Ok(parse_stmt(stmt))
}

fn parse_stmt(pair: Pair<Rule>) -> Statement {
    match pair.as_rule() {
        Rule::let_stmt => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let expr = parse_expr(inner.next().unwrap());
            Statement::Let(name, expr)
        }
        Rule::command_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap());
            Statement::Command(Command::Info(expr))
        }
        Rule::expr => Statement::Expr(parse_expr(pair)),
        _ => unreachable!(),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    let mut terms = pair.into_inner().map(parse_term).collect::<Vec<_>>();
    let mut expr = terms.remove(0);
    while !terms.is_empty() {
        let rhs = terms.remove(0);
        expr = Expr::Mul(Box::new(expr), Box::new(rhs)); // Handle * by default
    }
    expr
}

fn parse_term(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::scalar => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Scalar(inner.as_str().to_string())
        }
        Rule::coord1 => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Coord1(inner.as_str().to_string())
        }
        Rule::point1 => {
            let mut inner = pair.into_inner();
            let x = parse_coord1_arg(inner.next().unwrap());
            let y = parse_coord1_arg(inner.next().unwrap());
            Expr::Point1(Box::new(x), Box::new(y))
        }
        Rule::point1c => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Point1Compressed(inner.as_str().to_string())
        }
        Rule::sqrt => {
            let inner = pair.into_inner().next().unwrap();
            Expr::Sqrt(Box::new(parse_coord1_arg(inner)))
        }
        Rule::coord1_arg => parse_coord1_arg(pair.into_inner().next().unwrap()),
        Rule::ident => Expr::Var(pair.as_str().to_string()),
        _ => unreachable!(),
    }
}

fn parse_coord1_arg(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::number => Expr::Coord1(pair.as_str().to_string()),
        Rule::ident => Expr::Var(pair.as_str().to_string()),
        _ => unreachable!(),
    }
}

fn strip_line_column_prefix(error: &str) -> String {
    error
        .lines()
        .filter(|line| !line.trim_start().starts_with("-->"))
        .collect::<Vec<_>>()
        .join("\n")
}
