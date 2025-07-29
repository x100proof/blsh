use pest::{error::InputLocation, Parser};
use pest::iterators::Pair;
use pest_derive::Parser;
use num_bigint::BigUint;
use pest::Span;
use log::debug;
use miette::{self, Diagnostic};
use thiserror::Error;

use crate::ast::{Command, Expr, PointComponentKind, SpanInfo, Statement};

type ErrorSpan = (usize, usize);

#[derive(Debug, Error, Diagnostic)]
pub enum ParseError {
    #[error("Invalid hex number")]
    InvalidHex {
        #[label("not a valid hex number")]
        span: ErrorSpan,
    },

    #[error("Literal must be exactly {expected} hex digits long")]
    MustBeExactlyHexDigits {
        #[label("found {actual} digits")]
        span: ErrorSpan,
        expected: usize,
        actual: usize,
    },

    #[error("Number too large")]
    NumberTooLarge {
        #[label("must be less than {max}")]
        span: ErrorSpan,
        max: String,
    },

    #[error("Unexpected token or end of input")]
    UnexpectedToken {
        #[label("unexpected")]
        span: ErrorSpan,
    },

    #[error("Cannot use reserved word as identifier")]
    ReservedWord {
        #[label("reserved word")]
        span: ErrorSpan,
    },

    #[error("Literal cannot be decimal")]
    CannotBeDecimal {
        #[label("must use hex format (0x...)")]
        span: ErrorSpan,
    },

    #[error("Parse error")]
    PestError {
        #[label("{message}")]
        span: ErrorSpan,
        message: String,
    },
}

// Built-in identifiers are not the same as reserved words
const RESERVED_WORDS: &[&str] = &[
    "let", "info", "sqrt", "point1", "point2", "coord1", "coord2", "scalar", "embed",
];

fn span_to_error_span(span: Span) -> ErrorSpan {
    (span.start(), span.end() - span.start())
}

#[derive(Parser)]
#[grammar = "blsh.pest"]

pub struct BlshParser;

pub fn parse_program(input: &str) -> Result<Statement, ParseError> {
    let mut pairs = BlshParser::parse(Rule::program, input)
        .map_err(|e| {
            ParseError::PestError {
                span: match e.location {
                    InputLocation::Pos(a) => (a, input.len() - a),
                    InputLocation::Span(span) => span

                },
                message: "Could not parse this fragment".to_string()
            }
        })?;
    let pair = pairs.next().unwrap();
    let stmt = pair.into_inner().next().unwrap(); // Now Rule::statement
    parse_statement(stmt.into_inner().next().unwrap())
}

fn parse_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> {
    let span = pair.as_span();
    match pair.as_rule() {
        Rule::let_stmt => {
            let mut inner = pair.into_inner();
            let name_pair = inner.next().unwrap();
            let name = name_pair.as_str().to_string();
            let expr = parse_expr(inner.next().unwrap())?;
            Ok(Statement::Let(name, expr, SpanInfo::from_span(name_pair.as_span())))
        }
        Rule::command_stmt => {
            let expr = parse_expr(pair.into_inner().next().unwrap())?;
            Ok(Statement::Command(Command::Info(expr)))
        }
        Rule::expr => Ok(Statement::Expr(parse_expr(pair)?)),
        _ => Err(ParseError::UnexpectedToken {
            span: span_to_error_span(span),
        }),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let span = pair.as_span();
    debug!("Parsing expression: {:?}", pair);

    match pair.as_rule() {
        Rule::expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(op_pair) = inner.next() {
                debug!("op_pair {:?}", op_pair);
                let rhs = parse_expr(inner.next().unwrap())?;
                let op_str = op_pair.as_str();
                let span_info = lhs.span_info().merge(rhs.span_info());
                lhs = match op_str {
                    "=="  => Expr::Equal(Box::new(lhs), Box::new(rhs), span_info),
                    _ => return Err(ParseError::UnexpectedToken {
                        span: span_to_error_span(span),
                    }),
                };
            }
            Ok(lhs)
        }
        Rule::eq_expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(op_pair) = inner.next() {
                let rhs = parse_expr(inner.next().unwrap())?;
                let op_str = op_pair.as_str();
                let span_info = lhs.span_info().merge(rhs.span_info());
                // debug!("parse lhs: {:?}, rhs: {:?}, span: {:?}, lhs_span: {:?}", lhs, rhs, span, lhs_span);
                lhs = match op_str {
                    "+"  => Expr::Add(Box::new(lhs), Box::new(rhs), span_info),
                    "-"  => Expr::Sub(Box::new(lhs), Box::new(rhs), span_info),
                    _ => return Err(ParseError::UnexpectedToken {
                        span: span_to_error_span(span),
                    }),
                };
            }
            Ok(lhs)

        }
        Rule::mul_expr => {
            let mut inner = pair.into_inner();
            let mut lhs = parse_expr(inner.next().unwrap())?;
            while let Some(rhs_pair) = inner.next() {
                let rhs = parse_expr(rhs_pair)?;
                let span_info = lhs.span_info().merge(rhs.span_info());
                lhs = Expr::Mul(Box::new(lhs), Box::new(rhs), span_info);
            }
            Ok(lhs)
        }
        Rule::neg_expr => {
            let mut inner = pair.into_inner();
            let first = inner.next().unwrap();
            if first.as_rule() == Rule::neg_expr {
                let span_info = SpanInfo::from_span(span);
                Ok(Expr::Neg(Box::new(parse_expr(first)?), span_info))
            } else {
                parse_expr(first)
            }
        }
        Rule::primary_or_component  => {
            let mut inner = pair.into_inner();
            let child = inner.next().unwrap();
            let primary_expr = match child.as_rule() {
                Rule::primary => parse_primary(child.into_inner().next().unwrap()),
                _ => panic!("Expected rule::primary"),
            }?;
            let mut expr = primary_expr;
            while let Some(component_pair) = inner.next() {
                let component_span = component_pair.as_span();
                let component_span_info = SpanInfo::from_span(component_span);
                let span_info = expr.span_info().merge(&component_span_info);
                let opt_kind = match component_span.as_str() {
                    ".x" => Some(PointComponentKind::X),
                    ".y" => Some(PointComponentKind::Y),
                    ".z" => Some(PointComponentKind::Z),
                    _    => None
                };
                if let Some(kind) = opt_kind {
                  expr = Expr::PointComponent(Box::new(expr), (kind, component_span_info), span_info);
                }
            }
            Ok(expr)
        }
        _ => parse_primary(pair),
    }
}


