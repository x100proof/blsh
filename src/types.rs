use blst::*;
use num_bigint::BigUint;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};
use pretty::{RcDoc};
use std::ops::Add;
use ansi_term::Colour::{Yellow, Blue, Cyan, Fixed};
use ansi_term::Style;
use atty::Stream;

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
    TypeMismatchMapToG1,
    TypeMismatchMapToG2,
    TypeMismatchPointAdd {
        source: String,
        lhs_span: (usize, usize),
        rhs_span: (usize, usize),
        lhs_type: String,
        rhs_type: String,
    },
    TypeMismatchNeg,
    TypeMismatchPointSub,
    UnknownIdent(String),
}

#[derive(Clone, Debug)]
pub enum Value {
    // PkScalar(blst_scalar), // for secret keys
    Scalar(Box<[u8]>),
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

#[derive(Clone,Copy, Debug)]
enum Brightness {
    Light,
    Medium,
    Dark
}

impl Brightness {
    fn darken(self) -> Self {
        match self {
            Self::Light => Self::Medium,
            _           => Self::Dark,
        }
    }
}

struct PP<'a>(RcDoc<'a>);

impl<'a> Add for PP<'a> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        PP(self.0.append(rhs.0))
    }
}

fn supports_color() -> bool {
    atty::is(Stream::Stdout)
}

fn styled<'a>(s: &str, style: Style) -> PP<'a> {
    if supports_color() {
        PP(RcDoc::text(style.paint(s).to_string()))
    } else {
        PP(RcDoc::text(s.to_string()))
    }
}

fn type_pp<'a>(s: &str) -> PP<'a> {
    styled(s, Blue.bold())
}

fn coord_pp<'a>(s: &str) -> PP<'a> {
    styled(s, Yellow.normal())
}

fn ident<'a>(s: &str) -> PP<'a> {
    styled(s, Fixed(129).normal())
}

fn scalar_pp<'a>(s: &str) -> PP<'a> {
    styled(s, Cyan.normal())
}

fn punct<'a>(s: &str, b: Brightness) -> PP<'a> {
    let i = match b {
        Brightness::Dark   => 240,
        Brightness::Medium => 245,
        Brightness::Light  => 250,
    };
    styled(s, Fixed(i).normal())
}

fn line<'a>() -> PP<'a> { PP(RcDoc::line()) }
fn line_<'a>() -> PP<'a> { PP(RcDoc::line_()) }
fn space<'a>() -> PP<'a> { PP(RcDoc::space()) }
// fn text<'a>(s: &'a str) -> PP<'a> { PP(RcDoc::text(s))}
fn concat<'a, I>(docs: I) -> PP<'a>
    where I: IntoIterator<Item = PP<'a>>,
    {
        PP(RcDoc::concat(docs.into_iter().map(|x| x.0)))
    }

#[allow(dead_code)]
impl<'a> PP<'a> {
    fn group(self) -> Self {
        PP(self.0.group())
    }

    fn render(self, width: usize, w: &mut Vec<u8>) -> Result<(), std::io::Error> {
       self.0.render(width, w)
    }

    fn nest(self, i: isize) -> Self {
        PP(self.0.nest(i))
    }

    fn hang(self, adjust: isize) -> Self {
        PP(self.0.hang(adjust))
    }

    fn indent(self, adjust: usize) -> Self {
        PP(self.0.indent(adjust))
    }

    fn with_type(self, type_str: &'a str) -> Self {
        self + concat(
           vec![line(),
                concat(vec![punct(":",Brightness::Light), line(), type_pp(type_str)]).group()
               ]).group()
    }

    fn wrap_brackets(self, brackets: Brackets, b: Brightness) -> Self {
        match brackets {
            Brackets::Brace  => concat(vec![punct("(", b) + space(), self, space() + punct(")", b)]),
            Brackets::Curly  => concat(vec![punct("{", b) + space(), self, space() + punct("}", b)]),
            Brackets::Square => concat(vec![punct("[", b) + space(), self, space() + punct("]", b)]),
        }
    }
}

