use crate::*;

pub type FnId = usize; // function id
pub type BlkId = usize; // block id

pub type ValueId = usize; // local SSA-value id
pub type GlobalId = usize; // global variable id

pub struct IR {
    fns: HashMap<FnId, FnDef>,
    start: FnId,

    global_types: HashMap<GlobalId, Ty>,
}

struct FnDef {
    blocks: HashMap<BlkId, BlkDef>,
    start: BlkId,
    // fn arguments are inherited from the start block.
}

struct BlkDef {
    // the only ValueIds usable in a block are its `args` and those defined via `Compute` in its stmts.
    stmts: Vec<Stmt>,
    terminator: Terminator,

    args: Vec<ValueId>,
    types: HashMap<ValueId, Ty>,
}

enum Stmt {
    Compute(ValueId, Expr),

    WriteGlobal(GlobalId, ValueId),

    Push(/*list*/ValueId, /*value*/ValueId),
    ListStore(/*list*/ValueId, /*int*/ValueId, /*v*/ValueId), // list[int] = v
    DictStore(/*dict*/ValueId, /*k*/ValueId, /*v*/ValueId), // dict[k] = v
    Print(ValueId),
}

enum Expr {
    FnCall(ValueId, ValueId), // should this stop the block, like a terminator? no?

    Fn(FnId),
    NewList,
    NewDict,

    IndexList(/*list*/ValueId, /*index*/ValueId),
    IndexDict(/*dict*/ValueId, /*key*/ValueId),
    BinOp(BinOpKind, ValueId, ValueId),
    Length(ValueId),
    Var(Symbol),
    Input,

    IntLit(i64),
    StringLit(String),
    BoolLit(bool),
    NilLit,
}

enum Terminator {
    Return(/*retval*/ ValueId),
    Goto(BlkId),
    IfGoto(/*cond*/ ValueId, /*then*/ BlkId, /*else*/BlkId),
}

enum Ty {
    Value,
    Int,
    String,
    Fn,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BinOpKind {
    Lt,
    Gt,
    Mod,
    Plus,
    Mul,
    Minus,
    Equ,
    Ne, // !=
}
