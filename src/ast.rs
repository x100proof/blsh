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
    Point1(Box<Expr>, Box<Expr>),
    Point1Compressed([u8; 48]),
    Sqrt(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Var(String),
}
