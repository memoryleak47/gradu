use crate::*;

use crate::ir::{Terminator, BlkDef, FnDef, ValueId, Ty};

fn lower_expr(e: &ast::Expr, out: &mut BlkDef) -> ValueId {
    let fresh = fresh();

    let e = match e {
        ast::Expr::StringLit(x) => ir::Expr::StringLit(x.clone()),
        ast::Expr::IntLit(x) => ir::Expr::IntLit(*x),
        ast::Expr::NilLit => ir::Expr::NilLit,
        ast::Expr::BoolLit(x) => ir::Expr::BoolLit(*x),
        _ => todo!(),
    };

    out.stmts.push(ir::Stmt::Compute(fresh, e));
    out.types.insert(fresh, Ty::Value); // everything is value-typed on start.
    fresh
}

fn lower_stmt(stmt: &ast::Stmt, out: &mut BlkDef) {
    match stmt {
        ast::Stmt::Print(x) => {
            let v = lower_expr(x, out);
            out.stmts.push(ir::Stmt::Print(v));
        },
        _ => todo!(),
    }
}

pub fn lower(ast: &AST) -> IR {
    let def = {
        let mut blocks = HashMap::new();

        let mut bdef = BlkDef {
            args: Vec::new(),
            stmts: Vec::new(),
            terminator: Terminator::Exit,
            types: HashMap::new(),
        };

        for st in ast {
            lower_stmt(st, &mut bdef);
        }

        let bname = fresh();
        blocks.insert(bname, bdef);
        FnDef {
            blocks,
            start: bname,
        }
    };
    let mut fns = HashMap::new();
    let fname = fresh();
    fns.insert(fname, def);
    let ir = IR {
        fns,
        global_types: HashMap::new(),
        start: fname,
    };
    ir
} 
