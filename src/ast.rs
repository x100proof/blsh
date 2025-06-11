use std::fmt;
use pest::Span;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Expr, SpanInfo),
    Command(Command),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Command {
    Info(Expr),
}

#[derive(Clone)]
pub struct SpanInfo {
    pub span: (usize, usize), // start and end positions
}

impl SpanInfo {

    pub fn from_span(span: Span) -> Self {
        Self {
            span: (span.start(), span.end()),
        }
    }

    pub fn merge(&self, other: &Self) -> Self {
        Self {
           span: (self.span.0, other.span.1)
        }
    }

    pub fn miette_span(&self) -> (usize, usize) {
        let s = self.span;
        (s.0, s.1 - s.0)
    }

}

impl fmt::Debug for SpanInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = self.span;
        write!(f, "SpanInfo ({},{})", start, end)?;
        Ok(())
    }
}

/*
 *   The extra `SpanInfo` field might be a bit ugly but if we try to create e.g. a `FullExpr`
 *   data structure that contains `Expr` and `SpanInfo` has fields we get "recursive type hell"
 *   since each of the `Box<Expr>` below would have to be `Box<FullExpr>` even though the
 *   type itself is `Expr`.
 */
#[derive(Clone)]
pub enum Expr {
    // SkScalar([u8; 32]), // distinguished from an ordinary scalar which can be unbounded
    LiteralBool(bool, SpanInfo),
    Scalar(Box<[u8]>, SpanInfo), // bytes are in little-endian order
    Coord1([u8; 48], SpanInfo),
    CoordPair(Box<Expr>, Box<Expr>, SpanInfo),
    CoordTriple(Box<Expr>, Box<Expr>, Box<Expr>, SpanInfo),
    Point(Box<Expr>, Box<Expr>, SpanInfo),
    Point1Compressed([u8; 48], SpanInfo),
    Point1Uncompressed([u8;96], SpanInfo),
    Point2Compressed([u8; 96], SpanInfo),
    Point2Uncompressed([u8;192], SpanInfo),
    Embed(Box<Expr>, Box<Expr>, SpanInfo),
    Inv(Box<Expr>, SpanInfo), // Coord12
    Sqr(Box<Expr>, SpanInfo), // Coord12
    Sqrt(Box<Expr>, SpanInfo),
    MillerLoop(Box<Expr>, SpanInfo),
    MapToG1(Box<Expr>,Box<Expr>, SpanInfo),
    MapToG2(Box<Expr>,Box<Expr>, SpanInfo),
    Equal(Box<Expr>, Box<Expr>, SpanInfo),
    Mul(Box<Expr>, Box<Expr>, SpanInfo), // Point[1/2] * Scalar or Scalar * Point[1/2]
    Add(Box<Expr>, Box<Expr>, SpanInfo), // Point[1/2] + Point[1/2]
    Sub(Box<Expr>, Box<Expr>, SpanInfo), // Point[1/2] - Point[1/2]
    Neg(Box<Expr>, SpanInfo),            // -Point[1/2]
    Var(String, SpanInfo),
    List(Vec<Box<Expr>>, SpanInfo),
}

impl Expr {
    pub fn span_info(&self) -> &SpanInfo {
        match self {
            Expr::LiteralBool(_, s) => s,
            Expr::Scalar(_, s) => s,
            Expr::Coord1(_, s) => s,
            Expr::CoordPair(_, _, s) => s,
            Expr::CoordTriple(_, _, _, s) => s,
            Expr::Point(_, _, s) => s,
            Expr::Point1Compressed(_, s) => s,
            Expr::Point1Uncompressed(_, s) => s,
            Expr::Point2Compressed(_, s) => s,
            Expr::Point2Uncompressed(_, s) => s,
            Expr::Embed(_, _, s) => s,
            Expr::Inv(_, s) => s,
            Expr::Sqr(_, s) => s,
            Expr::Sqrt(_, s) => s,
            Expr::MillerLoop(_, s) => s,
            Expr::MapToG1(_, _, s) => s,
            Expr::MapToG2(_, _, s) => s,
            Expr::Equal(_, _, s) => s,
            Expr::Mul(_, _, s) => s,
            Expr::Add(_, _, s) => s,
            Expr::Sub(_, _, s) => s,
            Expr::Neg(_, s) => s,
            Expr::Var(_, s) => s,
            Expr::List(_, s) => s,
        }
    }

    pub fn miette_span(&self) -> (usize, usize) {
        self.span_info().miette_span()
    }


}


impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::LiteralBool(b, span_info) => f.debug_tuple("LiteralBool").field(b).field(span_info).finish(),
            Expr::Scalar(bytes, span_info) => f.debug_tuple("Scalar").field(bytes).field(span_info).finish(),
            Expr::Coord1(bytes, span_info) => f.debug_tuple("Coord1").field(bytes).field(span_info).finish(),
            Expr::CoordPair(e1, e2, span_info) => f.debug_tuple("CoordPair").field(e1).field(e2).field(span_info).finish(),
            Expr::CoordTriple(e1, e2, e3, span_info) => f.debug_tuple("CoordTriple").field(e1).field(e2).field(e3).field(span_info).finish(),
            Expr::Point(e1, e2, span_info) => f.debug_tuple("Point").field(e1).field(e2).field(span_info).finish(),
            Expr::Point1Compressed(bytes, span_info) => f.debug_tuple("Point1Compressed").field(bytes).field(span_info).finish(),
            Expr::Point1Uncompressed(bytes, span_info) => f.debug_tuple("Point1Uncompressed").field(bytes).field(span_info).finish(),
            Expr::Point2Compressed(bytes, span_info) => f.debug_tuple("Point2Compressed").field(bytes).field(span_info).finish(),
            Expr::Point2Uncompressed(bytes, span_info) => f.debug_tuple("Point2Uncompressed").field(bytes).field(span_info).finish(),
            Expr::Embed(e1, e2, span_info) => f.debug_tuple("Embed").field(e1).field(e2).field(span_info).finish(),
            Expr::Inv(e, span_info) => f.debug_tuple("Inv").field(e).field(span_info).finish(),
            Expr::Sqr(e, span_info) => f.debug_tuple("Sqr").field(e).field(span_info).finish(),
            Expr::Sqrt(e, span_info) => f.debug_tuple("Sqrt").field(e).field(span_info).finish(),
            Expr::MillerLoop(e, span_info) => f.debug_tuple("MillerLoop").field(e).field(span_info).finish(),
            Expr::MapToG1(e1, e2, span_info) => f.debug_tuple("MapToG1").field(e1).field(e2).field(span_info).finish(),
            Expr::MapToG2(e1, e2, span_info) => f.debug_tuple("MapToG2").field(e1).field(e2).field(span_info).finish(),
            Expr::Equal(e1, e2, span_info) => f.debug_tuple("Equal").field(e1).field(e2).field(span_info).finish(),
            Expr::Mul(e1, e2, span_info) => f.debug_tuple("Mul").field(e1).field(e2).field(span_info).finish(),
            Expr::Add(e1, e2, span_info) => f.debug_tuple("Add").field(e1).field(e2).field(span_info).finish(),
            Expr::Sub(e1, e2, span_info) => f.debug_tuple("Sub").field(e1).field(e2).field(span_info).finish(),
            Expr::Neg(e, span_info) => f.debug_tuple("Neg").field(e).field(span_info).finish(),
            Expr::Var(name, span_info) => f.debug_tuple("Var").field(name).field(span_info).finish(),
            Expr::List(vs, span_info) => f.debug_tuple("Array").field(vs).field(span_info).finish(),
        }
    }
}
