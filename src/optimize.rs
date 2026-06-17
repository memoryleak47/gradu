use crate::*;

// return true if something happened.
pub fn optimize(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    if inline_const_global_read(ast, nameres, actxt) { return true }
    if inline_const_local_read(ast, nameres, actxt) { return true }
    if remove_unreachable_stmts(ast) { return true }
    if redundant_local_write_elimination(ast, nameres) { return true }
    false
}

fn inline_const_global_read(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    for &g in &nameres.globals {
        let lat = get(Location::GlobalVar(g), actxt);
        let Some(e) = as_concrete_expr(lat) else { continue };
        for (fid, f) in ast.fns.iter_mut().enumerate() {
            if nameres.vars[fid].get(&g) != Some(&VarKind::Global) { continue }

            visit_body_mut(&mut f.body, &mut |expr|{
                if expr == &Expr::Var(g) {
                    *expr = e.clone();
                }
            }, &mut |_|{});
        }
    }
    false
}

fn inline_const_local_read(ast: &mut AST, nameres: &mut Nameres, actxt: &ACtxt) -> bool {
    for (fid, f) in ast.fns.iter_mut().enumerate() {
        for (&v, &kind) in &nameres.vars[fid] {
            let VarKind::Local = kind else { continue };
            let lat = get(Location::Var(fid, v), actxt);
            let Some(e) = as_concrete_expr(lat) else { continue };

            visit_body_mut(&mut f.body, &mut |expr|{
                if expr == &Expr::Var(v) {
                    *expr = e.clone();
                }
            }, &mut |_|{});
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


fn remove_unreachable_stmts(ast: &mut AST) -> bool {
    let mut changed = false;
    for f in ast.fns.iter_mut() {
        unreach(&mut f.body, &mut changed);
    }
    changed
}

// returns true if it certainly returned.
fn unreach(body: &mut Vec<Stmt>, changed: &mut bool) -> bool {
    for i in 0..body.len() {
        let stop = match &mut body[i] {
            Stmt::Return(_) => true,
            Stmt::If(_, then_, else_) => unreach(then_, changed) && unreach(else_, changed),
            Stmt::While(_, body_) => {
                unreach(body_, changed);
                false
            },
            _ => false,
        };
        if stop {
            if body.len() != i+1 { *changed = true; }
            body.truncate(i+1);
            return true;
        }
    }
    false
}

fn redundant_local_write_elimination(ast: &mut AST, nameres: &Nameres) -> bool {
    let mut changed = false;
    for (fid, f) in ast.fns.iter_mut().enumerate() {
        let mut good_writes = HashSet::new();
        let state = State::new();
        prop_body(&f.body, nameres, state, fid, &mut good_writes);

        visit_body_mut(&mut f.body, &mut |_|{}, &mut |stmt: &mut Stmt| {
            let stmt_ref = stmt as *const Stmt;
            let Stmt::Assign(v, e) = stmt else { return };
            let VarKind::Local = nameres.vars[fid][v] else { return };
            if good_writes.contains(&stmt_ref) { return }
            if !side_effect_free(e) { return }

            changed = true;
            *stmt = Stmt::If(Expr::BoolLit(true), Vec::new(), Vec::new()); // nop
        });
    }
    changed
}

// Remembers at what Write Locations each Variable might have been written the last time.
type State = HashMap<Symbol, HashSet<WriteLoc>>;
type WriteLoc = *const Stmt;

fn prop_body(body: &Body, nameres: &Nameres, mut state: State, fid: FnId, good_writes: &mut HashSet<WriteLoc>) -> State {
    use Stmt::*;
    for stmt in body {
        let mut handle_expr = |expr: &Expr| {
            visit_expr(expr, &mut |expr| {
                let Expr::Var(v) = expr else { return };
                let VarKind::Local = nameres.vars[fid][v] else { return };
                let Some(x) = state.get(v) else { return };
                good_writes.extend(x);
            }, &mut |_|{});
        };
        match stmt {
            Global(_) => {},
            Return(e) | Assign(_, e) | If(e, _, _) | While(e, _) | Print(e) => handle_expr(e),
            Push(e1, e2) => {
                handle_expr(e1);
                handle_expr(e2);
            },
            ListStore(e1, e2, e3) | DictStore(e1, e2, e3) => {
                handle_expr(e1);
                handle_expr(e2);
                handle_expr(e3);
            },
        }

        match stmt {
            Stmt::Assign(v, _) => {
                state.insert(*v, std::iter::once(stmt as *const Stmt).collect());
            },
            Stmt::If(_, then_, else_) => {
                let mut new_state = prop_body(then_, nameres, state.clone(), fid, good_writes);
                for (x, y) in prop_body(else_, nameres, state, fid, good_writes) {
                    new_state.entry(x).or_default().extend(y);
                }
                state = new_state;
            },
            Stmt::While(_, b) => {
                // The first iteration adds all new "writes" to the state, the second derives the "good_writes" from them.
                for _ in 0..2 {
                    for (x, y) in prop_body(b, nameres, state.clone(), fid, good_writes) {
                        state.entry(x).or_default().extend(y);
                    }
                }
            },
            _ => {},
        }
    }

    state
}

fn side_effect_free(e: &Expr) -> bool {
    use Expr::*;
    match e {
        FnId(_) | NewList | NewDict | Var(_) | IntLit(_) | StringLit(_) | BoolLit(_) | NilLit => true,
        Input | FnCall(_, _) => false,
        IndexList(a, b) | IndexDict(a, b) | BinOp(_, a, b) => side_effect_free(a) && side_effect_free(b),
        Length(a) => side_effect_free(a),
    }
}
