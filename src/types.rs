use blst::*;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};
use pretty::{RcDoc};
use std::ops::Add;

#[derive(Debug)]
pub enum EvalError {
    CannotRedefineBuiltin,
    EmbedHasIncorrectTypes,
    PointNotOnCurve,
    PointNotInGroup,
    PointCanOnlyContainCoord, // FIXME: Show what types were in it
    PairCanOnlyContainCoord1Or6,
    TripleCanOnlyContainCoord2,
    InvalidCompressedPoint,
    NotASquare(String),
    TypeMismatchCoord1,
    TypeMismatchMul,
    TypeMismatchPoint1Add,
    TypeMismatchPoint1Neg,
    TypeMismatchPoint1Sub,
    UnknownIdent(String),
}

#[derive(Clone, Debug)]
pub enum Value {
    Scalar(blst_scalar),
    Point1(blst_p1),
    Point2(blst_p2),
    Coord1(blst_fp),
    Coord2(blst_fp2),
    Coord6(blst_fp6),
    Coord12(blst_fp12),
}


enum Brackets {
    Brace,
    Curly,
    Square
}


// impl<'a> Add for RcDoc<'a> {
//     fn add(self, other: RcDoc<'a>) -> RcDoc<'a> {
//         self.append(other)
//     }
// }

trait PPrintExt {
    fn with_type(self, type_str: String) -> Self;
    fn wrap_brackets(self, brackets: Brackets) -> Self;
}

impl<'a> PPrintExt for RcDoc<'a> {
    fn with_type(self, type_str: String) -> Self {
        self.append(RcDoc::concat(
           vec![RcDoc::line(),
                    RcDoc::concat(vec![RcDoc::text(":"), RcDoc::line(), RcDoc::text(type_str)]).group()
                     ]).group()
        )
    }

   fn wrap_brackets(self, brackets: Brackets) -> Self {
        match brackets {
            Brackets::Brace  => RcDoc::concat(vec![RcDoc::text("(").append(RcDoc::space()), self, RcDoc::space().append(RcDoc::text(")"))]),
            Brackets::Curly  => RcDoc::concat(vec![RcDoc::text("{").append(RcDoc::space()), self, RcDoc::space().append(RcDoc::text("}"))]),
            Brackets::Square => RcDoc::concat(vec![RcDoc::text("[").append(RcDoc::space()), self, RcDoc::space().append(RcDoc::text("]"))]),
        }
    }
}

fn pretty_pair<'a>(x: RcDoc<'a>, y: RcDoc<'a>, brackets: Brackets) -> RcDoc<'a> {
    RcDoc::concat(vec![x, RcDoc::line_(), RcDoc::text(","), RcDoc::space(), y]).wrap_brackets(brackets).group()
}

fn pretty_triple<'a>(x: RcDoc<'a>, y: RcDoc<'a>, z: RcDoc<'a>, brackets: Brackets) -> RcDoc<'a> {
    RcDoc::concat(vec![
        x, RcDoc::line_(),
        RcDoc::text(","), RcDoc::space(), y, RcDoc::line_(),
        RcDoc::text(","), RcDoc::space(), z
    ]).wrap_brackets(brackets).group()
}

fn pretty_point1<'a>(p: &blst_p1) -> RcDoc<'a> {
    let mut aff = blst_p1_affine::default();
    unsafe { blst_p1_to_affine(&mut aff, p) };
    pretty_pair(pretty_coord1(&aff.x).nest(2),
                pretty_coord1(&aff.y).nest(2),
                Brackets::Brace)
}

fn pretty_point2(p: &blst_p2) -> RcDoc<'static> {
    let mut aff = blst_p2_affine::default();
    unsafe { blst_p2_to_affine(&mut aff, p) };
    let a_doc = pretty_coord2(&aff.x).nest(2);
    let b_doc = pretty_coord2(&aff.y).nest(2);
    pretty_pair(a_doc, b_doc, Brackets::Brace)
}

fn pretty_coord1<'a>(fp: &blst_fp) -> RcDoc<'a> {
    RcDoc::as_string(fp_to_hex(fp))
}

fn pretty_coord2<'a>(fp2: &blst_fp2) -> RcDoc<'a> {
    pretty_pair(pretty_coord1(&fp2.fp[0]), pretty_coord1(&fp2.fp[1]), Brackets::Square)
}

