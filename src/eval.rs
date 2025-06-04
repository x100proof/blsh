use std::collections::HashMap;
use crate::ast::{Expr, ExprMeta};
use crate::types::{fp_to_hex, EvalError, Value, value_to_type};
use blst::*;
use num_bigint::BigUint;
use log::debug;

pub fn eval(expr: Expr, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
    debug!("Evaluating: {:?}", expr);

    match expr {
        Expr::Scalar(bytes, _) => {
            Ok(Value::Scalar(bytes))
        }
        Expr::Coord1(bytes, _) => {
            let mut fp = blst_fp::default();
            unsafe { blst_fp_from_bendian(&mut fp, bytes.as_ptr()) };
            Ok(Value::Coord1(fp))
        }
        Expr::CoordPair(expr1, expr2, _) => {
            let v1 = eval(*expr1, env)?;
            let v2 = eval(*expr2, env)?;

            match (v1, v2) {
                (Value::Coord1(x), Value::Coord1(y)) => {
                    Ok(Value::Coord2(blst_fp2 { fp: [x, y] }))
                }
                (Value::Coord6(x), Value::Coord6(y)) => {
                    Ok(Value::Coord12(blst_fp12 { fp6: [x, y] }))
                }
                _ => Err(EvalError::PairCanOnlyContainCoord1Or6)
            }
        }
        Expr::CoordTriple(expr1, expr2, expr3, _) => {
            let v1 = eval(*expr1, env)?;
            let v2 = eval(*expr2, env)?;
            let v3 = eval(*expr3, env)?;

            match (v1, v2, v3) {
                (Value::Coord2(x), Value::Coord2(y), Value::Coord2(z)) => {
                    Ok(Value::Coord6(blst_fp6 { fp2: [x, y, z] }))
                }
                _ => Err(EvalError::TripleCanOnlyContainCoord2)
            }
        }

        Expr::Point(expr1, expr2, _) => {
            let v1 = eval(*expr1, env)?;
            let v2 = eval(*expr2, env)?;
            match (v1,v2) {
                (Value::Coord1(x), Value::Coord1(y)) => {
                    let aff = blst_p1_affine { x: x, y: y };
                    let mut p = blst_p1::default();
                    unsafe { blst_p1_from_affine(&mut p, &aff); }
                    Ok(Value::Point1(p))
                }
                (Value::Coord2(x), Value::Coord2(y)) => {
                    let aff = blst_p2_affine { x: x, y: y };
                    let mut p = blst_p2::default();
                    unsafe { blst_p2_from_affine(&mut p, &aff); }
                    Ok(Value::Point2(p))
                }
                _ => Err(EvalError::PointCanOnlyContainCoord)
            }
        }

        Expr::Point1Compressed(bytes, _) => {
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_uncompress(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err)?;
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }

        Expr::Point1Uncompressed(bytes, _) => {
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_deserialize(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err)?;
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }


        Expr::Point2Compressed(bytes, _) => {
            let mut aff = blst_p2_affine::default();
            let err = unsafe { blst_p2_uncompress(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err)?;
            let mut p = blst_p2::default();
            unsafe { blst_p2_from_affine(&mut p, &aff); }
            Ok(Value::Point2(p))
        }

        Expr::Point2Uncompressed(bytes, _) => {
            let mut aff = blst_p2_affine::default();
            let err = unsafe { blst_p2_deserialize(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err)?;
            let mut p = blst_p2::default();
            unsafe { blst_p2_from_affine(&mut p, &aff); }
            Ok(Value::Point2(p))
        }

        Expr::Embed(expr1, expr2, _) => {
            let v1 = eval(*expr1, env)?;
            let v2 = eval(*expr2, env)?;
            match (v1, v2) {
                (Value::Point1(p1), Value::Point2(p2)) => {
                    let mut fp12 = blst_fp12::default();
                    let mut aff1 = blst_p1_affine::default();
                    let mut aff2 = blst_p2_affine::default();
                    unsafe {
                        blst_p1_to_affine(&mut aff1, &p1);
                        blst_p2_to_affine(&mut aff2, &p2);
                        blst_miller_loop(&mut fp12, &aff2, &aff1);
                        blst_final_exp(&mut fp12, &fp12);
                    }
                    Ok(Value::Coord12(fp12))
                }
                _ => Err(EvalError::EmbedHasIncorrectTypes)
            }
        }

        Expr::Sqrt(inner, _) => {
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
        },

        // 100proof: I had considered just calling the primitive `map` but then you lose
        // the idea that it's a prime order subgroup. `map_to_g` didn't really work for me either
        // Hence the duplication of logic for MapToG1/MapToG2
        Expr::MapToG1(x_expr, y_expr, _) => {
            let l = eval(*x_expr, env)?;
            let r = eval(*y_expr, env)?;
            match (l, r) {
                (Value::Coord1(x), Value::Coord1(y)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_map_to_g1(&mut result, &x, &y); }
                    Ok(Value::Point1(result))
                }
                _ => Err(EvalError::TypeMismatchMapToG1),
            }
        },

        Expr::MapToG2(x_expr,y_expr, _) => {
            let l = eval(*x_expr, env)?;
            let r = eval(*y_expr, env)?;
            match (l, r) {
                (Value::Coord2(x), Value::Coord2(y)) => {
                    let mut result = blst_p2::default();
                    unsafe { blst_map_to_g2(&mut result, &x, &y); }
                    Ok(Value::Point2(result))
                }
                _ => Err(EvalError::TypeMismatchMapToG2),
            }
        },

        Expr::Mul(lhs, rhs, _) => {
            let l = eval(*lhs, env)?;
            let r = eval(*rhs, env)?;
            match (l, r) {
                (Value::Point1(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point1(p)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_p1_mult(&mut result, &p, s.as_ptr(), s.len() * 8); }
                    Ok(Value::Point1(result))
                }
                (Value::Point2(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point2(p)) => {
                    let mut result = blst_p2::default();
                    unsafe { blst_p2_mult(&mut result, &p, s.as_ptr(), s.len() * 8); }
                    Ok(Value::Point2(result))
                }
                (Value::Scalar(s1), Value::Scalar(s2)) => {
                    let r = BigUint::from_bytes_le(&s1) * BigUint::from_bytes_le(&s2);
                    Ok(Value::Scalar(r.to_bytes_le().into_boxed_slice()))
                }
                _ => Err(EvalError::TypeMismatchMul),
            }
        }

        Expr::Add(lhs, rhs, meta) => {
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
                (lhsv, rhsv) => {
                    let lhs_type = value_to_type(&lhsv);
                    let rhs_type = value_to_type(&rhsv);

                    Err(EvalError::TypeMismatchPointAdd {
                        source: meta.source.clone(),
                        lhs_span: lhs.meta().span,
                        rhs_span: rhs.meta().span,
                        lhs_type,
                        rhs_type,
                    })
                }
            }
        }
        Expr::Sub(lhs, rhs, _) => {
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
                _ => Err(EvalError::TypeMismatchPointSub),
            }
        }
        Expr::Neg(inner, _) => {
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
                Value::Coord1(fp) => {
                    let mut fp_out = blst_fp::default();
                    unsafe { blst_fp_cneg(&mut fp_out, &fp, true); }
                    Ok(Value::Coord1(fp_out))
                }
                Value::Coord2(fp2) => {
                    let mut fp2_out = blst_fp2::default();
                    unsafe { blst_fp2_cneg(&mut fp2_out, &fp2, true); }
                    Ok(Value::Coord2(fp2_out))
                }
                _ => Err(EvalError::TypeMismatchNeg),
            }
        }
        Expr::Var(name, _) => {
            env.get(&name).cloned().ok_or(EvalError::UnknownIdent(name))
        }
    }
}


fn check_deserialize_error(err: BLST_ERROR) -> Result<(),EvalError> {
    if err == BLST_ERROR::BLST_SUCCESS { return Ok(()); }
    let e = match err {
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => EvalError::PointNotInGroup,
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => EvalError::PointNotOnCurve,
        _ => EvalError::InvalidCompressedPoint,
    };
    Err(e)
}
