use crate::*;

pub type FnId = usize;

#[derive(Debug)]
pub struct AST {
    pub fns: Vec<FnDef>, // indexed by FnId
    pub main_fn: FnId,
}

#[derive(Debug)]
pub struct FnDef {
    pub args: Vec<Symbol>,
    pub body: Body,
}

pub type Body = Vec<Stmt>;

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt {
    Global(Symbol),
    Return(Expr),
    Assign(Symbol, Expr),
    Push(/*list*/Expr, /*value*/Expr),
    ListStore(/*list*/Expr, /*int*/Expr, /*v*/Expr), // list[int] = v
    DictStore(/*dict*/Expr, /*k*/Expr, /*v*/Expr), // dict[k] = v
    If(Expr, /*then*/ Body, /*else*/ Body),
    While(Expr, Body),
    Print(Expr),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    FnId(FnId),
    NewList,
    NewDict,
    IndexList(/*list*/Box<Expr>, /*index*/Box<Expr>),
    IndexDict(/*dict*/Box<Expr>, /*key*/Box<Expr>),
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
