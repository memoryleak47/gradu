use crate::*;

pub fn nil_init(ast: &mut AST, nameres: &Nameres) {
    for f in ast.fns.iter_mut() {
        f.body.push(Stmt::Return(Expr::NilLit));
    }
}
