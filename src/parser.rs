use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use num_bigint::BigUint;
use crate::ast::{Expr, Statement, Command};

use pest::error::{Error as PestError, ErrorVariant};
use pest::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub pest: PestError<Rule>,
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    InvalidNumber,
    NumberTooLarge,
    UnexpectedToken,
    ReservedWord,
    BareNumber,
}

// Built-in identifiers are not the same as reserved words
const RESERVED_WORDS: &[&str] = &[
    "let", "info", "sqrt", "point1", "point2", "coord1", "coord2", "scalar",
];


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        fn clean_parse_error(error: &str) -> String {
            let lines: Vec<&str> = error.lines().collect();

            if lines.len() <= 2 {
                return error.to_string(); // Nothing to clean
            }

            lines[1..lines.len() - 1]  // Skip first and last lines
                .iter()
                .map(|line| {
                    if let Some(idx) = line.find('|') {
                        line[idx + 1..].to_string()
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n")
        }


        let msg = match self.kind {
            ParseErrorKind::InvalidNumber => "Number not valid hex",
            ParseErrorKind::NumberTooLarge => "Number too large",
            ParseErrorKind::UnexpectedToken => "Unexpected end of input or unexpected token",
            ParseErrorKind::ReservedWord => "cannot use reserved word as identifier",
            ParseErrorKind::BareNumber => "Bare number: use a constructor like scalar(...)",
        };
        writeln!(f, "{}\n{}", msg, clean_parse_error(&format!("{}", self.pest)))
    }
}


fn parse_error(kind: ParseErrorKind, span: Span) -> ParseError {
    ParseError {
        kind: kind,
        pest: PestError::new_from_span(ErrorVariant::ParsingError { positives: vec![], negatives: vec![] }, span)
    }
}



#[derive(Parser)]
#[grammar = "blsh.pest"]

pub struct BlshParser;

pub fn parse_program(input: &str) -> Result<Statement, ParseError> {
    let mut pairs = BlshParser::parse(Rule::program, input)
        .map_err(|e| ParseError { kind: ParseErrorKind::UnexpectedToken, pest: e })?;
    let pair = pairs.next().unwrap();
    let stmt = pair.into_inner().next().unwrap(); // Now Rule::statement
    parse_statement(stmt.into_inner().next().unwrap())
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let span = pair.as_span();
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
        _ => Err(parse_error(ParseErrorKind::UnexpectedToken, span)),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::expr => {
            let mut inner = pair.into_inner();
//            println!("*** expr = {:?}", inner);
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(op_pair) = inner.next() {
                let rhs = parse_expr(inner.next().unwrap())?;
                lhs = match op_pair.as_str() {
                    "+" => Expr::Add(Box::new(lhs), Box::new(rhs)),
                    "-" => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                    _ => return Err(parse_error(ParseErrorKind::UnexpectedToken, span)),
                };
            }
            Ok(lhs)
        }
        Rule::mul_expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(rhs) = inner.next() {
                lhs = Expr::Mul(Box::new(lhs), Box::new(parse_expr(rhs)?));
            }
            Ok(lhs)
        }
        Rule::unary_expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            if first.as_rule() == Rule::unary_expr {
                Ok(Expr::Neg(Box::new(parse_expr(first)?)))
            } else {
                parse_expr(first)
            }
        }
        Rule::primary => {
            let mut inner = pair.into_inner();
            let child = inner.next().unwrap();
            match child.as_rule() {
                Rule::expr => parse_expr(child),
                _ => parse_primary(child),
            }
        }
        _ => parse_primary(pair),
    }
}

fn parse_primary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::scalar => {
            let inner = pair.into_inner().next().unwrap();
//            println!("*** scalar inner {:?}", inner);
            let bytes = parse_number::<32>(inner)?;
            Ok(Expr::Scalar(bytes))
        }
        Rule::coord1 => {
            let inner = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner)?;
            Ok(Expr::Coord1(bytes))
        }
        Rule::point1 => {
            let mut inner = pair.into_inner();
//            println!("*** point1 inner {:?}", inner);
//            println!("+++ parse x");
            let x = parse_number_or_ident(inner.next().unwrap())?;
//            println!("+++ parse y");
            let y = parse_number_or_ident(inner.next().unwrap())?;
            Ok(Expr::Point1(Box::new(x), Box::new(y)))
        }
        Rule::point1c => {
            let inner = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner)?;
            Ok(Expr::Point1Compressed(bytes))
        }
        Rule::sqrt => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            Ok(Expr::Sqrt(Box::new(arg)))
        }
        Rule::ident => Ok(Expr::Var(parse_ident(pair)?)),
        Rule::number => Err(parse_error(ParseErrorKind::BareNumber, span)),
        _ => Err(parse_error(ParseErrorKind::UnexpectedToken, span)),
    }
}


fn parse_number_or_ident(pair: Pair<Rule>) -> Result<Expr, ParseError> {
//    println!("*** number_or_ident pair {:?}", pair);
//    println!("*** number_or_ident_as_rule {:?}", pair.as_rule());
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::number => {
            let bytes = parse_number::<48>(pair)?;
            Ok(Expr::Coord1(bytes))
        }
        Rule::ident => Ok(Expr::Var(parse_ident(pair)?)),
        _ => Err(parse_error(ParseErrorKind::UnexpectedToken, span)),
    }
}

fn parse_ident(pair: Pair<Rule>) -> Result<String, ParseError> {
    let span = pair.as_span();
    let s = pair.as_str();
    if RESERVED_WORDS.contains(&s) {
        Err(parse_error(ParseErrorKind::ReservedWord, span))
    } else {
        Ok(s.to_string())
    }
}


fn parse_number<const N: usize>(pair: Pair<Rule>) -> Result<[u8; N], ParseError> {
    let span = pair.as_span();
    let err_invalid_number = parse_error(ParseErrorKind::InvalidNumber, span);
    let s = pair.as_str();
    let n = if let Some(hex) = s.strip_prefix("0x") {
        BigUint::parse_bytes(hex.as_bytes(), 16).ok_or(err_invalid_number.clone())?
    } else {
        BigUint::parse_bytes(s.as_bytes(), 10).ok_or(err_invalid_number.clone())?
    };
    let bytes = n.to_bytes_be();
    if bytes.len() > N {
        return Err(parse_error(ParseErrorKind::NumberTooLarge, span));
    }
    let mut padded = [0u8; N];
    padded[N - bytes.len()..].copy_from_slice(&bytes);
    Ok(padded)
}

