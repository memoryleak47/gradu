mod ast;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub grammar);

fn main() {
    let s = std::fs::read_to_string("examples/a.gradu").unwrap();
    let x = grammar::ASTParser::new().parse(&s);
    dbg!(x);
}
