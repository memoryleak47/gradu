use crate::*;

use crate::ir::{Terminator, BlkDef, FnDef, ValueId};

fn lower_expr(e: &ast::Expr, out: &mut Vec<ir::Stmt>) -> ValueId {
    match e {
        ast::Expr::StringLit(x) => {
            let fresh = 240; // TODO freshness system.
            out.push(ir::Stmt::Compute(fresh, ir::Expr::StringLit(x.clone())));
            fresh
        },
        _ => todo!(),
    }
}

fn lower_stmt(stmt: &ast::Stmt, out: &mut Vec<ir::Stmt>) {
    match stmt {
        ast::Stmt::Print(x) => {
            let v = lower_expr(x, out);
            out.push(ir::Stmt::Print(v));
        },
        _ => todo!(),
    }
}

pub fn lower(ast: AST) -> IR {
    let def = {
        let mut blocks = HashMap::new();
        let mut stmts = Vec::new();

        for st in &ast {
            lower_stmt(st, &mut stmts);
        }

        let bdef = BlkDef {
            args: Vec::new(),
            stmts,
            terminator: Terminator::Exit,
            types: HashMap::new(),
        };
        blocks.insert(0, bdef);
        FnDef {
            blocks,
            start: 0,
        }
    };
    let mut fns = HashMap::new();
    fns.insert(0, def);
    let ir = IR {
        fns,
        global_types: HashMap::new(),
        start: 0,
    };
    ir
} 
