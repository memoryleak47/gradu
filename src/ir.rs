use crate::*;

pub type FnId = usize; // function id
pub type BlkId = usize; // block id

pub type ValueId = usize; // local SSA-value id
pub type GlobalId = usize; // global variable id

pub type AppliedBlk = (BlkId, Box<[ValueId]>);

#[derive(Debug, Clone, Default)]
pub struct IR {
    pub fns: HashMap<FnId, FnDef>,
    pub start: FnId,

    pub global_types: HashMap<GlobalId, Ty>,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub blocks: HashMap<BlkId, BlkDef>,
    pub start: BlkId,
    pub retty: Ty,
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
    Nil,
}

impl IR {
    pub fn dump(&self) {
        for (f, fdef) in &self.fns {
            println!("fn fn_{f}:");
            for (b, bdef) in &fdef.blocks {
                let startstr = if *b == fdef.start { "start " } else { "" };
                print!("  {startstr}bb_{b}(");
                for (i, a) in bdef.args.iter().enumerate() {
                    print!("v_{a}");
                    if i != bdef.args.len()-1 {
                        print!(", ");
                    }
                }
                println!("):");
                for st in &bdef.stmts {
                    match st {
                        Stmt::Compute(n, e) => println!("    v_{n} = {e:?}"),
                        x => println!("    {x:?}"),
                    }
                }
                let dump_goto = |xy: &AppliedBlk| {
                    let (x, y) = xy;
                    print!("goto bb_{x}(");
                    for (i, a) in y.iter().enumerate() {
                        print!("v_{a}");
                        if i != y.len()-1 {
                            print!(", ");
                        }
                    }
                    print!(")");
                };
                match &bdef.terminator {
                    Terminator::Goto(t) => {
                        print!("    ");
                        dump_goto(t);
                        println!("");
                    },
                    Terminator::IfGoto(cond, x, y) => {
                        print!("    if v_{cond}: ");
                        dump_goto(x);
                        print!(" else ");
                        dump_goto(y);
                        println!("");
                    },
                    Terminator::Exit => println!("    exit"),
                    Terminator::Return(v) => println!("    return v_{v}"),
                }
            }
        }
    }
}
