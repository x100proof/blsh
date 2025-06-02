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

#[derive(Debug, Clone)]
pub enum Expr {
    // SkScalar([u8; 32]), // distinguished from an ordinary scalar which can be unbounded
    Scalar(Box<[u8]>),
    Coord1([u8; 48]),
    CoordPair(Box<Expr>, Box<Expr>),
    CoordTriple(Box<Expr>, Box<Expr>, Box<Expr>),
    Point(Box<Expr>, Box<Expr>),
    Point1Compressed([u8; 48]),
    Point1Uncompressed([u8;96]),
    Point2Compressed([u8; 96]),
    Point2Uncompressed([u8;192]),
    Embed(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    MapToG1(Box<Expr>,Box<Expr>),
    MapToG2(Box<Expr>,Box<Expr>),
    Mul(Box<Expr>, Box<Expr>), // Point[1/2] * Scalar or Scalar * Point[1/2]
    Add(Box<Expr>, Box<Expr>), // Point[1/2] + Point[1/2]
    Sub(Box<Expr>, Box<Expr>), // Point[1/2] - Point[1/2]
    Neg(Box<Expr>),            // -Point[1/2]
    Var(String),
}
