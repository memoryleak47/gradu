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
        Expr::IntLit(x) => write!(out, "{x}").unwrap(),
        Expr::BoolLit(b) => write!(out, "{b}").unwrap(),
        Expr::BinOp(BinOpKind::Plus, x, y) => write!(out, "v_{x} + v_{y}").unwrap(),
        Expr::BinOp(BinOpKind::Minus, x, y) => write!(out, "v_{x} - v_{y}").unwrap(),
        Expr::BinOp(BinOpKind::Lt, x, y) => write!(out, "v_{x} < v_{y}").unwrap(),
        Expr::BinOp(BinOpKind::Gt, x, y) => write!(out, "v_{x} > v_{y}").unwrap(),
        Expr::BinOp(BinOpKind::Mod, x, y) => write!(out, "v_{x} % v_{y}").unwrap(),
        Expr::BinOp(BinOpKind::Equ, x, y) => write!(out, "is_equal(v_{x}, v_{y})").unwrap(),
        Expr::BinOp(BinOpKind::Ne, x, y) => write!(out, "!is_equal(v_{x}, v_{y})").unwrap(),

        Expr::TToValue(x, Ty::String) => write!(out, "str_to_value(v_{x})").unwrap(),
        Expr::TToValue(x, Ty::Int) => write!(out, "int_to_value(v_{x})").unwrap(),
        Expr::TToValue(x, Ty::Bool) => write!(out, "bool_to_value(v_{x})").unwrap(),

        Expr::ValueToT(x, Ty::String) => write!(out, "value_to_str(v_{x})").unwrap(),
        Expr::ValueToT(x, Ty::Int) => write!(out, "value_to_int(v_{x})").unwrap(),
        Expr::ValueToT(x, Ty::Bool) => write!(out, "value_to_bool(v_{x})").unwrap(),
        x => todo!("{x:?}"),
    }
}

fn ty_str(x: &Ty) -> &str {
    match x {
        Ty::Value => "Value",
        Ty::String => "char*",
        Ty::Int => "int",
        Ty::Bool => "bool",
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
        Terminator::Exit => writeln!(out, "    exit(0);").unwrap(),
        Terminator::Goto(x) => writeln!(out, "    goto bb_{};", x.0).unwrap(),
        Terminator::IfGoto(cond, then_, else_) => writeln!(out, "    if (v_{cond}) goto bb_{}; else goto bb_{};", then_.0, else_.0).unwrap(),
        x => todo!("{x:?}"),
    }
}

fn compile_ir(ir: &IR) -> String {
    let mut out = String::new();
    writeln!(&mut out, "#include \"preamble.h\"\n").unwrap();

    for (f, fdef) in &ir.fns {
        writeln!(&mut out, "void fn_{f}() {{").unwrap();

        let startblk = fdef.start;

        // forward declarations:
        for (b, bdef) in &fdef.blocks {
            for (var, ty) in &bdef.types {
                let ty = ty_str(ty);
                writeln!(&mut out, "  {ty} v_{var};").unwrap();
            }
        }

        writeln!(&mut out, "  goto bb_{startblk};").unwrap();

        // stmts of the block.
        for (b, bdef) in &fdef.blocks {
            writeln!(&mut out, "  bb_{b}:").unwrap();
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
