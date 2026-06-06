use crate::*;

pub fn get_vars(f: &FnDef) -> HashSet<Symbol> {
    let mut set = HashSet::new();
    set.extend(&f.args);

    get_vars_body(&f.body, &mut set);

    set
}

fn get_vars_body(body: &Body, set: &mut HashSet<Symbol>) {
    for s in body {
        get_vars_stmt(s, set);
    }
}

fn get_vars_stmt(stmt: &Stmt, set: &mut HashSet<Symbol>) {
    use Stmt::*;
    match stmt {
        Return(e) => get_vars_expr(e, set),
        Assign(v, e) => {
            set.insert(*v);
            get_vars_expr(e, set);
        },
        If(c, then_, else_) => {
            get_vars_expr(c, set);
            get_vars_body(then_, set);
            get_vars_body(else_, set);
        },
        While(e, b) => {
            get_vars_expr(e, set);
            get_vars_body(b, set);
        },
        Print(e) => {
            get_vars_expr(e, set);
        },
    }
}

fn get_vars_expr(expr: &Expr, set: &mut HashSet<Symbol>) {
    use Expr::*;
    match expr {
        BinOp(_, e1, e2) => {
            get_vars_expr(e1, set);
            get_vars_expr(e2, set);
        },
        IntLit(_) => {},
        StringLit(_) => {},
        BoolLit(_) => {},
        Var(v) => {
            set.insert(*v);
        },
        Input => {},
        FnCall(_, es) => {
            for e in es {
                get_vars_expr(e, set);
            }
        },
    }
}
