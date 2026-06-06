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
    use std::path::*;

    let filename = std::env::args().nth(1).unwrap_or(String::from("factorial.gradu"));
    let filename = filename.replace("examples/", "").replace(".gradu", "");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples").join(filename + ".gradu");
    let s = std::fs::read_to_string(path).unwrap();

    let ast = parse(&s);
    comp::comp(&ast);
}
