use std::collections::HashMap;
use crate::ast::{Expr, PointComponentKind};
use crate::eval_error::{EvalError};
use crate::types::{Value, ValueType};
use blst::*;
use num_bigint::BigUint;
use log::debug;

pub fn eval(expr: &Expr, env: &HashMap<String, Value>) -> Result<Value, EvalError> {
    debug!("Evaluating: {:?}", expr);

    match expr {
        Expr::LiteralBool(b, _) => Ok(Value::Bool(*b)),
        Expr::Scalar(bytes, _) => Ok(Value::Scalar(bytes.clone())),
        Expr::Coord1(bytes, _) => {
            let mut fp = blst_fp::default();
            unsafe { blst_fp_from_bendian(&mut fp, bytes.as_ptr()) };
            Ok(Value::Coord1(fp))
        }
        Expr::CoordPair(expr1, expr2, _) => {
            let v1 = eval(expr1, env)?;
            let v2 = eval(expr2, env)?;

            match (v1, v2) {
                (Value::Coord1(x), Value::Coord1(y)) => {
                    Ok(Value::Coord2(blst_fp2 { fp: [x, y] }))
                }
                (Value::Coord6(x), Value::Coord6(y)) => {
                    Ok(Value::Coord12(blst_fp12 { fp6: [x, y] }))
                }
                (Value::Point1(p1), Value::Point2(p2)) => {
                    Ok(Value::MillerPair(p1, p2))
                }
                (v1,v2) => Err(EvalError::PairCanOnlyContainCoord1Or6 {
                    x_span: expr1.miette_span(),
                    y_span: expr2.miette_span(),
                    x_coord_type: v1.to_type_string(),
                    y_coord_type: v2.to_type_string(),
                })
            }
        }
        Expr::CoordTriple(expr1, expr2, expr3, _) => {
            let v1 = eval(expr1, env)?;
            let v2 = eval(expr2, env)?;
            let v3 = eval(expr3, env)?;

            match (&v1, &v2, &v3) {
                (Value::Coord2(x), Value::Coord2(y), Value::Coord2(z)) => {
                    Ok(Value::Coord6(blst_fp6 { fp2: [*x, *y, *z] }))
                }
                _ => Err(EvalError::TripleCanOnlyContainCoord2 {
                    x_span: expr1.miette_span(),
                    y_span: expr2.miette_span(),
                    z_span: expr3.miette_span(),
                    x_coord_type: v1.to_type_string(),
                    y_coord_type: v2.to_type_string(),
                    z_coord_type: v3.to_type_string(),
                })
            }
        }

        Expr::Point(expr1, expr2, _) => {
            let v1 = eval(expr1, env)?;
            let v2 = eval(expr2, env)?;
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
                (v1,v2) => Err(EvalError::PointCanOnlyContainCoord {
                    x_span: expr1.miette_span(),
                    y_span: expr2.miette_span(),
                    point_type: match (&v1, &v2) {
                        (Value::Coord1(_), _) | (_, Value::Coord1(_)) => "Point1",
                        (Value::Coord2(_), _) | (_, Value::Coord2(_)) => "Point2",
                        _ => "Point",
                    }.to_string(),
                    expected_coord_type: match (&v1, &v2) {
                        (Value::Coord1(_), _) | (_, Value::Coord1(_)) => "Coord1",
                        (Value::Coord2(_), _) | (_, Value::Coord2(_)) => "Coord2",
                        _ => "Coord",
                    }.to_string(),
                    x_coord_type: v1.to_type_string(),
                    y_coord_type: v2.to_type_string(),
                })
            }
        }

        Expr::PointComponent(e, (kind, s), _) => {
            let v = eval(e,env)?;
            let z_component_error =
                    EvalError::ZComponentSelectorOnlyOnCoord6 {
                        component_span: s.miette_span(), expr_span: e.miette_span(), expr_type: v.to_type_string() }.to_owned();
            match v {
                Value::Point1(p1) =>
                    match kind {
                        PointComponentKind::X => Ok(Value::Coord1(p1.x)),
                        PointComponentKind::Y => Ok(Value::Coord1(p1.y)),
                        PointComponentKind::Z => Err(z_component_error),

                    },
                Value::Point2(p2) =>
                    match kind {
                        PointComponentKind::X => Ok(Value::Coord2(p2.x)),
                        PointComponentKind::Y => Ok(Value::Coord2(p2.y)),
                        PointComponentKind::Z => Err(z_component_error),
                    },
                Value::Coord2(c2) =>
                    match kind {
                        PointComponentKind::X => Ok(Value::Coord1(c2.fp[0])),
                        PointComponentKind::Y => Ok(Value::Coord1(c2.fp[1])),
                        PointComponentKind::Z => Err(z_component_error),
                    },
                Value::Coord6(c6) =>
                    match kind {
                        PointComponentKind::X => Ok(Value::Coord2(c6.fp2[0])),
                        PointComponentKind::Y => Ok(Value::Coord2(c6.fp2[1])),
                        PointComponentKind::Z => Ok(Value::Coord2(c6.fp2[2])),
                    },
                Value::Coord12(c12) =>
                    match kind {
                        PointComponentKind::X => Ok(Value::Coord6(c12.fp6[0])),
                        PointComponentKind::Y => Ok(Value::Coord6(c12.fp6[1])),
                        PointComponentKind::Z => Err(z_component_error),
                    },

                _ => Err(EvalError::ComponentsOnlyValidForPointsAndCoords {
                    component_span: s.miette_span(),
                    expr_span: e.miette_span(),
                    expr_type: v.to_type_string()
                })
            }
        },


        Expr::Point1Compressed(bytes, _) => {
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_uncompress(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err, expr.miette_span(), Curve::E1)?;
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }

        Expr::Point1Uncompressed(bytes, _) => {
            let mut aff = blst_p1_affine::default();
            let err = unsafe { blst_p1_deserialize(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err, expr.miette_span(), Curve::E1)?;
            let mut p = blst_p1::default();
            unsafe { blst_p1_from_affine(&mut p, &aff); }
            Ok(Value::Point1(p))
        }


        Expr::Point2Compressed(bytes, _) => {
            let mut aff = blst_p2_affine::default();
            let err = unsafe { blst_p2_uncompress(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err, expr.miette_span(), Curve::E2)?;
            let mut p = blst_p2::default();
            unsafe { blst_p2_from_affine(&mut p, &aff); }
            Ok(Value::Point2(p))
        }

        Expr::Point2Uncompressed(bytes, _) => {
            let mut aff = blst_p2_affine::default();
            let err = unsafe { blst_p2_deserialize(&mut aff, bytes.as_ptr()) };
            check_deserialize_error(err, expr.miette_span(), Curve::E2)?;
            let mut p = blst_p2::default();
            unsafe { blst_p2_from_affine(&mut p, &aff); }
            Ok(Value::Point2(p))
        }

        Expr::Embed(expr1, expr2, _) => {
            let v1 = eval(expr1, env)?;
            let v2 = eval(expr2, env)?;
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
                (v1,v2) => Err(EvalError::EmbedHasIncorrectTypes {
                         span1: expr1.miette_span(),
                         span2: expr2.miette_span(),
                         type1: v1.to_type_string(),
                         type2: v2.to_type_string(),
                })
            }
        }

        // FIXME: Distinguish multiplicative inverse from exponentiation inverse
        Expr::Inv(inner, _) => {
            let v = eval(inner, env)?;
            match v {
                Value::Coord1(fp) => {
                    let mut result = blst_fp::default();
                    unsafe { blst_fp_inverse(&mut result, &fp); }
                    Ok(Value::Coord1(result))
                },
                Value::Coord2(fp2) => {
                    let mut result = blst_fp2::default();
                    unsafe { blst_fp2_inverse(&mut result, &fp2); }
                    Ok(Value::Coord2(result))
                },
                Value::Coord12(fp12) => {
                    let mut result = blst_fp12::default();
                    unsafe { blst_fp12_inverse(&mut result, &fp12); }
                    Ok(Value::Coord12(result))
                },
                v => Err(EvalError::InvExpectsCoord12 { span: inner.miette_span(), expr_type: v.to_type_string() })
            }
        }

        Expr::Sqr(inner, _) => {
            let v = eval(inner, env)?;
            match v {
                Value::Coord12(fp12) => {
                    let mut result = blst_fp12::default();
                    unsafe { blst_fp12_sqr(&mut result, &fp12); }
                    Ok(Value::Coord12(result))
                },
                v => Err(EvalError::SqrExpectsCoord12 { span: inner.miette_span(), expr_type: v.to_type_string() })
            }
        }

        Expr::Sqrt(inner, _) => {
            let v = eval(inner, env)?;
            match v {
                Value::Coord1(fp) => {
                    if unsafe { blst_fp_is_square(&fp) } {
                        let mut result = blst_fp::default();
                        unsafe { blst_fp_sqrt(&mut result, &fp); }
                        Ok(Value::Coord1(result))
                    } else {
                        Err(EvalError::NotASquare {
                            span: inner.miette_span(),
                            expr_type: v.to_type_string(),
                        })
                    }
                }
                Value::Coord2(fp2) => {
                    if unsafe { blst_fp2_is_square(&fp2) } {
                        let mut result = blst_fp2::default();
                        unsafe { blst_fp2_sqrt(&mut result, &fp2); }
                        Ok(Value::Coord2(result))
                    } else {
                        Err(EvalError::NotASquare {
                            span: inner.miette_span(),
                            expr_type: v.to_type_string(),
                        })
                    }
                }
                v => Err(EvalError::SqrtExpectsCoords { span: inner.miette_span(), expr_type: v.to_type_string()}),
            }
        }
        Expr::Pow(e1, e2, _) => {
            let v1 = eval(e1, env)?;
            let v2 = eval(e2, env)?;
            match (&v1, &v2) {
                (Value::Coord1(x), Value::Coord1(e)) =>
                    Ok(Value::Coord1(fp_pow(*x,*e))),
                _ => todo!()
            }

        }


        Expr::MillerLoop(inner, _) => {
            let v = eval(inner, env)?;
            let eval_error = EvalError::MillerLoopMustContainListOfMillerPairs { span: inner.miette_span(),
                                                                                            expr_type: v.to_type_string()};
            match v {
                Value::List(miller_points) => {
                    if miller_points[0].to_type() == ValueType::MillerPair {
                        let (ps0,qs0): (Vec<blst_p1_affine>, Vec<blst_p2_affine>) =
                                miller_points.iter().map(|v| {
                            match *v {
                                Value::MillerPair(p, q) => {
                                    let mut p_aff = blst_p1_affine::default();
                                    unsafe { blst_p1_to_affine(&mut p_aff, &p) };
                                    let mut q_aff = blst_p2_affine::default();
                                    unsafe { blst_p2_to_affine(&mut q_aff, &q) };
                                    Ok((p_aff, q_aff))
                                }
                                _ => Err(eval_error.clone())
                            }
                        }).collect::<Result<_,_>>()?;
                        let mut res = blst_fp12::default();
                        let ps: Vec<*const blst_p1_affine> = ps0.iter().map(|p|p as *const _).collect();
                        let qs: Vec<*const blst_p2_affine> = qs0.iter().map(|q|q as *const _).collect();
                        unsafe { blst_miller_loop_n(&mut res, qs.as_ptr(), ps.as_ptr(), qs.len()) };
                        unsafe { blst_final_exp(&mut res, &res)};
                        Ok(Value::Coord12(res))
                    } else {
                        Err(eval_error)
                    }
                },
                _ => Err(eval_error)
            }
        }

        // 100proof: I had considered just calling the primitive `map` but then you lose
        // the idea that it's a prime order subgroup. `map_to_g` didn't really work for me either
        // Hence the duplication of logic for MapToG1/MapToG2
        Expr::MapToG1(x_expr, y_expr, _) => {
            let l = eval(x_expr, env)?;
            let r = eval(y_expr, env)?;
            match (l, r) {
                (Value::Coord1(x), Value::Coord1(y)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_map_to_g1(&mut result, &x, &y); }
                    Ok(Value::Point1(result))
                }
                (l, r) => Err(EvalError::TypeMismatchMapToG1 {
                    span1: x_expr.miette_span(),
                    span2: y_expr.miette_span(),
                    type1: l.to_type_string(),
                    type2: r.to_type_string(),
                }),
            }
        },

        Expr::MapToG2(x_expr,y_expr, _) => {
            let l = eval(x_expr, env)?;
            let r = eval(y_expr, env)?;
            match (l, r) {
                (Value::Coord2(x), Value::Coord2(y)) => {
                    let mut result = blst_p2::default();
                    unsafe { blst_map_to_g2(&mut result, &x, &y); }
                    Ok(Value::Point2(result))
                }
                (l,r) => Err(EvalError::TypeMismatchMapToG2 {
                    span1: x_expr.miette_span(),
                    span2: y_expr.miette_span(),
                    type1: l.to_type_string(),
                    type2: r.to_type_string(),
                }),
            }
        },

        Expr::Equal(lhs, rhs, _) => {
            let l = &eval(lhs, env)?;
            let r = &eval(rhs, env)?;
            if l.equal_type(r) {
                Ok(Value::Bool(l.is_equal(r)))
            } else {
                Err(EvalError::EqualityOnSameTypesOnly {
                    lhs_span: lhs.miette_span(),
                    rhs_span: rhs.miette_span(),
                    lhs_type: l.to_type_string(),
                    rhs_type: r.to_type_string()
                })
            }
        }

        Expr::Mul(lhs, rhs, _) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
            match (l, r) {
                (Value::Scalar(s1), Value::Scalar(s2)) => {
                    let r = BigUint::from_bytes_le(&s1) * BigUint::from_bytes_le(&s2);
                    Ok(Value::Scalar(r.to_bytes_le().into_boxed_slice()))
                },
                (Value::Point1(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point1(p)) => {
                    let mut result = blst_p1::default();
                    unsafe { blst_p1_mult(&mut result, &p, s.as_ptr(), s.len() * 8); }
                    Ok(Value::Point1(result))
                },
                (Value::Point2(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point2(p)) => {
                    let mut result = blst_p2::default();
                    debug!("num bits: {:?}", s.len() * 8);
                    unsafe { blst_p2_mult(&mut result, &p, s.as_ptr(), s.len() * 8); }
                    Ok(Value::Point2(result))
                },
                (Value::Coord1(a), Value::Coord1(b)) => {
                    let mut result = blst_fp::default();
                    unsafe { blst_fp_mul(&mut result, &a, &b)}
                    Ok(Value::Coord1(result))
                },
                (Value::Coord2(a), Value::Coord2(b)) => {
                    let mut result = blst_fp2::default();
                    unsafe { blst_fp2_mul(&mut result, &a, &b)}
                    Ok(Value::Coord2(result))
                },
                (Value::Coord12(a), Value::Coord12(b)) => {
                    let mut result = blst_fp12::default();
                    unsafe { blst_fp12_mul(&mut result, &a, &b)}
                    Ok(Value::Coord12(result))
                },
                (l,r) => Err(EvalError::TypeMismatchMul {
                    lhs_span: lhs.miette_span(),
                    rhs_span: rhs.miette_span(),
                    lhs_type: l.to_type_string(),
                    rhs_type: r.to_type_string(),
                }),
            }
        }

        Expr::Add(lhs, rhs, _) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
            match (l, r) {
                (Value::Scalar(s1), Value::Scalar(s2)) => {
                    let r = BigUint::from_bytes_le(&s1) + BigUint::from_bytes_le(&s2);
                    Ok(Value::Scalar(r.to_bytes_le().into_boxed_slice()))
                },
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
                (Value::Coord1(a), Value::Coord1(b)) => {
                    let mut out = blst_fp::default();
                    unsafe { blst_fp_add(&mut out, &a, &b); }
                    Ok(Value::Coord1(out))
                }
                (Value::Coord2(a), Value::Coord2(b)) => {
                    let mut out = blst_fp2::default();
                    unsafe { blst_fp2_add(&mut out, &a, &b); }
                    Ok(Value::Coord2(out))
                }
                (l,r) => Err(EvalError::TypeMismatchPointAdd {
                        lhs_span: lhs.miette_span(),
                        rhs_span: rhs.miette_span(),
                        lhs_type: l.to_type_string(),
                        rhs_type: r.to_type_string(),
                    })
            }
        }
        Expr::Sub(lhs, rhs, _) => {
            let l = eval(lhs, env)?;
            let r = eval(rhs, env)?;
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
                (Value::Coord1(a), Value::Coord1(b)) => {
                    let mut out = blst_fp::default();
                    unsafe { blst_fp_sub(&mut out, &a, &b); }
                    Ok(Value::Coord1(out))
                }
                (Value::Coord2(a), Value::Coord2(b)) => {
                    let mut out = blst_fp2::default();
                    unsafe { blst_fp2_sub(&mut out, &a, &b); }
                    Ok(Value::Coord2(out))
                }
                (l,r) => Err(EvalError::TypeMismatchPointSub {
                    lhs_span: lhs.miette_span(),
                    rhs_span: rhs.miette_span(),
                    lhs_type: l.to_type_string(),
                    rhs_type: r.to_type_string(),
                }),
            }
        }
        Expr::Neg(inner, _) => {
            let v = eval(inner, env)?;
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
                _ => Err(EvalError::TypeMismatchNeg {
                    span: inner.miette_span(),
                    expr_type: v.to_type_string(),
                }),
            }
        }
        Expr::Var(name, _) => {
            env.get(name).cloned().ok_or(EvalError::UnknownIdent {
                span: expr.miette_span(),
            })
        },

        Expr::List(es, _) => {
            let vs = &es.into_iter().map(|e| eval(e, env)).collect::<Result<Vec<_>,EvalError>>()?;
            // Check all Values have the same type
            let val_0 = vs[0].to_owned();
            let opt_val_wrong_index = vs.into_iter().position(|v| !v.equal_type(&val_0));
            match opt_val_wrong_index {
              Some(i) =>
                Err(EvalError::ListElemsMustBeAllOfSameType {
                    first_span: es[0].miette_span(),
                    first_type: val_0.to_type_string(),
                    wrong_span: es[i].miette_span(),
                    wrong_type: vs[i].to_type_string(),
                }),
              None => {
                Ok(Value::List(vs.to_owned()))
              }
            }
        }
    }
}

