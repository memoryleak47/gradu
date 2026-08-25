use crate::*;

use crate::ir::{Terminator, BlkDef, FnDef, ValueId, Ty, BlkId};

fn lower_binop(kind: ast::BinOpKind, l: ValueId, r: ValueId, out: &mut BlkDef) -> ir::Expr {
    match kind {
        ast::BinOpKind::Plus => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), out);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), out);
            let o = mk_expr(ir::Expr::BinOp(ir::BinOpKind::Plus, l, r), out);
            ir::Expr::TToValue(o, Ty::Int)
        },
        ast::BinOpKind::Gt => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), out);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), out);
            let o = mk_expr(ir::Expr::BinOp(ir::BinOpKind::Gt, l, r), out);
            ir::Expr::TToValue(o, Ty::Bool)
        },
        x => todo!("{x:?}"),
    }
}

fn lower_expr(e: &ast::Expr, out: &mut BlkDef, varmap: &HashMap<Symbol, ValueId>) -> ValueId {
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
    use ir::BinOpKind::*;
    match e {
        ir::Expr::StringLit(_) => Ty::String,
        ir::Expr::IntLit(_) => Ty::Int,
        ir::Expr::BoolLit(_) => Ty::Bool,
        ir::Expr::TToValue(_, _) => Ty::Value,
        ir::Expr::ValueToT(_, o) => *o,
        ir::Expr::BinOp(Plus|Minus, _, _) => Ty::Int,
        ir::Expr::BinOp(Lt|Gt|Ne, _, _) => Ty::Bool,
        x => todo!("{x:?}")
    }
}

fn mk_expr(e: ir::Expr, out: &mut BlkDef) -> ValueId {
    let fresh = fresh();
    out.types.insert(fresh, ty_of(&e));
    out.stmts.push(ir::Stmt::Compute(fresh, e));
    fresh
}

fn lower_blk(stmts: &[ast::Stmt], out: &mut FnDef, post: BlkId, varmap: &mut HashMap<Symbol, ValueId>) -> BlkId {
    let new = fresh();
    let mut new_mut = new;

    out.blocks.insert(new, BlkDef {
        args: Vec::new(),
        stmts: Vec::new(),
        terminator: Terminator::Exit,
        types: HashMap::new(),
    });

    for st in stmts {
        lower_stmt(st, out, &mut new_mut, varmap);
    }
    out.blocks.get_mut(&new_mut).unwrap().terminator = Terminator::Goto((post, Box::new([])));

    new
}

fn lower_stmt(stmt: &ast::Stmt, out: &mut FnDef, bid: &mut BlkId, varmap: &mut HashMap<Symbol, ValueId>) {
    match stmt {
        ast::Stmt::Print(x) => {
            let v = lower_expr(x, out.blocks.get_mut(bid).unwrap(), varmap);
            out.blocks.get_mut(bid).unwrap().stmts.push(ir::Stmt::Print(v));
        },
        ast::Stmt::If(cond, then_, else_) => {
            let cond = lower_expr(cond, out.blocks.get_mut(bid).unwrap(), varmap);
            let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), out.blocks.get_mut(bid).unwrap());

            let post = fresh();
            out.blocks.insert(post, BlkDef {
                args: Vec::new(),
                stmts: Vec::new(),
                terminator: Terminator::Exit,
                types: HashMap::new(),
            });

            let then_ = lower_blk(then_, out, post, varmap);
            let else_ = lower_blk(else_, out, post, varmap);

            out.blocks.get_mut(bid).unwrap().terminator = Terminator::IfGoto(cond, (then_, Box::new([])), (else_, Box::new([])));

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
    let mut blocks = HashMap::new();

    let mut bdef = BlkDef {
        args: Vec::new(),
        stmts: Vec::new(),
        terminator: Terminator::Exit,
        types: HashMap::new(),
    };
    let bname = fresh();
    blocks.insert(bname, bdef);

    let mut fdef = FnDef {
        blocks,
        start: bname,
    };

    let mut current = bname;
    let mut varmap = HashMap::new();
    for st in ast {
        lower_stmt(st, &mut fdef, &mut current, &mut varmap);
    }
    let mut fns = HashMap::new();
    let fname = fresh();
    fns.insert(fname, fdef);
    let ir = IR {
        fns,
        global_types: HashMap::new(),
        start: fname,
    };
    ir
}
