mod ast;
pub use ast::AST;

mod parse;
pub use parse::*;

mod ir;
pub use ir::IR;

mod lower;
pub use lower::*;

extern crate symbol_table;
pub type Symbol = symbol_table::GlobalSymbol;

use std::collections::{HashMap, HashSet};

fn main() {
    use std::path::*;

    let filename = std::env::args().nth(1).unwrap_or(String::from("dict"));
    let filename = filename.replace("examples/", "").replace(".gradu", "");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples").join(filename + ".gradu");
    let s = std::fs::read_to_string(path).unwrap();

    let ast = parse(&s);
    let ir = lower(ast);
    dbg!(ir);
}