fn pretty_scalar<'a>(s: Box<[u8]>) -> PP<'a> {
  scalar_pp(&scalar_to_hex(s))
}

fn pretty_pair<'a>(x: PP<'a>, y: PP<'a>, brackets: Brackets, b: Brightness) -> PP<'a> {
    concat(vec![x, line_(), punct(",", b), space(), y]).wrap_brackets(brackets, b).group()
}

fn pretty_triple<'a>(x: PP<'a>, y: PP<'a>, z: PP<'a>, brackets: Brackets, b: Brightness) -> PP<'a> {
    concat(vec![
        x, line_(),
        punct(",", b), space(), y, line_(),
        punct(",", b), space(), z
    ]).wrap_brackets(brackets, b).group()
}

fn pretty_point1<'a>(p: &blst_p1) -> PP<'a> {
    let mut aff = blst_p1_affine::default();
    unsafe { blst_p1_to_affine(&mut aff, p) };
    pretty_pair(pretty_coord1(&aff.x).nest(2),
                pretty_coord1(&aff.y).nest(2),
                Brackets::Brace,
                Brightness::Light)
}

fn pretty_point2<'a>(p: &blst_p2) -> PP<'a> {
    let b = Brightness::Light;
    let mut aff = blst_p2_affine::default();
    unsafe { blst_p2_to_affine(&mut aff, p) };
    let a_doc = pretty_coord2_rec(&aff.x, b.darken()).nest(2);
    let b_doc = pretty_coord2_rec(&aff.y, b.darken()).nest(2);
    pretty_pair(a_doc, b_doc, Brackets::Brace, b)
}

fn pretty_coord1<'a>(fp: &blst_fp) -> PP<'a> {
    coord_pp(&fp_to_hex(fp))
}

fn pretty_coord2_rec<'a>(fp2: &blst_fp2, b: Brightness) -> PP<'a> {
    pretty_pair(pretty_coord1(&fp2.fp[0]), pretty_coord1(&fp2.fp[1]), Brackets::Square, b)
}

fn pretty_coord2<'a>(fp2: &blst_fp2) -> PP<'a> {
    pretty_coord2_rec(fp2, Brightness::Light)
}

fn pretty_coord6_rec<'a>(fp6: &blst_fp6, b: Brightness) -> PP<'a> {
    pretty_triple(
        pretty_coord2_rec(&fp6.fp2[0], b.darken()).nest(2),
        pretty_coord2_rec(&fp6.fp2[1], b.darken()).nest(2),
        pretty_coord2_rec(&fp6.fp2[2], b.darken()).nest(2),
        Brackets::Curly,
        b
    )
}

fn pretty_coord6<'a>(fp6: &blst_fp6) -> PP<'a> {
    pretty_coord6_rec(fp6, Brightness::Light)
}


fn pretty_coord12<'a>(fp12: &blst_fp12) -> PP<'a> {
    let b = Brightness::Light;
    pretty_pair(pretty_coord6_rec(&fp12.fp6[0], b.darken()).nest(2),
                pretty_coord6_rec(&fp12.fp6[1], b.darken()).nest(2),
                Brackets::Square, b)
}

fn pretty_value<'a>(v: &Value) -> PP<'a> {
    match v {
        Value::Point1(p) => pretty_point1(p).with_type("Point1"),
        Value::Point2(p) => pretty_point2(p).with_type("Point2"),
        Value::Scalar(s) => pretty_scalar(s.clone()).with_type("Scalar"),
        Value::Coord1(fp) => pretty_coord1(fp).with_type("Coord1"),
        Value::Coord2(fp2) => pretty_coord2(fp2).with_type("Coord2"),
        Value::Coord6(fp6) => pretty_coord6(fp6).with_type("Coord6"),
        Value::Coord12(fp12) => pretty_coord12(fp12).with_type("Coord12"),
    }
}

