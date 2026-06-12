use crate::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeLattice {
    pub might_be_bool: bool,
    pub might_be_nil: bool,
    pub might_be_str: bool,
    pub might_be_int: bool,
    pub might_be_list: bool,
    pub fn_options: HashSet<FnId>,
}

// analysis context.
pub type ACtxt = HashMap<Location, TypeLattice>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Location {
    Var(/*fn*/ FnId, /*var*/ Symbol), // also includes fn args
    GlobalVar(/*var*/ Symbol),
    RetVal(/*fn*/ FnId),
    ListItem,
}

// points to a `Expr::NewList`.
pub type ListLoc = *const Expr;

pub fn analyze(ast: &AST, nameres: &Nameres) -> ACtxt {
    let mut actxt = HashMap::new();

    'outer: loop {
        let bkp = actxt.clone();
        for (fid, fdef) in ast.fns.iter().enumerate() {
            ty_infer_body(&fdef.body, fid, ast, nameres, &mut actxt);
        }
        if bkp == actxt { break 'outer; }
    }

    actxt
}

pub fn get(x: Location, actxt: &ACtxt) -> TypeLattice {
    actxt.get(&x).cloned().unwrap_or(TypeLattice::bot())
}

fn ty_infer_body(body: &Body, fid: FnId, ast: &AST, nameres: &Nameres, actxt: &mut ACtxt) {
    for st in body {
        ty_infer_stmt(st, fid, ast, nameres, actxt);
    }
}

fn ty_infer_stmt(stmt: &Stmt, fid: FnId, ast: &AST, nameres: &Nameres, actxt: &mut ACtxt) {
    match stmt {
        Stmt::Global(_) => {},
        Stmt::ListStore(l, i, v) => {
            let _l = ty_infer_expr(l, fid, ast, nameres, actxt);
            let _i = ty_infer_expr(i, fid, ast, nameres, actxt);
            let v = ty_infer_expr(v, fid, ast, nameres, actxt);
            add(Location::ListItem, &v, actxt);
        },
        Stmt::Push(l, v) => {
            let _l = ty_infer_expr(l, fid, ast, nameres, actxt);
            let v = ty_infer_expr(v, fid, ast, nameres, actxt);
            add(Location::ListItem, &v, actxt);
        },
        Stmt::Assign(v, e) => {
            let r = ty_infer_expr(e, fid, ast, nameres, actxt);
            let l = get_var_loc(fid, *v, nameres);
            add(l, &r, actxt);
        },
        Stmt::Return(e) => {
            let r = ty_infer_expr(e, fid, ast, nameres, actxt);
            let l = Location::RetVal(fid);
            add(l, &r, actxt);
        },
        Stmt::If(c, then_, else_) => {
            ty_infer_expr(c, fid, ast, nameres, actxt);
            ty_infer_body(then_, fid, ast, nameres, actxt);
            ty_infer_body(else_, fid, ast, nameres, actxt);
        },
        Stmt::While(c, body) => {
            ty_infer_expr(c, fid, ast, nameres, actxt);
            ty_infer_body(body, fid, ast, nameres, actxt);
        },
        Stmt::Print(e) => {
            ty_infer_expr(e, fid, ast, nameres, actxt);
        },
    }
}

pub fn add(v: Location, ty: &TypeLattice, actxt: &mut ACtxt) {
    let ty2 = get(v, actxt);
    let ty = TypeLattice::merge(ty, &ty2);
    actxt.insert(v, ty);
}

pub fn ty_infer_expr(expr: &Expr, fid: FnId, ast: &AST, nameres: &Nameres, actxt: &mut ACtxt) -> TypeLattice {
    match expr {
        Expr::FnId(f) => {
            TypeLattice { fn_options: std::iter::once(*f).collect(), ..TypeLattice::bot() }
        },
        Expr::Length(l) => {
            let _l = ty_infer_expr(l, fid, ast, nameres, actxt);
            TypeLattice { might_be_int: true, ..TypeLattice::bot() }
        },
        Expr::NewList => {
            TypeLattice { might_be_list: true, ..TypeLattice::bot() }
        },
        Expr::IndexList(l, i) => {
            let _l = ty_infer_expr(l, fid, ast, nameres, actxt);
            let _i = ty_infer_expr(i, fid, ast, nameres, actxt);

            get(Location::ListItem, actxt)
        },
        Expr::FnCall(f, args) => {
            let f = ty_infer_expr(f, fid, ast, nameres, actxt);
            let args: Vec<_> = args.iter().map(|x| ty_infer_expr(x, fid, ast, nameres, actxt)).collect();

            let mut ret = TypeLattice::bot();
            for callee_fid in f.fn_options.iter().copied() {
                let callee_fdef = &ast.fns[callee_fid];
                if callee_fdef.args.len() != args.len() { continue }

                // push `args` data to all possible callees.
                for i in 0..args.len() {
                    let l = Location::Var(callee_fid, callee_fdef.args[i]);
                    add(l, &args[i], actxt);
                }

                // union return value from all possible callees.
                let callee_ret = get(Location::RetVal(callee_fid), actxt);
                ret = TypeLattice::merge(&ret, &callee_ret);
            }

            ret
        },
        Expr::BinOp(kind, l, r) => {
            let _l = ty_infer_expr(l, fid, ast, nameres, actxt);
            let _r = ty_infer_expr(r, fid, ast, nameres, actxt);
            match kind {
                BinOpKind::Equ | BinOpKind::Ne | BinOpKind::Lt | BinOpKind::Gt =>
                    TypeLattice { might_be_bool: true, ..TypeLattice::bot() },
                BinOpKind::Plus | BinOpKind::Mod | BinOpKind::Minus | BinOpKind::Mul =>
                    TypeLattice { might_be_int: true, ..TypeLattice::bot() },
            }
        },
        Expr::IntLit(_) => TypeLattice { might_be_int: true, ..TypeLattice::bot() },
        Expr::StringLit(_) => TypeLattice { might_be_str: true, ..TypeLattice::bot() },
        Expr::BoolLit(_) => TypeLattice { might_be_bool: true, ..TypeLattice::bot() },
        Expr::Var(v) => {
            let l = get_var_loc(fid, *v, nameres);
            get(l, actxt)
        }
        Expr::Input => TypeLattice {
            might_be_bool: true,
            might_be_nil: true,
            might_be_str: true,
            might_be_int: true,
            might_be_list: false,
            fn_options: HashSet::new(),
        },
    }
}

impl TypeLattice {
    pub fn bot() -> Self {
        TypeLattice {
            might_be_bool: false,
            might_be_nil: false,
            might_be_str: false,
            might_be_int: false,
            might_be_list: false,
            fn_options: HashSet::new(),
        }
    }

    pub fn merge(x: &TypeLattice, y: &TypeLattice) -> TypeLattice {
        TypeLattice {
            might_be_bool: x.might_be_bool || y.might_be_bool,
            might_be_nil: x.might_be_nil|| y.might_be_nil,
            might_be_str: x.might_be_str || y.might_be_str,
            might_be_int: x.might_be_int || y.might_be_int,
            might_be_list: x.might_be_list || y.might_be_list,
            fn_options: x.fn_options.union(&y.fn_options).copied().collect(),
        }
    }
}

pub fn get_var_loc(fid: FnId, varname: Symbol, nameres: &Nameres) -> Location {
    match nameres.vars[fid][&varname] {
        VarKind::Global => Location::GlobalVar(varname),
        VarKind::Local => Location::Var(fid, varname),
    }
}
