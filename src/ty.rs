use crate::*;

#[derive(Clone)]
pub struct TypeLattice {
    might_be_bool: bool,
    might_be_nil: bool,
    might_be_str: bool,
    might_be_int: bool,
    might_be_list: bool,
    fn_options: HashSet<FnId>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    List,
    Fn,
    Value, // "any"
}

pub type TyCtxt = HashMap<Location, LayoutType>;
type TyLatticeCtxt = HashMap<Location, TypeLattice>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Location {
    Var(/*fn*/ FnId, /*var*/ Symbol), // also includes fn args
    GlobalVar(/*var*/ Symbol),
    RetVal(/*fn*/ FnId),
    ListItem,
}

// points to a `Expr::NewList`.
pub type ListLoc = *const Expr;

pub fn ty_infer(ast: &AST) -> TyCtxt {
    let mut m = HashMap::new();

    // initilize `m`.
    for (fid, fdef) in ast.fns.iter().enumerate() {
        let l = Location::RetVal(fid);
        m.insert(l, TypeLattice::bot());

        for v in get_vars(fdef) {
            let l = get_var_loc(fid, v, ast);
            m.insert(l, TypeLattice::bot());
        }
    }

    m.insert(Location::ListItem, TypeLattice::bot());

    // After 5 rounds, we have to have converged!
    for _ in 0..5 {
        for (fid, fdef) in ast.fns.iter().enumerate() {
            ty_infer_body(&fdef.body, fid, ast, &mut m);
        }
    }

    m.into_iter()
     .map(|(v, ty)| (v, layout(ty)))
     .collect()
}

fn get(x: Location, ctxt: &TyLatticeCtxt) -> TypeLattice {
    ctxt.get(&x).cloned().unwrap_or(TypeLattice::bot())
}

fn ty_infer_body(body: &Body, fid: FnId, ast: &AST, ctxt: &mut TyLatticeCtxt) {
    for st in body {
        ty_infer_stmt(st, fid, ast, ctxt);
    }
}

fn ty_infer_stmt(stmt: &Stmt, fid: FnId, ast: &AST, ctxt: &mut TyLatticeCtxt) {
    match stmt {
        Stmt::Global(_) => {},
        Stmt::ListStore(l, i, v) => {
            let _l = ty_infer_expr(l, fid, ast, ctxt);
            let _i = ty_infer_expr(i, fid, ast, ctxt);
            let v = ty_infer_expr(v, fid, ast, ctxt);
            add(Location::ListItem, &v, ctxt);
        },
        Stmt::Push(l, v) => {
            let _l = ty_infer_expr(l, fid, ast, ctxt);
            let v = ty_infer_expr(v, fid, ast, ctxt);
            add(Location::ListItem, &v, ctxt);
        },
        Stmt::Assign(v, e) => {
            let r = ty_infer_expr(e, fid, ast, ctxt);
            let l = get_var_loc(fid, *v, ast);
            add(l, &r, ctxt);
        },
        Stmt::Return(e) => {
            let r = ty_infer_expr(e, fid, ast, ctxt);
            let l = Location::RetVal(fid);
            add(l, &r, ctxt);
        },
        Stmt::If(c, then_, else_) => {
            ty_infer_expr(c, fid, ast, ctxt);
            ty_infer_body(then_, fid, ast, ctxt);
            ty_infer_body(else_, fid, ast, ctxt);
        },
        Stmt::While(c, body) => {
            ty_infer_expr(c, fid, ast, ctxt);
            ty_infer_body(body, fid, ast, ctxt);
        },
        Stmt::Print(e) => {
            ty_infer_expr(e, fid, ast, ctxt);
        },
    }
}

fn add(v: Location, ty: &TypeLattice, ctxt: &mut TyLatticeCtxt) {
    let ty2 = get(v, ctxt);
    let ty = TypeLattice::merge(ty, &ty2);
    ctxt.insert(v, ty);
}

fn ty_infer_expr(expr: &Expr, fid: FnId, ast: &AST, ctxt: &mut TyLatticeCtxt) -> TypeLattice {
    match expr {
        Expr::FnId(f) => {
            TypeLattice { fn_options: std::iter::once(*f).collect(), ..TypeLattice::bot() }
        },
        Expr::Length(l) => {
            let _l = ty_infer_expr(l, fid, ast, ctxt);
            TypeLattice { might_be_int: true, ..TypeLattice::bot() }
        },
        Expr::NewList => {
            TypeLattice { might_be_list: true, ..TypeLattice::bot() }
        },
        Expr::IndexList(l, i) => {
            let _l = ty_infer_expr(l, fid, ast, ctxt);
            let _i = ty_infer_expr(i, fid, ast, ctxt);

            get(Location::ListItem, ctxt)
        },
        Expr::FnCall(f, args) => {
            let f = ty_infer_expr(f, fid, ast, ctxt);

            let mut out = TypeLattice::bot();

            for callee_fid in f.fn_options.iter().copied() {
                let callee_fdef = &ast.fns[callee_fid];
                if callee_fdef.args.len() != args.len() { continue }

                for (argname, argexpr) in callee_fdef.args.iter().zip(args) {
                    let argexpr_ty = ty_infer_expr(argexpr, fid, ast, ctxt);
                    let l = Location::Var(callee_fid, *argname);
                    add(l, &argexpr_ty, ctxt);
                }

                let l = Location::RetVal(callee_fid);
                out = TypeLattice::merge(&out, &get(l, ctxt));
            }

            out
        },
        Expr::BinOp(kind, l, r) => {
            let _l = ty_infer_expr(l, fid, ast, ctxt);
            let _r = ty_infer_expr(r, fid, ast, ctxt);
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
            get(l, ctxt)
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
    fn bot() -> Self {
        TypeLattice {
            might_be_bool: false,
            might_be_nil: false,
            might_be_str: false,
            might_be_int: false,
            might_be_list: false,
            fn_options: HashSet::new(),
        }
    }

    fn merge(x: &TypeLattice, y: &TypeLattice) -> TypeLattice {
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

fn layout(x: TypeLattice) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) + (x.might_be_list as u8) + ((x.fn_options.len() > 0) as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else if x.might_be_list { LayoutType::List }
    else if x.might_be_list { LayoutType::List }
    else if x.fn_options.len() > 0 { LayoutType::Fn }
    else { LayoutType::Value }
}

pub fn is_global_var(fid: FnId, varname: Symbol, ast: &AST) -> bool {
    if fid == ast.main_fn { return true }
    let f = &ast.fns[fid];
    if f.body.contains(&Stmt::Global(varname)) { return true }

    // In python, if you only read a variable without ever writing to it, it's implicitly global.
    // I'll ignore that for now though.

    false
}

pub fn get_var_loc(fid: FnId, varname: Symbol, ast: &AST) -> Location {
    if is_global_var(fid, varname, ast) {
        Location::GlobalVar(varname)
    } else {
        Location::Var(fid, varname)
    }
}
