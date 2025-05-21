use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use num_bigint::BigUint;
use crate::ast::{Expr, Statement, Command};
use crate::types::ParseError;

#[derive(Parser)]
#[grammar = "blsh.pest"]
pub struct BlshParser;

pub fn parse_statement(input: &str) -> Result<Statement, ParseError> {
    let mut pairs = BlshParser::parse(Rule::program, input)
        .map_err(|_| ParseError::UnexpectedToken)?;
    let stmt = pairs.next().unwrap().into_inner().next().unwrap();
    parse_stmt(stmt)
}

fn parse_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        Rule::let_stmt => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let expr = parse_expr(inner.next().unwrap())?;
            Ok(Statement::Let(name, expr))
        }
        Rule::command_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap())?;
            Ok(Statement::Command(Command::Info(expr)))
        }
        Rule::expr => Ok(Statement::Expr(parse_expr(pair)?)),
        _ => Err(ParseError::UnexpectedToken),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner().map(parse_term).collect::<Result<Vec<_>, _>>()?;
    let mut expr = inner.remove(0);
    while !inner.is_empty() {
        let rhs = inner.remove(0);
        expr = Expr::Mul(Box::new(expr), Box::new(rhs));
    }
    Ok(expr)
}

fn parse_term(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::scalar => {
            let inner = pair.into_inner().next().unwrap();
            let bytes = parse_number::<32>(&inner.as_str())?;
            Ok(Expr::Scalar(bytes))
        }
        Rule::coord1 => {
            let inner = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(&inner.as_str())?;
            Ok(Expr::Coord1(bytes))
        }
        Rule::point1 => {
            let mut inner = pair.into_inner();
            let x = parse_coord1_arg(inner.next().unwrap())?;
            let y = parse_coord1_arg(inner.next().unwrap())?;
            Ok(Expr::Point1(Box::new(x), Box::new(y)))
        }
        Rule::point1c => {
            let inner = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(&inner.as_str())?;
            Ok(Expr::Point1Compressed(bytes))
        }
        Rule::sqrt => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_coord1_arg(inner)?;
            Ok(Expr::Sqrt(Box::new(arg)))
        }
        Rule::coord1_arg => parse_coord1_arg(pair.into_inner().next().unwrap()),
        Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
        Rule::number => Err(ParseError::BareNumber),
        _ => Err(ParseError::UnexpectedToken),
    }
}

fn parse_coord1_arg(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::number => {
            let bytes = parse_number::<48>(pair.as_str())?;
            Ok(Expr::Coord1(bytes))
        }
        Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
        _ => Err(ParseError::UnexpectedToken),
    }
}

fn parse_number<const N: usize>(s: &str) -> Result<[u8; N], ParseError> {
    let n = if let Some(hex) = s.strip_prefix("0x") {
        BigUint::parse_bytes(hex.as_bytes(), 16).ok_or(ParseError::InvalidNumber)?
    } else {
        BigUint::parse_bytes(s.as_bytes(), 10).ok_or(ParseError::InvalidNumber)?
    };
    let bytes = n.to_bytes_be();
    if bytes.len() > N {
        return Err(ParseError::InvalidNumber);
    }
    let mut padded = [0u8; N];
    padded[N - bytes.len()..].copy_from_slice(&bytes);
    Ok(padded)
}