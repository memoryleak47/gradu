pub mod ast;
pub mod interp;
pub mod comp;

use std::process::Command;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

fn main() {
    let s = std::fs::read_to_string("examples/a.gradu").unwrap();
    let ast = grammar::ASTParser::new().parse(&s).unwrap();
    interp::interp(&ast);
    let o = comp::comp(&ast);
    std::fs::write("gen.c", o).unwrap();
    Command::new("gcc").arg("gen.c").arg("-o").arg("gen").output().expect("");
}
