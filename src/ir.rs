use crate::*;

pub type FnId = usize; // function id
pub type BlkId = usize; // block id

pub type ValueId = usize; // local SSA-value id
pub type GlobalId = usize; // global variable id

pub type AppliedBlk = (BlkId, Box<[ValueId]>);

#[derive(Debug, Clone)]
pub struct IR {
    pub fns: HashMap<FnId, FnDef>,
    pub start: FnId,

    pub global_types: HashMap<GlobalId, Ty>,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub blocks: HashMap<BlkId, BlkDef>,
    pub start: BlkId,
    // fn arguments are inherited from the start block.
}

#[derive(Debug, Clone)]
pub struct BlkDef {
    // the only ValueIds usable in a block are its `args` and those defined via `Compute` in its stmts.
    pub stmts: Vec<Stmt>,
    pub terminator: Terminator,

    pub args: Vec<ValueId>,
    pub types: HashMap<ValueId, Ty>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Compute(ValueId, Expr),

    WriteGlobal(GlobalId, ValueId),

    Push(/*list*/ ValueId, /*value*/ ValueId),
    ListStore(/*list*/ ValueId, /*int*/ ValueId, /*v*/ ValueId), // list[int] = v
    DictStore(/*dict*/ ValueId, /*k*/ ValueId, /*v*/ ValueId), // dict[k] = v
    Print(ValueId),
}

#[derive(Debug, Clone)]
pub enum Expr {
    FnCall(ValueId, Box<[ValueId]>), // in minirust this was a terminator, but not in LLVM, so I'll leave it as an Expr.

    LoadGlobal(GlobalId),

    Fn(FnId),
    NewList,
    NewDict,

    IndexList(/*list*/ ValueId, /*index*/ ValueId),
    IndexDict(/*dict*/ ValueId, /*key*/ ValueId),
    BinOp(BinOpKind, ValueId, ValueId),
    Length(ValueId),
    Input,

    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    NilLit,

    TToValue(ValueId, /*in type T*/ Ty), // wraps something to a Value. input type can be obtained from said ValueId.
    ValueToT(ValueId, /*out type T*/ Ty),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Exit,
    Return(/*retval*/ ValueId),
    Goto(AppliedBlk),
    IfGoto(/*cond*/ ValueId, /*then*/ AppliedBlk, /*else*/ AppliedBlk),
}

#[derive(Debug, Clone, Copy)]
pub enum Ty {
    Value,
    Int,
    String,
    Fn,
    Bool,
}
