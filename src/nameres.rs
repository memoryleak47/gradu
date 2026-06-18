use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VarKind {
    Global,
    Local
}

#[derive(Debug)]
pub struct Nameres {
    pub vars: Vec<HashMap<Symbol, VarKind>>, // indexed by FnId and Varname
    pub globals: HashSet<Symbol>,
}

pub fn nameres(ast: &AST) -> Nameres {
    let mut vars = Vec::new();
    let mut globals = HashSet::new();

    for fid in 0..ast.fns.len() {
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
                    VarKind::Global
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

    // every global has to be part of the main_fn.
    let main_vars = &mut vars[ast.main_fn];
    for &g in &globals {
        main_vars.insert(g, VarKind::Global);
    }

    Nameres {
        vars,
        globals,
    }
}
