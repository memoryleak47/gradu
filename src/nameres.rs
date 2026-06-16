use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VarKind {
    Global,
    Local
}

pub struct Nameres {
    pub vars: Vec<HashMap<Symbol, VarKind>>, // indexed by FnId and Varname
    pub globals: HashSet<Symbol>,
}

pub fn nameres(ast: &AST) -> Nameres {
    let mut vars = Vec::new();
    let mut globals = HashSet::new();

    let mut fnids: Vec<_> = (0..ast.fns.len()).collect();

    // so "main" is at the last position.
    let main_idx = fnids.iter().position(|x| *x == ast.main_fn).unwrap();
    fnids.swap_remove(main_idx);
    fnids.push(ast.main_fn);

    for &fid in &fnids {
        let f = &ast.fns[fid];
        let mut read: HashSet<Symbol> = HashSet::new();
        let mut assigned: HashSet<Symbol> = HashSet::new();
        let mut global: HashSet<Symbol> = HashSet::new();
        let args: HashSet<Symbol> = ast.fns[fid].args.iter().copied().collect();

        let mut f_expr = |expr: &_| {
            if let Expr::Var(v) = expr {
                read.insert(*v);
            }
        };
        let mut f_stmt = |stmt: &_| {
            if let Stmt::Assign(v, _) = stmt {
                assigned.insert(*v);
            } else if let Stmt::Global(v) = stmt {
                global.insert(*v);
            }
        };
        visit_body(&f.body, &mut f_expr, &mut f_stmt);

        let mut vmap = HashMap::new();

        for x in &(&(&global | &read) | &assigned) | &args {
            let kind =
                if fid == ast.main_fn {
                    if globals.contains(&x) { // if some other fn accesses it, then we have to make it global.
                        VarKind::Global
                    } else {
                        VarKind::Local
                    }
                } else {
                    if args.contains(&x) {
                        VarKind::Local
                    } else if global.contains(&x) {
                        VarKind::Global
                    } else if assigned.contains(&x) && fid != ast.main_fn {
                        VarKind::Local
                    } else { // only read.
                        VarKind::Global
                    }
                };
            if let VarKind::Global = kind {
                globals.insert(x);
            }
            vmap.insert(x, kind);
        }

        vars.push(vmap);
    }

    Nameres {
        vars,
        globals,
    }
}
