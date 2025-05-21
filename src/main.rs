use blst::*;
use rustyline::Editor;
use rustyline::history::FileHistory;
use std::collections::HashMap;
use num_bigint::BigUint;
use std::str::FromStr;

#[derive(Clone)]
enum Value {
    Point1(blst_p1),
    Point2(blst_p2),
    Coord1(blst_fp),
    Coord2(blst_fp2),
    Scalar(blst_scalar),
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

fn parse_big_int(input: &str) -> Option<BigUint> {
    if let Some(hex) = input.strip_prefix("0x") {
        BigUint::parse_bytes(hex.as_bytes(), 16)
    } else {
        BigUint::from_str(input).ok()
    }
}

fn parse_let(line: &str) -> Option<(String, Value)> {
    if let Some(rest) = line.strip_prefix("let ") {
        let parts: Vec<&str> = rest.splitn(2, '=').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let name = parts[0].to_string();
            let expr = parts[1];
            let value = parse_expr(expr)?;
            return Some((name, value));
        }
    }
    None
}

fn parse_expr(expr: &str) -> Option<Value> {
    if expr.starts_with("scalar(") {
        let inner = expr.strip_prefix("scalar(")?.strip_suffix(")")?.trim();
        return Some(parse_scalar(inner)?);
    } else if expr.starts_with("coord1(") {
        let inner = expr.strip_prefix("coord(")?.strip_suffix(")")?.trim();
        return Some(Value::Coord1(parse_coord(inner)?));
    } else if expr.starts_with("point1(") {
        let inner = expr.strip_prefix("point(")?.strip_suffix(")")?.trim();
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return None;
        }
        return Some(parse_point(parts[0], parts[1])?);
    } else if expr.starts_with("point1c(") {
        let inner = expr.strip_prefix("point1c(")?.strip_suffix(")")?.trim();
        return Some(parse_point_compressed(inner)?);
    } else if expr.starts_with("sqrt(") {
        let inner = expr.strip_prefix("sqrt(")?.strip_suffix(")")?.trim();
        let coord = parse_coord(inner)?;
        if unsafe { blst_fp_is_square(&coord) } {
            let mut result = blst_fp::default();
            unsafe { blst_fp_sqrt(&mut result, &coord); }
            return Some(Value::Coord1(result));
        } else {
            println!("Coordinate has no square root");
            return None;
        }
    } else if expr.contains('*') {
        let parts: Vec<&str> = expr.split('*').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let lhs = parse_expr(parts[0])?;
            let rhs = parse_expr(parts[1])?;
            return Some(eval_mul(lhs, rhs)?);
        }
    }
    None
}

fn parse_scalar(input: &str) -> Option<Value> {
    let bigint = parse_big_int(input)?;
    let bytes = bigint.to_bytes_be();
    if bytes.len() > 32 {
        return None;
    }
    let mut padded = [0u8; 32];
    padded[32 - bytes.len()..].copy_from_slice(&bytes);
    let mut s = blst_scalar::default();
    unsafe { blst_scalar_from_bendian(&mut s, padded.as_ptr()) };
    Some(Value::Scalar(s))
}

fn parse_coord(input: &str) -> Option<blst_fp> {
    let bigint = parse_big_int(input)?;
    let bytes = bigint.to_bytes_be();
    if bytes.len() > 48 {
        return None;
    }
    let mut padded = [0u8; 48];
    padded[48 - bytes.len()..].copy_from_slice(&bytes);
    let mut fp = blst_fp::default();
    unsafe { blst_fp_from_bendian(&mut fp, padded.as_ptr()) };
    Some(fp)
}

fn parse_point(x: &str, y: &str) -> Option<Value> {
    let x_fp = parse_coord(x)?;
    let y_fp = parse_coord(y)?;

    let aff = blst_p1_affine { x: x_fp, y: y_fp };
    if !(unsafe { blst_p1_affine_on_curve(&aff) }) {
        return None;
    }
    let mut p = blst_p1::default();
    unsafe { blst_p1_from_affine(&mut p, &aff) };
    Some(Value::Point1(p))
}

fn parse_point_compressed(hex_str: &str) -> Option<Value> {
    let bytes = hex::decode(hex_str.strip_prefix("0x").unwrap_or(hex_str)).ok()?;
    if bytes.len() != 48 {
        return None;
    }
    let mut aff = blst_p1_affine::default();
    let err = unsafe { blst_p1_uncompress(&mut aff, bytes.as_ptr()) };
    if err != BLST_ERROR::BLST_SUCCESS {
        return None;
    }
    let mut p = blst_p1::default();
    unsafe { blst_p1_from_affine(&mut p, &aff); }
    Some(Value::Point1(p))
}

fn eval_mul(lhs: Value, rhs: Value) -> Option<Value> {
    match (lhs, rhs) {
        (Value::Point1(p), Value::Scalar(s)) | (Value::Scalar(s), Value::Point1(p)) => {
            let mut result = blst_p1::default();
            unsafe { blst_p1_mult(&mut result, &p, s.b.as_ptr(), 256); }
            Some(Value::Point1(result))
        }
        _ => {
            println!("Unsupported operand types for *");
            None
        }
    }
}

fn main() {
    let mut rl = Editor::<(), FileHistory>::new().unwrap();
    let _ = rl.load_history("blsh-history.txt");
    let mut env: HashMap<String, Value> = HashMap::new();

    loop {
        let readline = rl.readline("blsh> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "exit" || line == "quit" {
                    break;
                }
                if let Some((name, value)) = parse_let(line) {
                    env.insert(name.clone(), value.clone());
                    println!("{} bound", name);
                } else if let Some(val) = env.get(line) {
                    println!("{}", val);
                } else if let Some(result) = parse_expr(line) {
                    println!("{}", result);
                } else {
                    println!("Unrecognized input or unknown identifier");
                }
                rl.add_history_entry(line).unwrap_or(true);
            }
            Err(_) => break,
        }
    }
    let _ = rl.save_history("blsh-history.txt");
}