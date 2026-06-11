use crate::*;

pub fn get_vars(f: &FnDef) -> HashSet<Symbol> {
    let mut set1 = HashSet::new();
    let mut set2 = HashSet::new();
    set1.extend(&f.args);

    let f_expr = &mut |expr: &Expr| {
        if let Expr::Var(v) = expr {
            set1.insert(*v);
        }
    };
    let f_stmt = &mut |stmt: &Stmt| {
        if let Stmt::Assign(v, _) = stmt {
            set2.insert(*v);
        }
    };

    visit_body(&f.body, f_expr, f_stmt);

    set1.union(&set2).copied().collect()
}

