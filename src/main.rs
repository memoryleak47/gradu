mod ast;
pub use ast::*;

mod interp;
pub use interp::*;

mod comp;
pub use comp::*;

mod parse;
pub use parse::*;

fn main() {
    let s = include_str!("../examples/isprime.gradu");
    let ast = parse(&s);
    interp::interp(&ast);

    comp::comp(&ast);
}
