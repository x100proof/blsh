use std::fmt;
use pest::Span;

#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Expr),
    Command(Command),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Command {
    Info(Expr),
}

#[derive(Clone)]
pub struct ExprMeta {
    pub span: (usize, usize), // start and end positions
}

impl ExprMeta {

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
}

impl fmt::Debug for ExprMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = self.span;
        write!(f, "ExprMeta ({},{})", start, end)?;
        Ok(())
    }
}

/*
 *   The extra `ExprMeta` field might be a bit ugly but if we try to create e.g. a `FullExpr`
 *   data structure that contains `Expr` and `ExprMeta` has fields we get "recursive type hell"
 *   since each of the `Box<Expr>` below would have to be `Box<FullExpr>` even though the
 *   type itself is `Expr`.
 */
#[derive(Clone)]
pub enum Expr {
    // SkScalar([u8; 32]), // distinguished from an ordinary scalar which can be unbounded
    Scalar(Box<[u8]>, ExprMeta), // bytes are in little-endian order
    Coord1([u8; 48], ExprMeta),
    CoordPair(Box<Expr>, Box<Expr>, ExprMeta),
    CoordTriple(Box<Expr>, Box<Expr>, Box<Expr>, ExprMeta),
    Point(Box<Expr>, Box<Expr>, ExprMeta),
    Point1Compressed([u8; 48], ExprMeta),
    Point1Uncompressed([u8;96], ExprMeta),
    Point2Compressed([u8; 96], ExprMeta),
    Point2Uncompressed([u8;192], ExprMeta),
    Embed(Box<Expr>, Box<Expr>, ExprMeta),
    Sqrt(Box<Expr>, ExprMeta),
    MapToG1(Box<Expr>,Box<Expr>, ExprMeta),
    MapToG2(Box<Expr>,Box<Expr>, ExprMeta),
    Mul(Box<Expr>, Box<Expr>, ExprMeta), // Point[1/2] * Scalar or Scalar * Point[1/2]
    Add(Box<Expr>, Box<Expr>, ExprMeta), // Point[1/2] + Point[1/2]
    Sub(Box<Expr>, Box<Expr>, ExprMeta), // Point[1/2] - Point[1/2]
    Neg(Box<Expr>, ExprMeta),            // -Point[1/2]
    Var(String, ExprMeta),
}

impl Expr {
    pub fn meta(&self) -> &ExprMeta {
        match self {
            Expr::Scalar(_, meta) => meta,
            Expr::Coord1(_, meta) => meta,
            Expr::CoordPair(_, _, meta) => meta,
            Expr::CoordTriple(_, _, _, meta) => meta,
            Expr::Point(_, _, meta) => meta,
            Expr::Point1Compressed(_, meta) => meta,
            Expr::Point1Uncompressed(_, meta) => meta,
            Expr::Point2Compressed(_, meta) => meta,
            Expr::Point2Uncompressed(_, meta) => meta,
            Expr::Embed(_, _, meta) => meta,
            Expr::Sqrt(_, meta) => meta,
            Expr::MapToG1(_, _, meta) => meta,
            Expr::MapToG2(_, _, meta) => meta,
            Expr::Mul(_, _, meta) => meta,
            Expr::Add(_, _, meta) => meta,
            Expr::Sub(_, _, meta) => meta,
            Expr::Neg(_, meta) => meta,
            Expr::Var(_, meta) => meta,
        }
    }

    pub fn miette_span(&self) -> (usize, usize) {
        let span = self.meta().span;
        (span.0, span.1 - span.0)
    }


}


impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Scalar(bytes, meta) => f.debug_tuple("Scalar").field(bytes).field(meta).finish(),
            Expr::Coord1(bytes, meta) => f.debug_tuple("Coord1").field(bytes).field(meta).finish(),
            Expr::CoordPair(e1, e2, meta) => f.debug_tuple("CoordPair").field(e1).field(e2).field(meta).finish(),
            Expr::CoordTriple(e1, e2, e3, meta) => f.debug_tuple("CoordTriple").field(e1).field(e2).field(e3).field(meta).finish(),
            Expr::Point(e1, e2, meta) => f.debug_tuple("Point").field(e1).field(e2).field(meta).finish(),
            Expr::Point1Compressed(bytes, meta) => f.debug_tuple("Point1Compressed").field(bytes).field(meta).finish(),
            Expr::Point1Uncompressed(bytes, meta) => f.debug_tuple("Point1Uncompressed").field(bytes).field(meta).finish(),
            Expr::Point2Compressed(bytes, meta) => f.debug_tuple("Point2Compressed").field(bytes).field(meta).finish(),
            Expr::Point2Uncompressed(bytes, meta) => f.debug_tuple("Point2Uncompressed").field(bytes).field(meta).finish(),
            Expr::Embed(e1, e2, meta) => f.debug_tuple("Embed").field(e1).field(e2).field(meta).finish(),
            Expr::Sqrt(e, meta) => f.debug_tuple("Sqrt").field(e).field(meta).finish(),
            Expr::MapToG1(e1, e2, meta) => f.debug_tuple("MapToG1").field(e1).field(e2).field(meta).finish(),
            Expr::MapToG2(e1, e2, meta) => f.debug_tuple("MapToG2").field(e1).field(e2).field(meta).finish(),
            Expr::Mul(e1, e2, meta) => f.debug_tuple("Mul").field(e1).field(e2).field(meta).finish(),
            Expr::Add(e1, e2, meta) => f.debug_tuple("Add").field(e1).field(e2).field(meta).finish(),
            Expr::Sub(e1, e2, meta) => f.debug_tuple("Sub").field(e1).field(e2).field(meta).finish(),
            Expr::Neg(e, meta) => f.debug_tuple("Neg").field(e).field(meta).finish(),
            Expr::Var(name, meta) => f.debug_tuple("Var").field(name).field(meta).finish(),
        }
    }
}
