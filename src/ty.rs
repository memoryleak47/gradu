use crate::*;

#[derive(Clone)]
pub struct TypeLattice {
    pub might_be_bool: bool,
    pub might_be_nil: bool,
    pub might_be_str: bool,
    pub might_be_int: bool,
    pub might_be_list: bool,
    pub fn_options: HashSet<FnId>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    List,
    Fn(Vec<LayoutType>, Box<LayoutType>),
    Value, // "any"
}

pub type TyCtxt = HashMap<Location, LayoutType>;
pub type TyLatticeCtxt = HashMap<Location, TypeLattice>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Location {
    Var(/*fn*/ FnId, /*var*/ Symbol), // also includes fn args
    GlobalVar(/*var*/ Symbol),
    RetVal(/*fn*/ FnId),
    ListItem,
}

// points to a `Expr::NewList`.
pub type ListLoc = *const Expr;

pub fn ty_infer(ast: &AST) -> (TyLatticeCtxt, TyCtxt) {
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

    let m2 = m.iter()
     .map(|(v, ty)| (*v, layout(ty.clone(), &m, ast)))
     .collect();
    (m, m2)
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

pub fn ty_infer_expr(expr: &Expr, fid: FnId, ast: &AST, ctxt: &mut TyLatticeCtxt) -> TypeLattice {
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

            let mut callee_options = Vec::new();
            for callee_fid in f.fn_options.iter().copied() {
                let callee_fdef = &ast.fns[callee_fid];
                if callee_fdef.args.len() != args.len() { continue }
                callee_options.push(callee_fid);
            }

            for i in 0..args.len() {
                // All of those fns need to have the same layout.
                // We guarantee this by giving them the same TypeLattice.
                let mut argty = ty_infer_expr(&args[i], fid, ast, ctxt);

                // accumulate argty
                for &callee_fid in &callee_options {
                    let callee_fdef = &ast.fns[callee_fid];
                    let l = Location::Var(callee_fid, callee_fdef.args[i]);
                    argty = TypeLattice::merge(&argty, &get(l, ctxt));
                }

                // write back argty.
                for &callee_fid in &callee_options {
                    let callee_fdef = &ast.fns[callee_fid];
                    let l = Location::Var(callee_fid, callee_fdef.args[i]);
                    add(l, &argty, ctxt);
                }
            }

            let mut callee_ret_type = TypeLattice::bot();

            // accumulate compute ret type
            for &callee_fid in &callee_options {
                let l = Location::RetVal(callee_fid);
                callee_ret_type = TypeLattice::merge(&callee_ret_type, &get(l, ctxt));
            }

            // write back ret type
            for &callee_fid in &callee_options {
                let l = Location::RetVal(callee_fid);
                add(l, &callee_ret_type, ctxt);
            }

            callee_ret_type
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

pub fn layout(x: TypeLattice, ctxt: &TyLatticeCtxt, ast: &AST) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) + (x.might_be_list as u8) + ((x.fn_options.len() > 0) as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else if x.might_be_list { LayoutType::List }
    else if x.might_be_list { LayoutType::List }
    else if let Some(&fid) = x.fn_options.iter().next() { // TODO we have to guarantee that all those fn have the same argtys & retty.
        fn_type_of(fid, ast, ctxt)
    } else { LayoutType::Value }
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

pub fn fn_type_of(fid: FnId, ast: &AST, ctxt: &TyLatticeCtxt) -> LayoutType {
    let mut argtys = Vec::new();
    for &a in &ast.fns[fid].args {
        argtys.push(layout(get(Location::Var(fid, a), ctxt), ctxt, ast));
    }
    let retty = layout(get(Location::RetVal(fid), ctxt), ctxt, ast);

    LayoutType::Fn(argtys, Box::new(retty.clone()))
}
