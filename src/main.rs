mod ast;
pub use ast::*;

mod parse;
pub use parse::*;

mod analysis;
pub use analysis::*;

mod layout;
pub use layout::*;

mod visit;
pub use visit::*;

mod comp;
pub use comp::*;

extern crate symbol_table;
pub type Symbol = symbol_table::GlobalSymbol;

use std::collections::{HashMap, HashSet};

fn main() {
    use std::path::*;

    let filename = std::env::args().nth(1).unwrap_or(String::from("mean"));
    let filename = filename.replace("examples/", "").replace(".gradu", "");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("examples").join(filename + ".gradu");
    let s = std::fs::read_to_string(path).unwrap();

    let ast = parse(&s);
    let actxt = analyze(&ast);
    let lctxt = layout(&actxt, &ast);
    comp::comp(&ast, &lctxt);
}
