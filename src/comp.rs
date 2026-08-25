use crate::*;

use std::process::Command;
use crate::ir::*;
use std::fmt::Write;

pub fn comp(ir: &IR) {
    let compiled = compile_ir(ir);

    let root = env!("CARGO_MANIFEST_DIR");
    let exe = &format!("{root}/exe");
    let exe_c = &format!("{root}/exe.c");

    std::fs::write(exe_c, compiled).unwrap();
    let co = Command::new("gcc").args([exe_c, "-o", exe, "-O3"]).output().unwrap().stderr;
    let co2 = String::from_utf8_lossy(&co);
    if !co2.is_empty() {
        println!("compiler error: {co2:?}");
    }

    let out = Command::new(exe).output().unwrap().stdout;
    let out2 = String::from_utf8_lossy(&out);
    println!("{out2}");
}

fn comp_expr(e: &Expr, ir: &IR, out: &mut String) {
    match e {
        Expr::StringLit(x) => write!(out, "\"{x}\"").unwrap(),
        Expr::TToValue(x, Ty::String) => write!(out, "str_to_value(v_{x})").unwrap(),
        x => todo!("{x:?}"),
    }
}

fn ty_str(x: &Ty) -> &str {
    match x {
        Ty::Value => "Value",
        Ty::String => "char*",
        x => todo!("{x:?}"),
    }
}

fn comp_stmt(stmt: &Stmt, ir: &IR, out: &mut String) {
    match stmt {
        Stmt::Print(x) => {
            writeln!(out, "    print_value(v_{x});").unwrap()
        },
        Stmt::Compute(v, e) => {
            write!(out, "    v_{v} = ").unwrap();
            comp_expr(e, ir, out);
            writeln!(out, ";").unwrap();
        },
        _ => todo!(),
    }
}

fn comp_terminator(terminator: &Terminator, ir: &IR, out: &mut String) {
    match terminator {
        Terminator::Exit => {
            writeln!(out, "    exit(0);").unwrap();
        },
        _ => todo!(),
    }
}

fn compile_ir(ir: &IR) -> String {
    let mut out = String::new();
    writeln!(&mut out, "#include \"preamble.h\"\n").unwrap();

    for (f, fdef) in &ir.fns {
        writeln!(&mut out, "void fn_{f}() {{").unwrap();

        let startblk = fdef.start;
        writeln!(&mut out, "  goto bb_{startblk};").unwrap();

        for (b, bdef) in &fdef.blocks {
            writeln!(&mut out, "  bb_{b}:").unwrap();
            for (var, ty) in &bdef.types {
                let ty = ty_str(ty);
                writeln!(&mut out, "    {ty} v_{var};").unwrap();
            }

            for st in &bdef.stmts {
                comp_stmt(st, ir, &mut out);
            }
            comp_terminator(&bdef.terminator, ir, &mut out);
        }

        writeln!(&mut out, "}}\n").unwrap();
    }

    let start = ir.start;
    write!(&mut out, "int main() {{ fn_{start}(); }}").unwrap();

    out
}
