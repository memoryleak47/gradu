mod ast;
pub use ast::*;

mod interp;
pub use interp::*;

// mod comp;
// pub use comp::*;

mod parse;
pub use parse::*;

// mod ty;
// pub use ty::*;

use std::collections::HashMap;

fn main() {
    let s = include_str!("../examples/factorial.gradu");
    let ast = parse(&s);

    interp::interp(&ast);
    // comp::comp(&ast.body);
}
