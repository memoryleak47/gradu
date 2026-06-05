pub type Body = Vec<Stmt>;

pub struct AST {
    pub fns: Vec<FnDef>,
}

pub struct FnDef {
    pub name: String,
    pub args: Vec<String>,
    pub body: Body,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    Assign(String, Expr),
    If(Expr, /*then*/ Body, /*else*/ Body),
    While(Expr, Body),
    Print(Expr),
}

#[derive(Debug)]
pub enum Expr {
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    Var(String),
    Input,
    FnCall(String, Vec<Expr>),
}

#[derive(Debug)]
pub enum BinOpKind {
    Lt,
    Gt,
    Mod,
    Plus,
    Mul,
    Minus,
    Equ,
}
