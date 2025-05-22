use blst::*;
use std::collections::HashMap;


#[derive(Debug)]
pub enum EvalError {
    CannotRedefineBuiltin,
    InvalidCompressedPoint,
    NotASquare(String),
    TypeMismatchCoord1,
    TypeMismatchMul,
    TypeMismatchPoint1Add,
    TypeMismatchPoint1Coord1,
    TypeMismatchPoint1Neg,
    TypeMismatchPoint1Sub,
    TypeMismatchPoint2Coord2,
    UnknownIdent(String),
}

#[derive(Clone, Debug)]
pub enum Value {
    Point1(blst_p1),
    Point2(blst_p2),
    Coord1(blst_fp),
    Coord2(blst_fp2),
    Scalar(blst_scalar),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::CannotRedefineBuiltin => write!(f, "Cannot redefine a built-in name"),
            EvalError::InvalidCompressedPoint => write!(f, "Invalid compressed point"),
            EvalError::NotASquare(n) => write!(f, "No square root for {}", n),
            EvalError::TypeMismatchCoord1 => write!(f, "Expected Coord1 operand"),
            EvalError::TypeMismatchMul => write!(f, "Expected Scalar * Point1 or Point1 * Scalar"),
            EvalError::TypeMismatchPoint1Add => write!(f, "Can only add Point1 operands"),
            EvalError::TypeMismatchPoint1Coord1 => write!(f, "Can only have Coord1 coordinates when constructing Point1"),
            EvalError::TypeMismatchPoint1Neg => write!(f, "Can only negate Point1 operands"),
            EvalError::TypeMismatchPoint1Sub => write!(f, "Can only subtract Point1 operands"),
            EvalError::TypeMismatchPoint2Coord2 => write!(f, "Can only have Coord2 coordinates when constructing Point2"),
            EvalError::UnknownIdent(s) => write!(f, "Unknown identifier: {}", s),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Point1(p) => {
                let mut aff = blst_p1_affine::default();
                unsafe { blst_p1_to_affine(&mut aff, p) };
                write!(f, "x: {}\ny: {}\n : Point1", fp_to_hex(&aff.x), fp_to_hex(&aff.y))
            }
            Value::Point2(p) => {
                let mut aff = blst_p2_affine::default();
                unsafe { blst_p2_to_affine(&mut aff, p) };
                write!(f, "x: {}\ny: {}\n : Point2", fp2_to_hex(&aff.x), fp2_to_hex(&aff.y))
            }
            Value::Scalar(s) => write!(f, "{} : Scalar", scalar_to_hex(s)),
            Value::Coord1(fp) => write!(f, "{} : Coord1", fp_to_hex(fp)),
            Value::Coord2(fp2) => write!(f, "{} : Coord2", fp2_to_hex(fp2)),
        }
    }
}

impl Value {
    pub fn describe(&self) -> String {
        match self {
            Value::Point1(p) => {
                let mut aff = blst_p1_affine::default();
                unsafe { blst_p1_to_affine(&mut aff, p) };
                let mut compressed = [0u8; 48];
                unsafe { blst_p1_compress(compressed.as_mut_ptr(), p) };
                let on_curve = unsafe { blst_p1_affine_on_curve(&aff) };
                let in_g1 = unsafe { blst_p1_in_g1(p) };
                let is_inf = unsafe { blst_p1_is_inf(p) };
                format!(
                    "x: {}\ny: {}\n: Point1\nOn curve: {}\nIn G1: {}\nInfinity: {}\nCompressed: 0x{}",
                    fp_to_hex(&aff.x),
                    fp_to_hex(&aff.y),
                    on_curve,
                    in_g1,
                    is_inf,
                    hex::encode(compressed)
                )
            }
            _ => format!("{}", self),
        }
    }
}

pub fn fp_to_hex(fp: &blst_fp) -> String {
    let mut out = [0u8; 48];
    unsafe { blst_bendian_from_fp(out.as_mut_ptr(), fp) };
    format!("0x{}", hex::encode(out))
}

pub fn fp2_to_hex(fp2: &blst_fp2) -> String {
    let mut a = [0u8; 48];
    let mut b = [0u8; 48];
    unsafe { blst_bendian_from_fp(a.as_mut_ptr(), &fp2.fp[0]) };
    unsafe { blst_bendian_from_fp(b.as_mut_ptr(), &fp2.fp[1]) };
    format!("0x{} + 0x{}*i", hex::encode(a), hex::encode(b))
}


pub fn scalar_to_hex(s: &blst_scalar) -> String {
    let mut out = [0u8; 32];
    unsafe { blst_bendian_from_scalar(out.as_mut_ptr(), s) };
    format!("0x{}", hex::encode(out))
}

pub fn builtin_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();
    env.insert("p1inf".to_string(), Value::Point1(blst_p1::default()));
    env.insert("p2inf".to_string(), Value::Point2(blst_p2::default()));
    let g1 = unsafe { *blst_p1_generator() };
    env.insert("g1".to_string(), Value::Point1(g1));
    let g2 = unsafe { *blst_p2_generator() };
    env.insert("g2".to_string(), Value::Point2(g2));
    env
}

pub fn is_builtin(name: &str) -> bool {
    matches!(name, "p1inf" | "p2inf" | "g1" | "g2")
}
