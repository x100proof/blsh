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
    Scalar([u8; 32]),
    Coord1([u8; 48]),
    Coord2([u8; 48], [u8; 48]),
    Point1(Box<Expr>, Box<Expr>),
    Point2(Box<Expr>, Box<Expr>),
    Point1Compressed([u8; 48]),
    Point2Compressed([u8; 96]),
    Sqrt(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>), // Point[1/2] * Scalar or Scalar * Point[1/2]
    Add(Box<Expr>, Box<Expr>), // Point[1/2] + Point[1/2]
    Sub(Box<Expr>, Box<Expr>), // Point[1/2] - Point[1/2]
    Neg(Box<Expr>),            // -Point[1/2]
    Var(String),
}
