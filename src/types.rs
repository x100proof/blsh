use blst::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum ParseError {
    InvalidNumber,
    InvalidCompressedPoint,
    UnexpectedToken,
    BareNumber,
}

#[derive(Debug)]
pub enum EvalError {
    InvalidExpression,
    MismatchedOperands,
    NotASquare,
    InvalidCompressedPoint,
    CannotRedefineBuiltin,
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
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Point1(p) => {
                let mut aff = blst_p1_affine::default();
                unsafe { blst_p1_to_affine(&mut aff, p) };
                write!(f, "x: {}\ny: {}\n : Point", fp_to_hex(&aff.x), fp_to_hex(&aff.y))
            }
            Value::Scalar(s) => write!(f, "{} : Scalar", scalar_to_hex(s)),
            Value::Coord1(fp) => write!(f, "{} : Coord1", fp_to_hex(fp)),
            _ => write!(f, ": Unsupported display"),
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
            _ => format!("{}", self), // Fallback
        }
    }
}


fn fp_to_hex(fp: &blst_fp) -> String {
    let mut out = [0u8; 48];
    unsafe { blst_bendian_from_fp(out.as_mut_ptr(), fp) };
    format!("0x{}", hex::encode(out))
}

fn scalar_to_hex(s: &blst_scalar) -> String {
    let mut out = [0u8; 32];
    unsafe { blst_bendian_from_scalar(out.as_mut_ptr(), s) };
    format!("0x{}", hex::encode(out))
}


impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn builtin_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();

    let inf = blst_p1::default();
    env.insert("p1inf".to_string(), Value::Point1(inf));
    let g1 = unsafe { *blst_p1_generator() };
    env.insert("g1".to_string(), Value::Point1(g1));

    env
}

pub fn is_builtin(name: &str) -> bool {
    matches!(name, "p1inf" | "g1")
}
