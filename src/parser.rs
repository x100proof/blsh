use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use num_bigint::BigUint;
use crate::ast::{Expr, Statement, Command, ExprMeta};

use pest::error::{Error as PestError, ErrorVariant};
use pest::Span;
use log::debug;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub pest: PestError<Rule>,
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    InvalidHex,
    MustBeExactlyHexDigits(usize, usize),
    NumberTooLarge(String),
    UnexpectedToken,
    ReservedWord,
    CannotBeDecimal,

}

// Built-in identifiers are not the same as reserved words
const RESERVED_WORDS: &[&str] = &[
    "let", "info", "sqrt", "point1", "point2", "coord1", "coord2", "scalar", "embed",
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


        let msg = match &self.kind {
            ParseErrorKind::InvalidHex => "Number not valid hex",
            ParseErrorKind::NumberTooLarge(hex_str) => &format!("Must be less than {}", hex_str),
            ParseErrorKind::UnexpectedToken => "Unexpected end of input or unexpected token",
            ParseErrorKind::ReservedWord => "cannot use reserved word as identifier",
            ParseErrorKind::CannotBeDecimal => &format!("Literal cannot be decimal"),
            // FIXME. Better error message based on type
            ParseErrorKind::MustBeExactlyHexDigits(expected, actual) =>
              &format!("Literal must be exactly {expected} hex digits long. Currently {actual} digits."),
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
    let source = span.as_str().to_string();
    debug!("Parsing expression: {}", source);
    
    match pair.as_rule() {
        Rule::expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(op_pair) = inner.next() {
                let rhs = parse_expr(inner.next().unwrap())?;
                let op_str = op_pair.as_str();
                let meta = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
                
                lhs = match op_str {
                    "+" => Expr::Add(Box::new(lhs), Box::new(rhs), meta),
                    "-" => Expr::Sub(Box::new(lhs), Box::new(rhs), meta),
                    _ => return Err(parse_error(ParseErrorKind::UnexpectedToken, span)),
                };
            }
            Ok(lhs)
        }
        Rule::mul_expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(rhs) = inner.next() {
                let meta = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
                lhs = Expr::Mul(Box::new(lhs), Box::new(parse_expr(rhs)?), meta);
            }
            Ok(lhs)
        }
        Rule::neg_expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            if first.as_rule() == Rule::neg_expr {
                let meta = ExprMeta::new(source, Some((span.start(), span.end())));
                Ok(Expr::Neg(Box::new(parse_expr(first)?), meta))
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
    let source = span.as_str().to_string();
    debug!("Parsing primary: {}", source);
    
    match pair.as_rule() {
        Rule::scalar => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_unbounded_number(inner_pair)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Scalar(bytes, meta))
        }
        Rule::coord1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner_pair, true, false)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Coord1(bytes, meta))
        }
        Rule::coord_pair => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::CoordPair(Box::new(expr1), Box::new(expr2), meta))
        }
        Rule::coord_triple => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let expr3 = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::CoordTriple(Box::new(expr1), Box::new(expr2), Box::new(expr3), meta))
        }

        // Package it up as if it's a CoordTriple of CoordPair
        Rule::coord_flat6 => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let expr3 = parse_expr(inner.next().unwrap())?;
            let expr4 = parse_expr(inner.next().unwrap())?;
            let expr5 = parse_expr(inner.next().unwrap())?;
            let expr6 = parse_expr(inner.next().unwrap())?;
            
            let meta1 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta2 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta3 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_outer = ExprMeta::from_span(span);
            
            Ok(Expr::CoordTriple(
                Box::new(Expr::CoordPair(Box::new(expr1), Box::new(expr2), meta1)),
                Box::new(Expr::CoordPair(Box::new(expr3), Box::new(expr4), meta2)),
                Box::new(Expr::CoordPair(Box::new(expr5), Box::new(expr6), meta3)),
                meta_outer
            ))
        }

        // Package it up as if it's a CoordPair of CoordTriple of CoordPair
        Rule::coord_flat12 => {
            let mut inner = pair.into_inner();
            let expr01 = parse_expr(inner.next().unwrap())?;
            let expr02 = parse_expr(inner.next().unwrap())?;
            let expr03 = parse_expr(inner.next().unwrap())?;
            let expr04 = parse_expr(inner.next().unwrap())?;
            let expr05 = parse_expr(inner.next().unwrap())?;
            let expr06 = parse_expr(inner.next().unwrap())?;
            let expr07 = parse_expr(inner.next().unwrap())?;
            let expr08 = parse_expr(inner.next().unwrap())?;
            let expr09 = parse_expr(inner.next().unwrap())?;
            let expr10 = parse_expr(inner.next().unwrap())?;
            let expr11 = parse_expr(inner.next().unwrap())?;
            let expr12 = parse_expr(inner.next().unwrap())?;

            let meta_cp1 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_cp2 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_cp3 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_cp4 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_cp5 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_cp6 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            
            let meta_ct1 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            let meta_ct2 = ExprMeta::new(source.clone(), Some((span.start(), span.end())));
            
            let meta_outer = ExprMeta::from_span(span);

            Ok(Expr::CoordPair(
                Box::new(
                    Expr::CoordTriple(
                        Box::new(Expr::CoordPair(Box::new(expr01), Box::new(expr02), meta_cp1)),
                        Box::new(Expr::CoordPair(Box::new(expr03), Box::new(expr04), meta_cp2)),
                        Box::new(Expr::CoordPair(Box::new(expr05), Box::new(expr06), meta_cp3)),
                        meta_ct1
                    )
                ),
                Box::new(
                    Expr::CoordTriple(
                        Box::new(Expr::CoordPair(Box::new(expr07), Box::new(expr08), meta_cp4)),
                        Box::new(Expr::CoordPair(Box::new(expr09), Box::new(expr10), meta_cp5)),
                        Box::new(Expr::CoordPair(Box::new(expr11), Box::new(expr12), meta_cp6)),
                        meta_ct2
                    )
                ),
                meta_outer
            ))
        }

        Rule::point => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?; // FIXME: better error messagesa
            let expr2 = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Point(Box::new(expr1), Box::new(expr2), meta))
        }

        Rule::compressed_point1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner_pair, false, true)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Point1Compressed(bytes, meta))
        }

        Rule::uncompressed_point1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<96>(inner_pair, false, true)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Point1Uncompressed(bytes, meta))
        }

        Rule::compressed_point2 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<96>(inner_pair, false, true)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Point2Compressed(bytes, meta))
        }

        Rule::uncompressed_point2 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<192>(inner_pair, false, true)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Point2Uncompressed(bytes, meta))
        }

        Rule::sqrt => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Sqrt(Box::new(arg), meta))
        }

        // 100proof: I'm doubling up on logic here but the naming aligns with `blst` so
        // users will known what is going on
        Rule::map_to_g1 => {
            let mut inner = pair.into_inner();
            let x_expr = parse_expr(inner.next().unwrap())?;
            let y_expr = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::MapToG1(Box::new(x_expr), Box::new(y_expr), meta))
        }

        Rule::map_to_g2 => {
            let mut inner = pair.into_inner();
            let x_expr = parse_expr(inner.next().unwrap())?;
            let y_expr = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::MapToG2(Box::new(x_expr), Box::new(y_expr), meta))
        }

        Rule::embed => {
            let mut inner = pair.into_inner();
            let p1 = parse_expr(inner.next().unwrap())?;
            let p2 = parse_expr(inner.next().unwrap())?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Embed(Box::new(p1), Box::new(p2), meta))
        }

        Rule::ident => {
            let name = parse_ident(pair)?;
            let meta = ExprMeta::from_span(span);
            Ok(Expr::Var(name, meta))
        },
        // Rule::expr => parse_expr(pair), // seems to be handled up in parse_expr for some reason
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

