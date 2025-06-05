use blst::*;
use num_bigint::BigUint;
use std::collections::HashMap;
use terminal_size::{Width, terminal_size};
use pretty::{RcDoc};
use std::ops::Add;
use ansi_term::Colour::{Yellow, Blue, Cyan, Fixed};
use ansi_term::Style;
use atty::Stream;

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
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

fn pretty_bool<'a>(b: &bool) -> PP<'a> {
    styled(if *b { "true" } else { "false" } , Fixed(208).normal())
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
        Value::Bool(b) => pretty_bool(b).with_type("Bool"),
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = pretty_to_string(pretty_value(self));
        write!(f, "{}", str)
    }
}


impl Value {
    pub fn to_type(&self) -> String {
        match self {
            Value::Scalar(_)  => "Scalar",
            Value::Point1(_)  => "Point1",
            Value::Point2(_)  => "Point2",
            Value::Coord1(_)  => "Coord1",
            Value::Coord2(_)  => "Coord2",
            Value::Coord6(_)  => "Coord6",
            Value::Coord12(_) => "Coord12",
            Value::Bool(_)    => "Bool",
        }.to_string()
    }

    pub fn equal_type(&self, other: &Self) -> bool {
        self.to_type() == other.to_type()
    }

    // Will panic if ever called on a value of different type
    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
          (Value::Scalar(bs1), Value::Scalar(bs2)) => bs1 == bs2,
          (Value::Point1(a), Value::Point1(b)) => a == b,
          (Value::Point2(a), Value::Point2(b)) => a == b,
          (Value::Coord2(a), Value::Coord2(b)) => a == b,
          (Value::Coord1(a), Value::Coord1(b)) => a == b,
          (Value::Coord6(a), Value::Coord6(b)) => a == b,
          (Value::Coord12(a), Value::Coord12(b)) => a == b,
          (Value::Bool(a), Value::Bool(b)) => a == b,
          _ => panic!("is_equal should never be called on different types"),
        }
    }

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

// These are words that could clash with identifiers
fn non_builtin_reserved_words() ->  [String; 2] {
    [ "true", "false"].map(|s|s.to_string())
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

// FIXME: This code is so incredibly wasteful of resources
pub fn is_reserved_word(name: &str) -> bool {
    let builtins = builtins();
    let non_builtin_reserved_words = non_builtin_reserved_words();
    let mut combined= builtins.iter().map(|(s,_)| s).chain(non_builtin_reserved_words.iter());
    combined.find(|s| *s == name).is_some()
}

// pub fn is_builtin(name: &str) -> bool {
//     builtins().iter().find(|(s, _)| *s == name).is_some()
// }

fn get_terminal_width(default: usize) -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        default
    }
}

