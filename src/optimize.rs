use crate::*;

// return true if something happened.
pub fn optimize(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    if inline_const_global_read(ast, nameres, actxt) { return true }
    if inline_const_local_read(ast, nameres, actxt) { return true }
    false
}

fn inline_const_global_read(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    for &g in &nameres.globals {
        let lat = get(Location::GlobalVar(g), actxt);
        let Some(e) = as_concrete_expr(lat) else { continue };
        for (fid, f) in ast.fns.iter_mut().enumerate() {
            if nameres.vars[fid].get(&g) != Some(&VarKind::Global) { continue }

            visit_body_mut(&mut f.body, &mut |expr|{
                if expr == &Expr::Var(g) {
                    *expr = e.clone();
                }
            }, &mut |_|{});
        }
    }
    false
}

fn inline_const_local_read(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    for (fid, f) in ast.fns.iter_mut().enumerate() {
        for (&v, &kind) in &nameres.vars[fid] {
            let VarKind::Local = kind else { continue };
            let lat = get(Location::Var(fid, v), actxt);
            let Some(e) = as_concrete_expr(lat) else { continue };

            visit_body_mut(&mut f.body, &mut |expr|{
                if expr == &Expr::Var(v) {
                    *expr = e.clone();
                }
            }, &mut |_|{});
        }
    }
    false
}


fn as_concrete_expr(mut lat: TypeLattice) -> Option<Expr> {
    let fns = std::mem::take(&mut lat.fn_options);
    if lat != TypeLattice::bot() { return None }
    if fns.len() != 1 { return None }
    let f = *fns.iter().next().unwrap();
    Some(Expr::FnId(f))
}
