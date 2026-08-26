use crate::*;

use std::collections::{BTreeSet, BTreeMap};
use crate::ir::{Terminator, BlkDef, FnDef, ValueId, Ty, BlkId, FnId, AppliedBlk};

fn fresh_blk(fdef: &mut FnDef, varmap: &BTreeMap<Symbol, ValueId>) -> BlkId {
    let new = fresh();
    fdef.blocks.insert(new, BlkDef {
        args: (0..varmap.len()).map(|_| fresh()).collect(),
        stmts: Vec::new(),
        terminator: Terminator::Exit,
        types: HashMap::new(),
    });
    new
}

fn mk_applied_blk(target: BlkId, varmap: &BTreeMap<Symbol, ValueId>) -> AppliedBlk {
    (target, varmap.values().copied().collect())
}

fn lower_binop(kind: BinOpKind, l: ValueId, r: ValueId, out: &mut BlkDef) -> ir::Expr {
    use BinOpKind::*;

    match kind {
        Plus|Minus|Mod => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), out);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), out);
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), out);
            ir::Expr::TToValue(o, Ty::Int)
        },
        Gt|Lt => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), out);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), out);
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), out);
            ir::Expr::TToValue(o, Ty::Bool)
        },
        Equ|Ne => {
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), out);
            ir::Expr::TToValue(o, Ty::Bool)
        },
        x => todo!("{x:?}"),
    }
}

fn lower_expr(e: &ast::Expr, out: &mut BlkDef, varmap: &BTreeMap<Symbol, ValueId>) -> ValueId {
    let e = match e {
        ast::Expr::StringLit(x) => {
            let a = mk_expr(ir::Expr::StringLit(x.clone()), out);
            ir::Expr::TToValue(a, Ty::String)
        },
        ast::Expr::IntLit(x) => {
            let a = mk_expr(ir::Expr::IntLit(*x), out);
            ir::Expr::TToValue(a, Ty::Int)
        },
        ast::Expr::BoolLit(x) => {
            let a = mk_expr(ir::Expr::BoolLit(*x), out);
            ir::Expr::TToValue(a, Ty::Bool)
        },
        ast::Expr::BinOp(kind, l, r) => {
            let l = lower_expr(l, out, varmap);
            let r = lower_expr(r, out, varmap);
            lower_binop(*kind, l, r, out)
        },
        ast::Expr::Var(v) => return varmap[v],
        x => todo!("{x:?}"),
    };

    mk_expr(e, out)
}

fn ty_of(e: &ir::Expr) -> Ty {
    use BinOpKind::*;
    match e {
        ir::Expr::StringLit(_) => Ty::String,
        ir::Expr::IntLit(_) => Ty::Int,
        ir::Expr::BoolLit(_) => Ty::Bool,
        ir::Expr::TToValue(_, _) => Ty::Value,
        ir::Expr::ValueToT(_, o) => *o,
        ir::Expr::BinOp(Plus|Minus|Mod, _, _) => Ty::Int,
        ir::Expr::BinOp(Lt|Gt|Equ|Ne, _, _) => Ty::Bool,
        x => todo!("{x:?}")
    }
}

fn mk_expr(e: ir::Expr, out: &mut BlkDef) -> ValueId {
    let fresh = fresh();
    out.types.insert(fresh, ty_of(&e));
    out.stmts.push(ir::Stmt::Compute(fresh, e));
    fresh
}

fn lower_blk(stmts: &[ast::Stmt], out: &mut FnDef, post: BlkId, varmap: &mut BTreeMap<Symbol, ValueId>) -> BlkId {
    let new = fresh_blk(out, varmap);
    let mut new_mut = new;

    for st in stmts {
        lower_stmt(st, out, &mut new_mut, varmap);
    }
    out.blocks.get_mut(&new_mut).unwrap().terminator = Terminator::Goto(mk_applied_blk(post, varmap));

    new
}

