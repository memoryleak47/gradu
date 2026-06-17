use crate::*;

pub fn visit_body(body: &Body, f_expr: &mut impl FnMut(&Expr), f_stmt: &mut impl FnMut(&Stmt)) {
    for s in body {
        visit_stmt(s, f_expr, f_stmt);
    }
}

pub fn visit_stmt(stmt: &Stmt, f_expr: &mut impl FnMut(&Expr), f_stmt: &mut impl FnMut(&Stmt)) {
    use Stmt::*;

    f_stmt(stmt);

    match stmt {
        Global(_) => {},
        ListStore(l, i, v) => {
            visit_expr(l, f_expr, f_stmt);
            visit_expr(i, f_expr, f_stmt);
            visit_expr(v, f_expr, f_stmt);
        },
        DictStore(t, k, v) => {
            visit_expr(t, f_expr, f_stmt);
            visit_expr(k, f_expr, f_stmt);
            visit_expr(v, f_expr, f_stmt);
        },
        Push(l, v) => {
            visit_expr(l, f_expr, f_stmt);
            visit_expr(v, f_expr, f_stmt);
        },

        Return(e) => {
            visit_expr(e, f_expr, f_stmt);
        },
        Assign(_, e) => {
            visit_expr(e, f_expr, f_stmt);
        },
        If(c, then_, else_) => {
            visit_expr(c, f_expr, f_stmt);
            visit_body(then_, f_expr, f_stmt);
            visit_body(else_, f_expr, f_stmt);
        },
        While(c, b) => {
            visit_expr(c, f_expr, f_stmt);
            visit_body(b, f_expr, f_stmt);
        },
        Print(e) => {
            visit_expr(e, f_expr, f_stmt);
        },
    }
}

pub fn visit_expr(expr: &Expr, f_expr: &mut impl FnMut(&Expr), f_stmt: &mut impl FnMut(&Stmt)) {
    use Expr::*;

    f_expr(expr);

    match expr {
        FnId(_) => {},
        NewList => {},
        NewDict => {},
        Length(l) => {
            visit_expr(l, f_expr, f_stmt);
        },
        IndexList(l, i) => {
            visit_expr(l, f_expr, f_stmt);
            visit_expr(i, f_expr, f_stmt);
        },
        IndexDict(t, k) => {
            visit_expr(t, f_expr, f_stmt);
            visit_expr(k, f_expr, f_stmt);
        },
        BinOp(_, e1, e2) => {
            visit_expr(e1, f_expr, f_stmt);
            visit_expr(e2, f_expr, f_stmt);
        },
        Var(_) => {},
        Input => {},
        FnCall(f, es) => {
            visit_expr(f, f_expr, f_stmt);
            for e in es {
                visit_expr(e, f_expr, f_stmt);
            }
        },

        IntLit(_) => {},
        StringLit(_) => {},
        BoolLit(_) => {},
        NilLit => {},
    }
}

//////////////
// mutable! //
//////////////

pub fn visit_body_mut(body: &mut Body, f_expr: &mut impl FnMut(&mut Expr), f_stmt: &mut impl FnMut(&mut Stmt)) {
    for s in body {
        visit_stmt_mut(s, f_expr, f_stmt);
    }
}

fn visit_stmt_mut(stmt: &mut Stmt, f_expr: &mut impl FnMut(&mut Expr), f_stmt: &mut impl FnMut(&mut Stmt)) {
    use Stmt::*;

    f_stmt(stmt);

    match stmt {
        Global(_) => {},
        ListStore(l, i, v) => {
            visit_expr_mut(l, f_expr, f_stmt);
            visit_expr_mut(i, f_expr, f_stmt);
            visit_expr_mut(v, f_expr, f_stmt);
        },
        DictStore(t, k, v) => {
            visit_expr_mut(t, f_expr, f_stmt);
            visit_expr_mut(k, f_expr, f_stmt);
            visit_expr_mut(v, f_expr, f_stmt);
        },
        Push(l, v) => {
            visit_expr_mut(l, f_expr, f_stmt);
            visit_expr_mut(v, f_expr, f_stmt);
        },

        Return(e) => {
            visit_expr_mut(e, f_expr, f_stmt);
        },
        Assign(_, e) => {
            visit_expr_mut(e, f_expr, f_stmt);
        },
        If(c, then_, else_) => {
            visit_expr_mut(c, f_expr, f_stmt);
            visit_body_mut(then_, f_expr, f_stmt);
            visit_body_mut(else_, f_expr, f_stmt);
        },
        While(c, b) => {
            visit_expr_mut(c, f_expr, f_stmt);
            visit_body_mut(b, f_expr, f_stmt);
        },
        Print(e) => {
            visit_expr_mut(e, f_expr, f_stmt);
        },
    }
}

fn visit_expr_mut(expr: &mut Expr, f_expr: &mut impl FnMut(&mut Expr), f_stmt: &mut impl FnMut(&mut Stmt)) {
    use Expr::*;

    f_expr(expr);

    match expr {
        FnId(_) => {},
        NewList => {},
        NewDict => {},
        Length(l) => {
            visit_expr_mut(l, f_expr, f_stmt);
        },
        IndexList(l, i) => {
            visit_expr_mut(l, f_expr, f_stmt);
            visit_expr_mut(i, f_expr, f_stmt);
        },
        IndexDict(t, k) => {
            visit_expr_mut(t, f_expr, f_stmt);
            visit_expr_mut(k, f_expr, f_stmt);
        },
        BinOp(_, e1, e2) => {
            visit_expr_mut(e1, f_expr, f_stmt);
            visit_expr_mut(e2, f_expr, f_stmt);
        },
        Var(_) => {},
        Input => {},
        FnCall(f, es) => {
            visit_expr_mut(f, f_expr, f_stmt);
            for e in es {
                visit_expr_mut(e, f_expr, f_stmt);
            }
        },

        IntLit(_) => {},
        StringLit(_) => {},
        BoolLit(_) => {},
        NilLit => {},
    }
}