fn parse_primary(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let span = pair.as_span();
    debug!("Parsing primary: {:?}", pair);

    match pair.as_rule() {
        Rule::bool => {
            let b = span.as_str() == "true";
            Ok(Expr::LiteralBool(b, SpanInfo::from_span(span)))
        }
        Rule::scalar => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_unbounded_number(inner_pair)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Scalar(bytes, span_info))
        }
        Rule::coord1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner_pair, true, false)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Coord1(bytes, span_info))
        }
        Rule::coord_pair => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::CoordPair(Box::new(expr1), Box::new(expr2), span_info))
        }
        Rule::coord_triple => {
            let mut inner = pair.into_inner();
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let expr3 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::CoordTriple(Box::new(expr1), Box::new(expr2), Box::new(expr3), span_info))
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

            let meta1 = SpanInfo::from_span(span);
            let meta2 = SpanInfo::from_span(span);
            let meta3 = SpanInfo::from_span(span);
            let meta_outer = SpanInfo::from_span(span);

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

            let meta_cp1 = SpanInfo::from_span(span);
            let meta_cp2 = SpanInfo::from_span(span);
            let meta_cp3 = SpanInfo::from_span(span);
            let meta_cp4 = SpanInfo::from_span(span);
            let meta_cp5 = SpanInfo::from_span(span);
            let meta_cp6 = SpanInfo::from_span(span);
            let meta_ct1 = SpanInfo::from_span(span);
            let meta_ct2 = SpanInfo::from_span(span);

            let meta_outer = SpanInfo::from_span(span);

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
            let expr1 = parse_expr(inner.next().unwrap())?;
            let expr2 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Point(Box::new(expr1), Box::new(expr2), span_info))
        }

        Rule::compressed_point1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<48>(inner_pair, false, true)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Point1Compressed(bytes, span_info))
        }

        Rule::uncompressed_point1 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<96>(inner_pair, false, true)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Point1Uncompressed(bytes, span_info))
        }

        Rule::compressed_point2 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<96>(inner_pair, false, true)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Point2Compressed(bytes, span_info))
        }

        Rule::uncompressed_point2 => {
            let inner_pair = pair.into_inner().next().unwrap();
            let bytes = parse_number::<192>(inner_pair, false, true)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Point2Uncompressed(bytes, span_info))
        }

        Rule::inv => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Inv(Box::new(arg), span_info))
        }

        Rule::sqr => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Sqr(Box::new(arg), span_info))
        }

        Rule::sqrt => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Sqrt(Box::new(arg), span_info))
        }

        Rule::conj => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Conjugate(Box::new(arg), span_info))
        }

        Rule::pow => {
            let mut inner = pair.into_inner();
            let e1 = parse_expr(inner.next().unwrap())?;
            let e2 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Pow(Box::new(e1), Box::new(e2), span_info))
        }

        Rule::miller_loop => {
            let inner = pair.into_inner().next().unwrap();
            let arg = parse_expr(inner)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::MillerLoop(Box::new(arg), span_info))
        }


        // 100proof: I'm doubling up on logic here but the naming aligns with `blst` so
        // users will known what is going on
        Rule::map_to_g1 => {
            let mut inner = pair.into_inner();
            let x_expr = parse_expr(inner.next().unwrap())?;
            let y_expr = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::MapToG1(Box::new(x_expr), Box::new(y_expr), span_info))
        }

        Rule::map_to_g2 => {
            let mut inner = pair.into_inner();
            let x_expr = parse_expr(inner.next().unwrap())?;
            let y_expr = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::MapToG2(Box::new(x_expr), Box::new(y_expr), span_info))
        }

        Rule::embed_nfe => {
            let mut inner = pair.into_inner();
            let p1 = parse_expr(inner.next().unwrap())?;
            let p2 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::EmbedNFE(Box::new(p1), Box::new(p2), span_info))
        }


        Rule::embed => {
            let mut inner = pair.into_inner();
            let p1 = parse_expr(inner.next().unwrap())?;
            let p2 = parse_expr(inner.next().unwrap())?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Embed(Box::new(p1), Box::new(p2), span_info))
        }

        Rule::ident => {
            let name = parse_ident(pair)?;
            let span_info = SpanInfo::from_span(span);
            Ok(Expr::Var(name, span_info))
        },
        // Rule::expr => parse_expr(pair), // seems to be handled up in parse_expr for some reason

        Rule::list_expr => parse_list_expr(pair),

        Rule::expr => parse_expr(pair),

        _ => Err(ParseError::UnexpectedToken {
            span: span_to_error_span(span),
        }),

    }
}