fn pretty_let<'a>(name: &str, value: &Value) -> PP<'a> {
    (ident(name) + space() + punct("=", Brightness::Light) + space() + pretty_value(value).hang(0)).group()
}

fn pretty_to_string<'a>(doc: PP<'a>) -> String {
        let width = get_terminal_width(80); // recalculate always
        let mut w = Vec::new();
        doc.render(width, &mut w).unwrap();
        String::from_utf8(w).unwrap()
}

pub fn pretty_let_to_string(name: &str, value: &Value) -> String {
    pretty_to_string(pretty_let(name, value))
}

fn pretty_error_add_mismatch<'a>(source: &str, lhs_span: (usize, usize), rhs_span: (usize, usize),
                   lhs_type: &str, rhs_type: &str, message: &str) -> PP<'a> {
    let (lhs_start, lhs_end) = lhs_span;
    let (rhs_start, rhs_end) = rhs_span;

    // Source line
    let source_pp = PP(RcDoc::text(source.to_string()));

    // Underlines for LHS and RHS
    let lhs_underline = concat(vec![
        PP(RcDoc::text(" ".repeat(lhs_start))),
        styled(&"^".repeat(lhs_end - lhs_start), ansi_term::Colour::Red.bold())
    ]);

    let rhs_underline = concat(vec![
        PP(RcDoc::text(" ".repeat(rhs_start))),
        styled(&"^".repeat(rhs_end - rhs_start), ansi_term::Colour::Red.bold())
    ]);

    // Ensure we have valid span widths
    let lhs_width = if lhs_end > lhs_start { lhs_end - lhs_start } else { 1 };
    let rhs_width = if rhs_end > rhs_start { rhs_end - rhs_start } else { 1 };

    // Calculate center points for arrows
    let lhs_center = lhs_start + (lhs_width / 2);
    let rhs_center = rhs_start + (rhs_width / 2);

    // Create vertical line annotations in Rust compiler style
    let mut annotation_line1 = String::new();
    let mut annotation_line2 = String::new();

    // First line with vertical bars
    for i in 0..=rhs_center + 1 {
        if i == lhs_center {
            annotation_line1.push('|');
        } else if i == rhs_center {
            annotation_line1.push('|');
        } else {
            annotation_line1.push(' ');
        }
    }

    // Second line with type annotations
    // Left type annotation
    let lhs_annotation = format!("- {}", lhs_type);
    let rhs_annotation = format!("- {}", rhs_type);

    // Add left annotation
    for mut i in 0..=rhs_center + rhs_annotation.len() + 1 {
        if i == lhs_center {
            annotation_line2.push('+');
            for (j, c) in lhs_annotation.chars().enumerate() {
                if i + j + 1 <= annotation_line2.len() {
                    break;
                }
                annotation_line2.push(c);
            }
            i += lhs_annotation.len();
        } else if i == rhs_center {
            annotation_line2.push('+');
            for c in rhs_annotation.chars() {
                annotation_line2.push(c);
            }
            break;
        } else if annotation_line2.len() < i {
            annotation_line2.push(' ');
        }
    }

    // Style the annotation lines
    let annotation_pp1 = styled(&annotation_line1, ansi_term::Colour::Blue.bold());
    let annotation_pp2 = styled(&annotation_line2, ansi_term::Colour::Blue.bold());

    // Error message
    let error_pp = styled(&format!("Error: {}", message), ansi_term::Colour::Red.bold());

    // Combine everything
    concat(vec![
        source_pp, line(),
        concat(vec![lhs_underline, rhs_underline]), line(),
        annotation_pp1, line(),
        annotation_pp2, line(),
        error_pp
    ])
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
            EvalError::TypeMismatchMapToG1     => write!(f, "map_to_g1 can only be applied to Coord1 types"),
            EvalError::TypeMismatchMapToG2     => write!(f, "map_to_g2 can only be applied to Coord2 types"),
            EvalError::TypeMismatchPointAdd { source, lhs_span, rhs_span, lhs_type, rhs_type } => {
                if !supports_color() {
                    return write!(f, "Can only add points (of the same type)");
                }

                let error_doc = pretty_error_add_mismatch(
                    source,
                    *lhs_span,
                    *rhs_span,
                    lhs_type,
                    rhs_type,
                    "Can only add points (of the same type)"
                );

                write!(f, "{}", pretty_to_string(error_doc))
            },
            EvalError::TypeMismatchNeg         => write!(f, "Can only negate Point1, Point2, Coord1, Coord2"),
            EvalError::TypeMismatchPointSub    => write!(f, "Can only subtract points (of the same type)"),
            EvalError::UnknownIdent(s) => write!(f, "Unknown identifier: {}", s),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = pretty_to_string(pretty_value(self));
        write!(f, "{}", str)
    }
}

