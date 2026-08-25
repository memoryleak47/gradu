use crate::*;

pub type FnId = usize;

pub type AST = Box<[Stmt]>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Stmt {
    Global(Symbol),
    Return(Expr),
    Assign(Symbol, Expr),
    Push(/*list*/Expr, /*value*/Expr),
    ListStore(/*list*/Expr, /*int*/Expr, /*v*/Expr), // list[int] = v
    DictStore(/*dict*/Expr, /*k*/Expr, /*v*/Expr), // dict[k] = v
    If(Expr, /*then*/ AST, /*else*/ AST),
    While(Expr, AST),
    Print(Expr),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Fn(/*args*/ Vec<Symbol>, /*body*/ AST),
    NewList,
    NewDict,
    IndexList(/*list*/Box<Expr>, /*index*/Box<Expr>),
    IndexDict(/*dict*/Box<Expr>, /*key*/Box<Expr>),
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    Length(Box<Expr>),
    Var(Symbol),
    Input,
    FnCall(Box<Expr>, Vec<Expr>),

    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    NilLit,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

// only for parsing!
#[derive(Debug, PartialEq, Eq)]
pub enum LValue {
    IndexList(Expr, Expr),
    IndexDict(Expr, Expr),
    Var(Symbol),
}
