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
    Scalar(String),
    Coord1(String),
    Point1(Box<Expr>, Box<Expr>),
    Point1Compressed(String),
    Sqrt(Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Var(String),
}
