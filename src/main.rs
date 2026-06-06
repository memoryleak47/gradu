mod ast;
pub use ast::*;

mod comp;
pub use comp::*;

mod parse;
pub use parse::*;

mod ty;
pub use ty::*;

extern crate symbol_table;
pub type Symbol = symbol_table::GlobalSymbol;

use std::collections::HashMap;

fn main() {
    let s = include_str!("../examples/factorial.gradu");
    let ast = parse(&s);

    comp::comp(&ast);
}