fn pretty_coord6<'a>(fp6: &blst_fp6) -> RcDoc<'a> {
    pretty_triple(
        pretty_coord2(&fp6.fp2[0]).nest(2),
        pretty_coord2(&fp6.fp2[1]).nest(2),
        pretty_coord2(&fp6.fp2[2]).nest(2),
        Brackets::Curly)
}

fn pretty_coord12<'a>(fp12: &blst_fp12) -> RcDoc<'a> {
    pretty_pair(pretty_coord6(&fp12.fp6[0]).nest(2),
                pretty_coord6(&fp12.fp6[1]).nest(2),
                Brackets::Square)
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::CannotRedefineBuiltin    => write!(f, "Cannot redefine a built-in name"),
            EvalError::EmbedHasIncorrectTypes   => write!(f, "Can only call `embed(x,y)` if x: Point1 and y: Point2"),
            EvalError::InvalidCompressedPoint   => write!(f, "Invalid compressed point"),
            EvalError::PointNotOnCurve          => write!(f, "Point is not on curve"), // FIXME: Make Point1 and Point2 specific
            EvalError::PointNotInGroup          => write!(f, "Point is not in group"), // FIXME: Make Point1 and Point2 specific G1 vs G2
            EvalError::PointCanOnlyContainCoord => write!(f, "Points can only contain Coord1/Coord2 values in them"),
            EvalError::PairCanOnlyContainCoord1Or6 => write!(f, "Coordinate pairs can only contain Coord1 or Coord6 values"),
            EvalError::TripleCanOnlyContainCoord2 => write!(f, "Coordinate triples can only contain Coord2 values"),
            EvalError::NotASquare(n)   => write!(f, "No square root for {}", n),
            EvalError::TypeMismatchCoord1       => write!(f, "Expected Coord1 operand"),
            EvalError::TypeMismatchMul          => write!(f, "Scalars can only be multiplied with points"),
            EvalError::TypeMismatchPoint1Add    => write!(f, "Can only add points (of the same type)"),
            EvalError::TypeMismatchPoint1Neg    => write!(f, "Can only negate points"),
            EvalError::TypeMismatchPoint1Sub    => write!(f, "Can only subtract points (of the same type)"),
            EvalError::UnknownIdent(s) => write!(f, "Unknown identifier: {}", s),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = get_terminal_width(80); // recalculate always
        let doc = match self {
            Value::Point1(p) => pretty_point1(p).with_type("Point1".to_string()),
            Value::Point2(p) => pretty_point2(p).with_type("Point2".to_string()),
            Value::Scalar(s) => RcDoc::as_string(scalar_to_hex(s)).with_type("Scalar".to_string()),
            Value::Coord1(fp) => pretty_coord1(fp).with_type("Coord1".to_string()),
            Value::Coord2(fp2) => pretty_coord2(fp2).with_type("Coord2".to_string()),
            Value::Coord6(fp6) => pretty_coord6(fp6).with_type("Coord6".to_string()),
            Value::Coord12(fp12) => pretty_coord12(fp12).with_type("Coord12".to_string()),
        };
        let mut w = Vec::new();
        let utf8 = doc.render(width, &mut w).unwrap();
        write!(f, "{}", String::from_utf8(w).unwrap())
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
                    "{}\nOn curve: {}\nIn G1:    {}\nInfinity: {}\nCompressed: 0x{}",
                    format!("{}", self),
                    on_curve,
                    in_g1,
                    is_inf,
                    hex::encode(compressed)
                )
            }
            Value::Point2(p) => {
                let mut aff = blst_p2_affine::default();
                unsafe { blst_p2_to_affine(&mut aff, p) };
                let mut compressed = [0u8; 96];
                unsafe { blst_p2_compress(compressed.as_mut_ptr(), p) };
                let on_curve = unsafe { blst_p2_affine_on_curve(&aff) };
                let in_g2 = unsafe { blst_p2_in_g2(p) };
                let is_inf = unsafe { blst_p2_is_inf(p) };
                format!(
                    "{}\nOn curve: {}\nIn G2:    {}\nInfinity: {}\nCompressed: 0x{}",
                    format!("{}", self),
                    on_curve,
                    in_g2,
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

fn get_terminal_width(default: usize) -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        default
    }
}