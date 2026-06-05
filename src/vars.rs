use crate::*;

pub fn get_vars(ast: &AST) -> HashSet<String> {
    let mut vars = HashSet::new();
    for f in &ast.fns {
        get_vars_body(&f.body, &mut vars);
    }
    vars
}

fn get_vars_body(body: &Body, vars: &mut HashSet<String>) {
    for st in body {
        match st {
            Stmt::Assign(v, e) => {
                vars.insert(v.to_string());
                get_vars_expr(e, vars);
            },
            Stmt::Return(e) => {
                get_vars_expr(e, vars);
            },
            Stmt::If(cond, then_, else_) => {
                get_vars_expr(cond, vars);
                get_vars_body(then_, vars);
                get_vars_body(else_, vars);
            },
            Stmt::While(cond, body) => {
                get_vars_expr(cond, vars);
                get_vars_body(body, vars);
            },
            Stmt::Print(e) => {
                get_vars_expr(e, vars);
            },
        }
    }
}

fn get_vars_expr(expr: &Expr, vars: &mut HashSet<String>) {
    match expr {
        Expr::BinOp(_, e1, e2) => {
            get_vars_expr(e1, vars);
            get_vars_expr(e2, vars);
        },
        Expr::FnCall(_f, args) => {
            for a in args {
                get_vars_expr(a, vars);
            }
        },
        Expr::IntLit(_) => {},
        Expr::StringLit(_) => {},
        Expr::BoolLit(_) => {},
        Expr::Var(v) => {
            vars.insert(v.to_string());
        },
        Expr::Input => {},
    }
}
