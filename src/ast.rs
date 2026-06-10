use crate::*;

pub type AST = Vec<Stmt>;

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Global(Symbol),
    Return(Expr),
    Assign(Symbol, Expr),
    Push(/*list*/Expr, /*value*/Expr),
    ListStore(/*list*/Expr, /*int*/Expr, /*v*/Expr), // list[int] = v
    If(Expr, /*then*/ AST, /*else*/ AST),
    While(Expr, AST),
    Print(Expr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    FnDef(/*args*/Vec<Symbol>, /*body*/AST),
    NewList,
    IndexList(/*list*/Box<Expr>, /*index*/Box<Expr>),
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    Length(Box<Expr>),
    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    Var(Symbol),
    Input,
    FnCall(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOpKind {
    Lt,
    Gt,
    Mod,
    Plus,
    Mul,
    Minus,
    Equ,
    Ne, // !=
}