fn parse_list_expr(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = &parse_expr(inner.next().unwrap())?;
    let mut span_info = first.span_info().to_owned();
    let mut elems = vec![first.clone()];
    while let Some(next_pair) = inner.next() {
        let next = &parse_expr(next_pair)?;
        elems.push(next.clone());
        span_info = span_info.merge(next.span_info());
    }
    Ok(Expr::List(elems, span_info.to_owned()))
}

fn parse_ident(pair: Pair<Rule>) -> Result<String, ParseError> {
    let span = pair.as_span();
    let s = pair.as_str();
    if RESERVED_WORDS.contains(&s) {
        Err(ParseError::ReservedWord {
            span: span_to_error_span(span),
        })
    } else {
        Ok(s.to_string())
    }
}

fn parse_unbounded_number(pair: Pair<Rule>) -> Result<Box<[u8]>, ParseError> {
    let span = pair.as_span();
    let s0 = pair.as_str();

    let (stripped, radix) = match s0.strip_prefix("0x") {
        Some(ss) => Ok((ss, 16)),
        None     => Ok((s0, 10)),
    }?;

    let n = BigUint::parse_bytes(stripped.as_bytes(), radix).ok_or(ParseError::InvalidHex {
        span: span_to_error_span(span),
    })?;

    let vec_bytes = n.to_bytes_le();
    Ok(vec_bytes.into_boxed_slice())
}

fn parse_number<const N: usize>(pair: Pair<Rule>, can_be_decimal: bool, exact_len: bool) -> Result<[u8; N], ParseError> {
    let span = pair.as_span();
    let s0 = pair.as_str();

    let (stripped, radix) = match (s0.strip_prefix("0x"), can_be_decimal) {
        (Some(ss), _) => Ok((ss, 16)),
        (None, true)  => Ok((s0, 10)),
        (None, false) => Err(ParseError::CannotBeDecimal {
            span: span_to_error_span(span),
        }),
    }?;

    let len = stripped.len();
    if exact_len && len != (N*2) {
        return Err(ParseError::MustBeExactlyHexDigits {
            span: span_to_error_span(span),
            expected: N*2,
            actual: len,
        });
    }

    let n = BigUint::parse_bytes(stripped.as_bytes(), radix).ok_or(ParseError::InvalidHex {
        span: span_to_error_span(span),
    })?;

    let bytes = n.to_bytes_be();
    if bytes.len() > N {
        return Err(ParseError::NumberTooLarge {
            span: span_to_error_span(span),
            max: format!("0x{}", "F".repeat(N*2)),
        });
    }

    let mut padded = [0u8; N];
    padded[N - bytes.len()..].copy_from_slice(&bytes);
    Ok(padded)
}

