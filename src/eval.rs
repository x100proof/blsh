use std::collections::HashMap;
use num_bigint::BigUint;
use crate::ast::Expr;
use crate::types::{Value, ParseError};
use blst::*;
use hex;

pub fn eval_expr(expr: Expr, env: &HashMap<String, Value>) -> Result<Value, ParseError> {
    match expr {
        Expr::Scalar(s) => parse_scalar(&s),
        Expr::Coord1(s) => parse_coord1(&s),
        Expr::Point1(x_expr, y_expr) => {
            let x = eval_expr(*x_expr, env)?;
            let y = eval_expr(*y_expr, env)?;
            match (x, y) {
                (Value::Coord1(x_fp), Value::Coord1(y_fp)) => {
                    let aff = blst_p1_affine { x: x_fp, y: y_fp };
                    let mut p = blst_p1::default();
                    unsafe { blst_p1_from_affine(&mut p, &aff); }
                    Ok(Value::Point1(p))
                }
                _ => Err(ParseError::TypeError("point1 requires Coord1 arguments")),
            }
        }
        Expr::Point1Compressed(s) => {
            let bytes = hex::decode(s.strip_prefix("0x").unwrap_or(&s)).map_err(|_| ParseError::InvalidCompressedPoint)?;
            if bytes.len() != 48 {
                return Err(ParseError::InvalidCompressedPoint);
            }
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_uncompress(&mut aff, bytes.as_ptr()) };
            if err != BLST_ERROR::BLST_SUCCESS {
                return Err(ParseError::InvalidCompressedPoint);
            }
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }
        Expr::Sqrt(inner) => {
            let v = eval_expr(*inner, env)?;
            let coord = match v {
                Value::Coord1(fp) => fp,
                _ => return Err(ParseError::TypeError("sqrt requires Coord1 input")),
            };
            if unsafe { blst_fp_is_square(&coord) } {
                let mut result = blst_fp::default();
                unsafe { blst_fp_sqrt(&mut result, &coord); }
                Ok(Value::Coord1(result))
            } else {
                Err(ParseError::NotASquare)
            }
        }
        Expr::Mul(lhs, rhs) => {
            let l = eval_expr(*lhs, env)?;
            let r = eval_expr(*rhs, env)?;
            match (l, r) {
                (Value::Point1(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point1(p)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_p1_mult(&mut result, &p, s.b.as_ptr(), 256); }
                    Ok(Value::Point1(result))
                }
                _ => Err(ParseError::MismatchedOperands),
            }
        }
        Expr::Var(name) => {
            env.get(&name).cloned().ok_or(ParseError::InvalidExpression)
        }
    }
}

fn parse_scalar(input: &str) -> Result<Value, ParseError> {
    use num_traits::Num;
    let bigint = if let Some(hex) = input.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16).map_err(|_| ParseError::InvalidBigInt)?
    } else {
        BigUint::from_str_radix(input, 10).map_err(|_| ParseError::InvalidBigInt)?
    };
    let bytes = bigint.to_bytes_be();
    if bytes.len() > 32 {
        return Err(ParseError::ScalarTooBig);
    }
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    let mut s = blst_scalar::default();
    unsafe { blst_scalar_from_bendian(&mut s, padded.as_ptr()) };
    Ok(Value::Scalar(s))
}

fn parse_coord1(input: &str) -> Result<Value, ParseError> {
    use num_traits::Num;
    let bigint = if let Some(hex) = input.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16).map_err(|_| ParseError::InvalidBigInt)?
    } else {
        BigUint::from_str_radix(input, 10).map_err(|_| ParseError::InvalidBigInt)?
    };
    let bytes = bigint.to_bytes_be();
    if bytes.len() > 48 {
        return Err(ParseError::CoordTooBig);
    }
    let mut padded = [0u8; 48];
    padded[48 - bytes.len()..].copy_from_slice(&bytes);
    let mut fp = blst_fp::default();
    unsafe { blst_fp_from_bendian(&mut fp, padded.as_ptr()) };
    Ok(Value::Coord1(fp))
}