fn lower_stmt(stmt: &ast::Stmt, out: &mut FnDef, bid: &mut BlkId, varmap: &mut BTreeMap<Symbol, ValueId>) {
    match stmt {
        ast::Stmt::Print(x) => {
            let v = lower_expr(x, out.blocks.get_mut(bid).unwrap(), varmap);
            out.blocks.get_mut(bid).unwrap().stmts.push(ir::Stmt::Print(v));
        },
        ast::Stmt::If(cond, then_, else_) => {
            let cond = lower_expr(cond, out.blocks.get_mut(bid).unwrap(), varmap);
            let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), out.blocks.get_mut(bid).unwrap());

            let post = fresh_blk(out, varmap);

            let then_ = lower_blk(then_, out, post, varmap);
            let else_ = lower_blk(else_, out, post, varmap);

            out.blocks.get_mut(bid).unwrap().terminator = Terminator::IfGoto(cond, mk_applied_blk(then_, varmap), mk_applied_blk(else_, varmap));

            *bid = post;
        },

        ast::Stmt::While(cond, body) => {
            let head = fresh_blk(out, varmap);
            let post = fresh_blk(out, varmap);
            out.blocks.get_mut(bid).unwrap().terminator = Terminator::Goto(mk_applied_blk(head, varmap));

            let cond = lower_expr(cond, out.blocks.get_mut(&head).unwrap(), varmap);
            let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), out.blocks.get_mut(&head).unwrap());

            let body = lower_blk(body, out, head, varmap);

            out.blocks.get_mut(&head).unwrap().terminator = Terminator::IfGoto(cond, mk_applied_blk(body, varmap), mk_applied_blk(post, varmap));

            *bid = post;
        },
        ast::Stmt::Assign(var, val) => {
            let val = lower_expr(val, out.blocks.get_mut(bid).unwrap(), varmap);
            varmap.insert(*var, val);
        },
        x => todo!("{x:?}"),
    }
}

pub fn lower(ast: &AST) -> IR {
    let mut ir = IR {
        fns: HashMap::new(),
        global_types: HashMap::new(),
        start: 0,
    };
    let start = lower_fn(&[], ast, &mut ir);
    ir.start = start;
    ir
}

fn lower_fn(args: &[Symbol], body: &[ast::Stmt], ir: &mut IR) -> FnId {
    let mut vars = BTreeSet::new();
    get_vars(body, &mut vars);
    vars.extend(args);

    let mut blocks = HashMap::new();

    let b1 = fresh();
    let b2 = fresh();
    let nil1 = fresh();
    let nil2 = fresh();

    let args1: Vec<_> = (0..args.len()).map(|_| fresh()).collect();
    let args2: Vec<_> = (0..vars.len()).map(|_| fresh()).collect();

    let varmap: BTreeMap<Symbol, ValueId> = vars.iter().map(|x| {
        let o = if let Some(i) = args.iter().position(|y| y == x) {
            args2[i]
        } else {
            nil2
        };

        (*x, o)
    }).collect();

    let mut types = HashMap::new();
    types.insert(nil1, Ty::Nil);
    types.insert(nil2, Ty::Value);
    let mut bdef1 = BlkDef {
        args: args1,
        stmts: vec![
            ir::Stmt::Compute(nil1, ir::Expr::NilLit),
            ir::Stmt::Compute(nil2, ir::Expr::TToValue(nil1, Ty::Nil)),
        ],
        terminator: Terminator::Goto(mk_applied_blk(b2, &varmap)),
        types,
    };
    blocks.insert(b1, bdef1);

    let mut bdef2 = BlkDef {
        args: args2.clone(),
        stmts: Vec::new(),
        terminator: Terminator::Exit,
        types: HashMap::new(),
    };
    blocks.insert(b2, bdef2);

    let mut fdef = FnDef {
        blocks,
        start: b1,
    };

    let mut varmap: BTreeMap<Symbol, ValueId> = vars.iter().copied().zip(args2.iter().copied()).collect();

    let mut current = b2;
    for st in body {
        lower_stmt(st, &mut fdef, &mut current, &mut varmap);
    }
    let fname = fresh();
    ir.fns.insert(fname, fdef);
    fname
}

fn get_vars(body: &[ast::Stmt], out: &mut BTreeSet<Symbol>) {
    for a in body {
        use ast::Stmt::*;
        match a {
            Global(s) => { out.insert(*s); },
            Return(e)|Print(e) => get_vars_expr(e, out),
            Assign(v, e) => { out.insert(*v); get_vars_expr(e, out); },
            Push(e1, e2) => { get_vars_expr(e1, out); get_vars_expr(e2, out); },
            ListStore(e1, e2, e3) | DictStore(e1, e2, e3) => { get_vars_expr(e1, out); get_vars_expr(e2, out); get_vars_expr(e3, out); },
            If(e, b1, b2) => { get_vars_expr(e, out); get_vars(b1, out); get_vars(b2, out); },
            While(e, b) => { get_vars_expr(e, out); get_vars(b, out); },
        }
    }
}

fn get_vars_expr(e: &ast::Expr, out: &mut BTreeSet<Symbol>) {
    use ast::Expr::*;
    match e {
        Var(v) => { out.insert(*v); },
        IndexList(e1, e2) | IndexDict(e1, e2) | BinOp(_, e1, e2) => {
            get_vars_expr(e1, out); get_vars_expr(e2, out);
        },
        Length(e) => {
            get_vars_expr(e, out);
        },
        FnCall(e, args) => {
            get_vars_expr(e, out);
            for a in args {
                get_vars_expr(a, out);
            }
        },
        Input|NewList|NewDict|Fn(..)|IntLit(_)|StringLit(_)|BoolLit(_)|NilLit => {},
    }
}
