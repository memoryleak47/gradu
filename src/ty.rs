use crate::*;

#[derive(Clone, Copy)]
pub struct TypeLattice {
    might_be_bool: bool,
    might_be_nil: bool,
    might_be_str: bool,
    might_be_int: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    Value, // "any"
}

pub type TyCtxt = HashMap<String, LayoutType>;
type TyLatticeCtxt = HashMap<String, TypeLattice>;

pub fn ty_infer(ast: &AST) -> TyCtxt {
    let m = &mut HashMap::new();

    // After 5 rounds, we have to have converged!
    for _ in 0..5 {
        ty_infer_ast(ast, m);
    }

    let mut out = HashMap::new();
    for v in get_vars(ast) {
        let ty = get_var(&v, m);
        out.insert(v.to_string(), layout(ty));
    }
    out
}

fn ty_infer_ast(ast: &AST, ctxt: &mut TyLatticeCtxt) {
    for st in ast {
        ty_infer_stmt(st, ctxt);
    }
}

fn ty_infer_stmt(stmt: &Stmt, ctxt: &mut TyLatticeCtxt) {
    match stmt {
        Stmt::Assign(v, e) => {
            let l = get_var(v, ctxt);
            let r = ty_infer_expr(e, ctxt);
            ctxt.insert(v.to_string(), TypeLattice::merge(l, r));
        },
        Stmt::If(_, then_, else_) => {
            ty_infer_ast(then_, ctxt);
            ty_infer_ast(else_, ctxt);
        },
        Stmt::While(_, body) => {
            ty_infer_ast(body, ctxt);
        },
        Stmt::Print(_) => {},
    }
}

fn get_var(v: &str, ctxt: &TyLatticeCtxt) -> TypeLattice {
    ctxt.get(v).cloned().unwrap_or(TypeLattice::bot())
}

fn ty_infer_expr(expr: &Expr, ctxt: &TyLatticeCtxt) -> TypeLattice {
    match expr {
        Expr::BinOp(k, e1, e2) => TypeLattice::top(),
        Expr::IntLit(_) => TypeLattice { might_be_int: true, ..TypeLattice::bot() },
        Expr::StringLit(_) => TypeLattice { might_be_str: true, ..TypeLattice::bot() },
        Expr::BoolLit(_) => TypeLattice { might_be_bool: true, ..TypeLattice::bot() },
        Expr::Var(v) => get_var(v, ctxt),
        Expr::Input => TypeLattice::top(),
    }
}


impl TypeLattice {
    fn top() -> Self {
        TypeLattice {
            might_be_bool: true,
            might_be_nil: true,
            might_be_str: true,
            might_be_int: true,
        }
    }

    fn bot() -> Self {
        TypeLattice {
            might_be_bool: false,
            might_be_nil: false,
            might_be_str: false,
            might_be_int: false,
        }
    }

    fn merge(x: TypeLattice, y: TypeLattice) -> TypeLattice {
        TypeLattice {
            might_be_bool: x.might_be_bool || y.might_be_bool,
            might_be_nil: x.might_be_nil|| y.might_be_nil,
            might_be_str: x.might_be_str || y.might_be_str,
            might_be_int: x.might_be_int || y.might_be_int,
        }
    }
}

fn layout(x: TypeLattice) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else { LayoutType::Value }
}
