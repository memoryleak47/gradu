use crate::*;

// return true if something happened.
pub fn optimize(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    if inline_const_global_read(ast, nameres, actxt) { return true }
    false
}

fn inline_const_global_read(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    for &g in &nameres.globals {
        let lat = get(Location::GlobalVar(g), actxt);
        if let Some(e) = as_concrete_expr(lat) {
            for (fid, f) in ast.fns.iter_mut().enumerate() {
                if nameres.vars[fid].get(&g) != Some(&VarKind::Global) { continue }

                visit_body_mut(&mut f.body, &mut |expr|{
                    if expr == &Expr::Var(g) {
                        *expr = e.clone();
                    }
                }, &mut |_|{});
            }
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
