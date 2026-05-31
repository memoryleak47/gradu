pub type AST = Vec<Stmt>;

#[derive(Debug)]
pub enum Stmt {
    Assign(String, Expr),
    If(Expr, /*then*/ AST, /*else*/ AST),
    Print(Expr),
}

#[derive(Debug)]
pub enum Expr {
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    IntLit(i64),
    StringLit(String),
    Var(String),
}

#[derive(Debug)]
pub enum BinOpKind {
    Gt,
}

fn main() {
    println!("Hello, world!");
}
