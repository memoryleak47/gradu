use crate::*;

pub type Body = Vec<Stmt>;

pub struct AST {
    pub fns: Vec<FnDef>,
}

pub struct FnDef {
    pub name: Symbol,
    pub args: Vec<Symbol>,
    pub body: Body,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    Assign(Symbol, Expr),
    Push(/*list*/Expr, /*value*/Expr),
    ListStore(/*list*/Expr, /*int*/Expr, /*v*/Expr), // list[int] = v
    If(Expr, /*then*/ Body, /*else*/ Body),
    While(Expr, Body),
    Print(Expr),
}

#[derive(Debug)]
pub enum Expr {
    NewList,
    IndexList(/*list*/Box<Expr>, /*index*/Box<Expr>),
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    Length(Box<Expr>),
    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    Var(Symbol),
    Input,
    FnCall(Symbol, Vec<Expr>),
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