fn parse_unbounded_number(pair: Pair<Rule>) -> Result<Box<[u8]>, ParseError> {
    let span = pair.as_span();
    let err_invalid_number = parse_error(ParseErrorKind::InvalidHex, span);
    let s0 = pair.as_str();

    let (stripped, radix) = match s0.strip_prefix("0x") {
        Some(ss) => Ok((ss, 16)),
        None     => Ok((s0, 10)),
    }?;

    let n = BigUint::parse_bytes(stripped.as_bytes(), radix).ok_or(err_invalid_number.clone())?;
    let vec_bytes = n.to_bytes_le();
    Ok(vec_bytes.into_boxed_slice())
}


fn parse_number<const N: usize>(pair: Pair<Rule>, can_be_decimal: bool, exact_len: bool) -> Result<[u8; N], ParseError> {
    let span = pair.as_span();
    let err_invalid_number = parse_error(ParseErrorKind::InvalidHex, span);
    let s0 = pair.as_str();

    let (stripped, radix)  = match (s0.strip_prefix("0x"), can_be_decimal) {
        (Some(ss), _) => Ok((ss, 16)),
        (None, true)  => Ok((s0, 10)),
        (None, false) => Err(parse_error(ParseErrorKind::CannotBeDecimal, span)),
    }?;

    let len = stripped.len();
    if exact_len && len != (N*2) {
        return Err(parse_error(ParseErrorKind::MustBeExactlyHexDigits(N*2, len), span))
    }

    let n = BigUint::parse_bytes(stripped.as_bytes(), radix).ok_or(err_invalid_number.clone())?;
    let bytes = n.to_bytes_be();
    if bytes.len() > N {
        return Err(parse_error(ParseErrorKind::NumberTooLarge("FIXME".to_string()), span));
    }
    let mut padded = [0u8; N];
    padded[N - bytes.len()..].copy_from_slice(&bytes);
    Ok(padded)
}