enum Curve {
    E1,
    E2
}


fn check_deserialize_error(err: BLST_ERROR, span: (usize, usize), curve: Curve) -> Result<(),EvalError> {
    let point_type = || {
        match curve {
            Curve::E1 => "Point1",
            Curve::E2 => "Point2"
        }.to_string()
    };

    if err == BLST_ERROR::BLST_SUCCESS { return Ok(()); }
    let e = match err {
        BLST_ERROR::BLST_POINT_NOT_IN_GROUP => EvalError::PointNotInGroup {
            span,
            pt_type: point_type(),
            group: match curve { Curve::E1 => "G1", Curve::E2 => "G2"}.to_string(),

        },
        BLST_ERROR::BLST_POINT_NOT_ON_CURVE => EvalError::PointNotOnCurve {
            span,
            pt_type: point_type(),
            curve: (match curve { Curve::E1 => "E1", Curve::E2 => "E2"}).to_string(),
        },
        _ => EvalError::InvalidCompressedPoint {
            span,
            pt_type: point_type(),
        },
    };
    Err(e)
}

fn fp_pow(x: blst_fp, e: blst_fp) -> blst_fp {
    let mut exp = [0u8; 48];
    unsafe { blst_bendian_from_fp(exp.as_mut_ptr(), &e)};
    let mut result = blst_fp::default();
    let mut bytes: [u8;48] = [0u8;48];
    bytes[bytes.len() - 1] = 1u8;
    let mut acc = x.clone();
    unsafe { blst_fp_from_bendian(&mut result, bytes.as_ptr()) };
    for limb in exp.iter().rev() {
        for i in 0..8 {
            if ((limb >> i) & 1) != 0 {
                // result = result * acc
                unsafe { blst_fp_mul(&mut result, &result, &acc) };
            }
            unsafe { blst_fp_sqr(&mut acc, &acc) };
        }
    }
    result
}



