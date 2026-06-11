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

pub fn analyze(ast: &AST) -> ACtxt {
    let mut actxt = HashMap::new();

    'outer: loop {
        let bkp = actxt.clone();
        for (fid, fdef) in ast.fns.iter().enumerate() {
            ty_infer_body(&fdef.body, fid, ast, &mut actxt);
        }
        if bkp == actxt { break 'outer; }
    }

    actxt
}

pub fn get(x: Location, actxt: &ACtxt) -> TypeLattice {
    actxt.get(&x).cloned().unwrap_or(TypeLattice::bot())
}

fn ty_infer_body(body: &Body, fid: FnId, ast: &AST, actxt: &mut ACtxt) {
    for st in body {
        ty_infer_stmt(st, fid, ast, actxt);
    }
}

fn ty_infer_stmt(stmt: &Stmt, fid: FnId, ast: &AST, actxt: &mut ACtxt) {
    match stmt {
        Stmt::Global(_) => {},
        Stmt::ListStore(l, i, v) => {
            let _l = ty_infer_expr(l, fid, ast, actxt);
            let _i = ty_infer_expr(i, fid, ast, actxt);
            let v = ty_infer_expr(v, fid, ast, actxt);
            add(Location::ListItem, &v, actxt);
        },
        Stmt::Push(l, v) => {
            let _l = ty_infer_expr(l, fid, ast, actxt);
            let v = ty_infer_expr(v, fid, ast, actxt);
            add(Location::ListItem, &v, actxt);
        },
        Stmt::Assign(v, e) => {
            let r = ty_infer_expr(e, fid, ast, actxt);
            let l = get_var_loc(fid, *v, ast);
            add(l, &r, actxt);
        },
        Stmt::Return(e) => {
            let r = ty_infer_expr(e, fid, ast, actxt);
            let l = Location::RetVal(fid);
            add(l, &r, actxt);
        },
        Stmt::If(c, then_, else_) => {
            ty_infer_expr(c, fid, ast, actxt);
            ty_infer_body(then_, fid, ast, actxt);
            ty_infer_body(else_, fid, ast, actxt);
        },
        Stmt::While(c, body) => {
            ty_infer_expr(c, fid, ast, actxt);
            ty_infer_body(body, fid, ast, actxt);
        },
        Stmt::Print(e) => {
            ty_infer_expr(e, fid, ast, actxt);
        },
    }
}

pub fn add(v: Location, ty: &TypeLattice, actxt: &mut ACtxt) {
    let ty2 = get(v, actxt);
    let ty = TypeLattice::merge(ty, &ty2);
    actxt.insert(v, ty);
}

pub fn ty_infer_expr(expr: &Expr, fid: FnId, ast: &AST, actxt: &mut ACtxt) -> TypeLattice {
    match expr {
        Expr::FnId(f) => {
            TypeLattice { fn_options: std::iter::once(*f).collect(), ..TypeLattice::bot() }
        },
        Expr::Length(l) => {
            let _l = ty_infer_expr(l, fid, ast, actxt);
            TypeLattice { might_be_int: true, ..TypeLattice::bot() }
        },
        Expr::NewList => {
            TypeLattice { might_be_list: true, ..TypeLattice::bot() }
        },
        Expr::IndexList(l, i) => {
            let _l = ty_infer_expr(l, fid, ast, actxt);
            let _i = ty_infer_expr(i, fid, ast, actxt);

            get(Location::ListItem, actxt)
        },
        Expr::FnCall(f, args) => {
            let f = ty_infer_expr(f, fid, ast, actxt);
            let args: Vec<_> = args.iter().map(|x| ty_infer_expr(x, fid, ast, actxt)).collect();

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
            let _l = ty_infer_expr(l, fid, ast, actxt);
            let _r = ty_infer_expr(r, fid, ast, actxt);
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
            let l = get_var_loc(fid, *v, ast);
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

// TODO: this should be cached obviously.
pub fn is_global_var(fid: FnId, varname: Symbol, ast: &AST) -> bool {
    if fid == ast.main_fn { return true } // TODO main-fn variables can remain "local" if no one else reads them. would be faster.
    let f = &ast.fns[fid];
    if f.args.contains(&varname) { return false }
    if f.body.contains(&Stmt::Global(varname)) { return true }
    if !is_assigned(fid, varname, ast) { return true }


    false
}

fn is_assigned(fid: FnId, varname: Symbol, ast: &AST) -> bool {
    let body = &ast.fns[fid].body;
    let mut assigned = false;
    visit_body(body, &mut |_|{}, &mut |stmt| {
        if let Stmt::Assign(v, _) = stmt && *v == varname {
            assigned = true;
        }
    });

    assigned
}


pub fn get_var_loc(fid: FnId, varname: Symbol, ast: &AST) -> Location {
    if is_global_var(fid, varname, ast) {
        Location::GlobalVar(varname)
    } else {
        Location::Var(fid, varname)
    }
}
