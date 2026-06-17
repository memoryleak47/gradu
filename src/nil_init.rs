use crate::*;

pub fn nil_init(ast: &mut AST, nameres: &Nameres) {
    // initialize returns
    for f in ast.fns.iter_mut() {
        f.body.push(Stmt::Return(Expr::NilLit));
    }

    // initialize globals
    let main_body = &mut ast.fns[ast.main_fn].body;
    for &g in &nameres.globals {
        main_body.insert(0, Stmt::Assign(g, Expr::NilLit));
    }

    // initialize locals
    for (fid, f) in ast.fns.iter_mut().enumerate() {
        for (v, kind) in &nameres.vars[fid] {
            let VarKind::Local = kind else { continue };
            if f.args.contains(v) { continue }
            f.body.insert(0, Stmt::Assign(*v, Expr::NilLit));
        }
    }
}
