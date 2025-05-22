use std::collections::HashMap;
use crate::ast::Expr;
use crate::types::{fp_to_hex, EvalError, Value};
use blst::*;

pub fn eval(expr: Expr, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
    match expr {
        Expr::Scalar(bytes) => {
            let mut s = blst_scalar::default();
            unsafe { blst_scalar_from_bendian(&mut s, bytes.as_ptr()) };
            Ok(Value::Scalar(s))
        }
        Expr::Coord1(bytes) => {
            let mut fp = blst_fp::default();
            unsafe { blst_fp_from_bendian(&mut fp, bytes.as_ptr()) };
            Ok(Value::Coord1(fp))
        }
        Expr::Coord2(bytes) => {
            let mut fp2 = blst_fp2::default();
            unsafe {
                // real part (first 48 bytes)
                blst_fp_from_bendian(&mut fp2.fp[0], bytes.as_ptr());

                // imaginary part (next 48 bytes)
                blst_fp_from_bendian(&mut fp2.fp[1], bytes.as_ptr().add(48));
            }
            Ok(Value::Coord2(fp2))
        }
        Expr::Point1(x_expr, y_expr) => {
            let x = eval(*x_expr, env)?;
            let y = eval(*y_expr, env)?;
            if let (Value::Coord1(x_fp), Value::Coord1(y_fp)) = (x, y) {
                let aff = blst_p1_affine { x: x_fp, y: y_fp };
                let mut p = blst_p1::default();
                unsafe { blst_p1_from_affine(&mut p, &aff); }
                Ok(Value::Point1(p))
            } else {
                Err(EvalError::TypeMismatchPoint1Coord1)
            }
        }
        Expr::Point2(x_expr, y_expr) => {
            let x = eval(*x_expr, env)?;
            let y = eval(*y_expr, env)?;
            if let (Value::Coord2(x_fp2), Value::Coord2(y_fp2)) = (x, y) {
                let aff = blst_p2_affine { x: x_fp2, y: y_fp2 };
                let mut p = blst_p2::default();
                unsafe { blst_p2_from_affine(&mut p, &aff); }
                Ok(Value::Point2(p))
            } else {
                Err(EvalError::TypeMismatchPoint2Coord2)
            }
        }
        Expr::Point1Compressed(bytes) => {
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_uncompress(&mut aff, bytes.as_ptr()) };
            if err != BLST_ERROR::BLST_SUCCESS {
                return Err(EvalError::InvalidCompressedPoint);
            }
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }
        Expr::Point2Compressed(bytes) => {
            let mut aff = blst_p2_affine::default();
            let err = unsafe { blst_p2_uncompress(&mut aff, bytes.as_ptr()) };
            if err != BLST_ERROR::BLST_SUCCESS {
                return Err(EvalError::InvalidCompressedPoint);
            }
            let mut p = blst_p2::default();
            unsafe { blst_p2_from_affine(&mut p, &aff); }
            Ok(Value::Point2(p))
        }
        Expr::Sqrt(inner) => {
            let v = eval(*inner, env)?;
            let coord = match v {
                Value::Coord1(fp) => fp,
                _ => return Err(EvalError::TypeMismatchCoord1),
            };
            if unsafe { blst_fp_is_square(&coord) } {
                let mut result = blst_fp::default();
                unsafe { blst_fp_sqrt(&mut result, &coord); }
                Ok(Value::Coord1(result))
            } else {
                Err(EvalError::NotASquare(fp_to_hex(&coord)))
            }
        }
        Expr::Mul(lhs, rhs) => {
            let l = eval(*lhs, env)?;
            let r = eval(*rhs, env)?;
            match (l, r) {
                (Value::Point1(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point1(p)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_p1_mult(&mut result, &p, s.b.as_ptr(), 256); }
                    Ok(Value::Point1(result))
                }
                _ => Err(EvalError::TypeMismatchMul),
            }
        }
        Expr::Add(lhs, rhs) => {
            let l = eval(*lhs, env)?;
            let r = eval(*rhs, env)?;
            match (l, r) {
                (Value::Point1(p1), Value::Point1(p2)) => {
                    let mut out = blst_p1::default();
                    unsafe { blst_p1_add_or_double(&mut out, &p1, &p2); }
                    Ok(Value::Point1(out))
                }
                (Value::Point2(p1), Value::Point2(p2)) => {
                    let mut out = blst_p2::default();
                    unsafe { blst_p2_add_or_double(&mut out, &p1, &p2); }
                    Ok(Value::Point2(out))
                }
                _ => Err(EvalError::TypeMismatchPoint1Add),
            }
        }
        Expr::Sub(lhs, rhs) => {
            let l = eval(*lhs, env)?;
            let r = eval(*rhs, env)?;
            match (l, r) {
                (Value::Point1(p1), Value::Point1(mut p2)) => {
                    unsafe { blst_p1_cneg(&mut p2, true); }
                    let mut out = blst_p1::default();
                    unsafe { blst_p1_add_or_double(&mut out, &p1, &p2); }
                    Ok(Value::Point1(out))
                }
                (Value::Point2(p1), Value::Point2(mut p2)) => {
                    unsafe { blst_p2_cneg(&mut p2, true); }
                    let mut out = blst_p2::default();
                    unsafe { blst_p2_add_or_double(&mut out, &p1, &p2); }
                    Ok(Value::Point2(out))
                }
                _ => Err(EvalError::TypeMismatchPoint1Sub),
            }
        }
        Expr::Neg(inner) => {
            let v = eval(*inner, env)?;
            match v {
                Value::Point1(mut p) => {
                    unsafe { blst_p1_cneg(&mut p, true); }
                    Ok(Value::Point1(p))
                }
                Value::Point2(mut p) => {
                    unsafe { blst_p2_cneg(&mut p, true); }
                    Ok(Value::Point2(p))
                }
                _ => Err(EvalError::TypeMismatchPoint1Neg),
            }
        }
        Expr::Var(name) => {
            env.get(&name).cloned().ok_or(EvalError::UnknownIdent(name))
        }
    }
}
