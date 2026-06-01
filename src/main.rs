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

    let compiled = comp::comp(&ast);
    std::fs::write("gen.c", compiled).unwrap();
    let co = Command::new("gcc").arg("gen.c").arg("-o").arg("gen").output().unwrap().stderr;
    let co2 = String::from_utf8_lossy(&co);
    if !co2.is_empty() {
        println!("compiler error: {co2:?}");
    }

    let out = Command::new("./gen").output().unwrap().stdout;
    let out2 = String::from_utf8_lossy(&out);
    println!("{out2}");
}
