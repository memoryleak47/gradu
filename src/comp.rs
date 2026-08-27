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

fn comp_expr(e: &Expr, out: &mut String) {
    match e {
        Expr::StringLit(x) => write!(out, "\"{x}\"").unwrap(),
        Expr::IntLit(x) => write!(out, "{x}").unwrap(),
        Expr::BoolLit(b) => write!(out, "{b}").unwrap(),
        Expr::NilLit => write!(out, "(nil) {{}}").unwrap(),
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
        Expr::TToValue(x, Ty::Nil) => write!(out, "nil_to_value(v_{x})").unwrap(),

        Expr::ValueToT(x, Ty::String) => write!(out, "value_to_str(v_{x})").unwrap(),
        Expr::ValueToT(x, Ty::Int) => write!(out, "value_to_int(v_{x})").unwrap(),
        Expr::ValueToT(x, Ty::Bool) => write!(out, "value_to_bool(v_{x})").unwrap(),
        Expr::ValueToT(x, Ty::Nil) => write!(out, "value_to_nil(v_{x})").unwrap(),

        x => todo!("{x:?}"),
    }
}

fn ty_str(x: &Ty) -> &str {
    match x {
        Ty::Value => "Value",
        Ty::String => "char*",
        Ty::Int => "int",
        Ty::Bool => "bool",
        Ty::Nil => "nil",
        Ty::Fn => "void*",
    }
}

fn comp_stmt(stmt: &Stmt, ir: &IR, out: &mut String) {
    match stmt {
        Stmt::Print(x) => {
            writeln!(out, "    print_value(v_{x});").unwrap()
        },
        Stmt::Compute(v, e) => {
            write!(out, "    v_{v} = ").unwrap();
            comp_expr(e, out);
            writeln!(out, ";").unwrap();
        },
        _ => todo!(),
    }
}

fn comp_goto((bid, args): &AppliedBlk, f: FnId, ir: &IR, out: &mut String) {
    let args2 = &ir.fns[&f].blocks[&bid].args;
    for (x, y) in args.iter().zip(args2.iter()) {
        write!(out, "v_{y} = v_{x}; ").unwrap();
    }
    write!(out, "goto bb_{bid};").unwrap();
}

fn comp_terminator(terminator: &Terminator, f: FnId, ir: &IR, out: &mut String) {
    match terminator {
        Terminator::Exit => writeln!(out, "    exit(0);").unwrap(),
        Terminator::Goto(x) => {
            write!(out, "    ").unwrap();
            comp_goto(&x, f, ir, out);
            writeln!(out, "").unwrap();
        },
        Terminator::IfGoto(cond, then_, else_) => {
            write!(out, "    if (v_{cond}) {{ ").unwrap();
            comp_goto(&then_, f, ir, out);
            write!(out, " }} else {{ ").unwrap();
            comp_goto(&else_, f, ir, out);
            writeln!(out, " }}").unwrap();
        },
        Terminator::Return(v) => writeln!(out, "    return v_{v};").unwrap(),
    }
}

fn compile_ir(ir: &IR) -> String {
    let mut out = String::new();
    writeln!(&mut out, "#include \"preamble.h\"\n").unwrap();

    for (f, fdef) in &ir.fns {
        let retty = ty_str(&fdef.retty);
        writeln!(&mut out, "{retty} fn_{f}() {{").unwrap();

        let startblk = fdef.start;

        // forward declarations:
        for (_, bdef) in &fdef.blocks {
            for (var, ty) in &bdef.types {
                let ty = ty_str(ty);
                writeln!(&mut out, "  {ty} v_{var};").unwrap();
            }
        }

        writeln!(&mut out, "  goto bb_{startblk};").unwrap();

        // stmts of the block.
        for (b, bdef) in &fdef.blocks {
            write!(&mut out, "  bb_{b}: // ").unwrap();
            for (i, a) in bdef.args.iter().enumerate() {
                write!(&mut out, "v_{a}").unwrap();
                if i != bdef.args.len()-1 {
                    write!(&mut out, ", ").unwrap();
                }
            }
            write!(&mut out, "\n").unwrap();
            for st in &bdef.stmts {
                comp_stmt(st, ir, &mut out);
            }
            comp_terminator(&bdef.terminator, *f, ir, &mut out);
        }


        writeln!(&mut out, "}}\n").unwrap();
    }

    let start = ir.start;
    write!(&mut out, "int main() {{ fn_{start}(); }}").unwrap();

    out
}
