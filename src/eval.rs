use std::collections::HashMap;
use crate::ast::Expr;
use crate::types::{Value, EvalError};
use blst::*;

pub fn eval_expr(expr: Expr, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
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
        Expr::Point1(x_expr, y_expr) => {
            let x = eval_expr(*x_expr, env)?;
            let y = eval_expr(*y_expr, env)?;
            if let (Value::Coord1(x_fp), Value::Coord1(y_fp)) = (x, y) {
                let aff = blst_p1_affine { x: x_fp, y: y_fp };
                let mut p = blst_p1::default();
                unsafe { blst_p1_from_affine(&mut p, &aff); }
                Ok(Value::Point1(p))
            } else {
                Err(EvalError::InvalidExpression)
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
        Expr::Sqrt(inner) => {
            let v = eval_expr(*inner, env)?;
            let coord = match v {
                Value::Coord1(fp) => fp,
                _ => return Err(EvalError::InvalidExpression),
            };
            if unsafe { blst_fp_is_square(&coord) } {
                let mut result = blst_fp::default();
                unsafe { blst_fp_sqrt(&mut result, &coord); }
                Ok(Value::Coord1(result))
            } else {
                Err(EvalError::NotASquare)
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
                _ => Err(EvalError::MismatchedOperands),
            }
        }
        Expr::Var(name) => {
            env.get(&name).cloned().ok_or(EvalError::InvalidExpression)
        }
    }
}