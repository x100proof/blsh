use blst::*;

#[derive(Clone)]
pub enum Value {
    Point1(blst_p1),
    Point2(blst_p2),
    Coord1(blst_fp),
    Coord2(blst_fp2),
    Scalar(blst_scalar),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidBigInt,
    ScalarTooBig,
    CoordTooBig,
    PointNotOnCurve,
    InvalidCompressedPoint,
    InvalidExpression,
    MismatchedOperands,
    NotASquare,
    TypeError(&'static str),
}

impl std::fmt::Display for ParseError {
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
                write!(
                    f,
                    "x: {}\ny: {}\n: Point1",
                    fp_to_hex(&aff.x),
                    fp_to_hex(&aff.y)
                )
            }
            Value::Scalar(s) => write!(f, "{} : Scalar", scalar_to_hex(s)),
            Value::Coord1(fp) => write!(f, "{} : Coord1", fp_to_hex(fp)),
            _ => write!(f, ": Unsupported display"),
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