pub fn value_to_type(value: &Value) -> String {
    match value {
        Value::Scalar(_) => "Scalar".to_string(),
        Value::Point1(_) => "Point1".to_string(),
        Value::Point2(_) => "Point2".to_string(),
        Value::Coord1(_) => "Coord1".to_string(),
        Value::Coord2(_) => "Coord2".to_string(),
        Value::Coord6(_) => "Coord6".to_string(),
        Value::Coord12(_) => "Coord12".to_string(),
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

pub fn scalar_to_hex(s: Box<[u8]>) -> String {
    let mut vec = s.into_vec();
    vec.reverse();
    format!("0x{}", hex::encode(vec))
}

pub fn hex_to_scalar(s: &str) -> Box<[u8]> {
    let s = s.strip_prefix("0x").unwrap_or(s); // optional prefix
    let n = BigUint::parse_bytes(s.as_bytes(), 16).unwrap();
    n.to_bytes_le().into_boxed_slice()
}

pub fn builtin_env() -> HashMap<String, Value> {
    let mut env = HashMap::new();
    for (key, value) in builtins() {
      env.insert(key.to_string(), value);
    }
    env
}

fn builtins() -> [(String, Value); 10] {
    [ ("p1inf", Value::Point1(blst_p1::default())),
      ("p2inf", Value::Point2(blst_p2::default())),
      ("g1", Value::Point1(unsafe { *blst_p1_generator() })),
      ("g2", Value::Point2(unsafe { *blst_p2_generator() })),
      ("bls_q", Value::Scalar(hex_to_scalar("0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab"))),
      ("bls_p", Value::Scalar(hex_to_scalar("0x73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"))),
      ("order_e1", Value::Scalar(hex_to_scalar("0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb15400008c0000000000aaab"))),
      ("order_e2", Value::Scalar(hex_to_scalar("0x2a437a4b8c35fc74bd278eaa22f25e9e2dc90e50e7046b466e59e49349e8bd050a62cfd16ddca6ef53149330978ef0137697386bf984315744a2d5eb3dd4d213f2484c55b94474ab096de2c62640b2643116b1e2788e6a8b2a9fffe1c7238e5"))),
      ("h1", Value::Scalar(hex_to_scalar("0x396c8c005555e1568c00aaab0000aaab"))),
      ("h2", Value::Scalar(hex_to_scalar("0x5d543a95414e7f1091d50792876a202cd91de4547085abaa68a205b2e5a7ddfa628f1cb4d9e82ef21537e293a6691ae1616ec6e786f0c70cf1c38e31c7238e5"))),
    ].map(|(s,v)| (s.to_string(), v))
}

pub fn is_builtin(name: &str) -> bool {
    builtins().iter().find(|(s, _)| *s == name).is_some()
}

fn get_terminal_width(default: usize) -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        default
    }
}
