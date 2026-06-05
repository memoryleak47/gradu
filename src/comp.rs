use crate::*;
use std::process::Command;

pub fn comp(ast: &AST) {
    let compiled = compile_ast(ast);

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

fn compile_ast(ast: &AST) -> String {
    let tyctxt = ty_infer(ast);

    let mut compiled = String::from(include_str!("preamble.h"));
    for f in &ast.fns {
        compiled.push_str(&compile_fn(f, &tyctxt));
    }

    compiled.push_str("int main() { fn_main(); return 0; }");

    compiled
}

fn compile_fn(f: &FnDef, tyctxt: &TyCtxt) -> String {
    let name = &f.name;
    let mut varprefix = String::new();
    for (x, ty) in tyctxt.iter() {
        if !f.args.contains(&x) {
            let ty = match ty {
                LayoutType::Value => "Value",
                LayoutType::Int => "int",
                LayoutType::Bool => "bool",
                LayoutType::Str => "char*",
                LayoutType::Nil => panic!(),
            };
            varprefix.push_str(&format!("    {ty} {x};\n"));
        }
    }

    let body_s = comp_body(&f.body, tyctxt, 0);
    let mut args_s = String::new();

    for (i, arg) in f.args.iter().enumerate() {
        args_s.push_str(&format!("Value {arg}"));
        if i != f.args.len() - 1 {
            args_s.push_str(", ");
        }
    }

    format!("Value fn_{name}({args_s}) {{\n{varprefix}\n{body_s}}}\n\n")
}

// always produces "Value" type
fn comp_expr(e: &Expr, tyctxt: &TyCtxt) -> String {
    let (s, ty) = comp_expr_raw(e, tyctxt);
    type_cast_to_value(s, ty)
}

fn type_cast_to_value(e: String, old: LayoutType) -> String {
    match old {
        LayoutType::Bool => format!("mk_bool({e})"),
        LayoutType::Nil => todo!(),
        LayoutType::Str => format!("mk_str({e})"),
        LayoutType::Int => format!("mk_int({e})"),
        LayoutType::Value => e,
    }
}

fn type_cast_to_int(e: String, old: LayoutType) -> String {
    match old {
        LayoutType::Int => e,
        LayoutType::Value => format!("to_int({e})"),
        _ => panic!(),
    }
}

fn type_cast_to_bool(e: String, old: LayoutType) -> String {
    match old {
        LayoutType::Bool => e,
        LayoutType::Value => format!("to_bool({e})"),
        _ => panic!(),
    }
}

fn comp_equ(e1: String, t1: LayoutType, e2: String, t2: LayoutType) -> String {
    if t1 == t2 && (t1 == LayoutType::Int || t1 == LayoutType::Bool) {
        return format!("({e1} == {e2})")
    }

    let e1 = type_cast_to_value(e1, t1);
    let e2 = type_cast_to_value(e2, t2);
    format!("is_equal({e1}, {e2})")
}

fn comp_expr_raw(e: &Expr, tyctxt: &TyCtxt) -> (String, LayoutType) {
    match e {
        Expr::FnCall(f, args) => {
            let args = args.iter().map(|x| comp_expr(x, tyctxt)).collect::<Vec<_>>();
            let args = args.join(", ");
            (format!("fn_{f}({args})"), LayoutType::Value)
        },
        Expr::BinOp(op, e1, e2) => {
            let (e1, t1) = comp_expr_raw(e1, tyctxt);
            let (e2, t2) = comp_expr_raw(e2, tyctxt);
            if let BinOpKind::Equ = op {
                return (comp_equ(e1, t1, e2, t2), LayoutType::Bool)
            }
            let e1 = type_cast_to_int(e1, t1);
            let e2 = type_cast_to_int(e2, t2);
            match op {
                BinOpKind::Lt => {
                    (format!("({e1} < {e2})"), LayoutType::Bool)
                },
                BinOpKind::Gt => {
                    (format!("({e1} > {e2})"), LayoutType::Bool)
                },
                BinOpKind::Mod => {
                    (format!("({e1} % {e2})"), LayoutType::Int)
                },
                BinOpKind::Plus => {
                    (format!("({e1} + {e2})"), LayoutType::Int)
                },
                BinOpKind::Mul => {
                    (format!("({e1} * {e2})"), LayoutType::Int)
                },
                BinOpKind::Minus => {
                    (format!("({e1} - {e2})"), LayoutType::Int)
                },
                BinOpKind::Equ => unreachable!(),
            }
        },
        Expr::IntLit(i) => {
            (format!("{i}"), LayoutType::Int)
        },
        Expr::StringLit(s) => {
            (format!("\"{s}\""), LayoutType::Str)
        },
        Expr::BoolLit(b) => {
            if *b {
                (format!("true"), LayoutType::Bool)
            } else {
                (format!("false"), LayoutType::Bool)
            }
        },
        Expr::Var(v) => (format!("{v}"), tyctxt[v]),
        Expr::Input => {
            (format!("input()"), LayoutType::Value)
        },
    }
}

fn comp_stmt(stmt: &Stmt, tyctxt: &TyCtxt, level: usize) -> String {
    let spaces = "    ".repeat(level+1);
    match stmt {
        Stmt::Assign(v, e) => {
            let (mut e, t) = comp_expr_raw(e, tyctxt);
            if t != tyctxt[&*v] {
                e = type_cast_to_value(e, t);
            }
            format!("{spaces}{v} = {e};\n")
        },
        Stmt::Return(e) => {
            let e = comp_expr(e, tyctxt);
            format!("{spaces}return {e};\n")
        },
        Stmt::If(cond, then_, else_) => {
            let (cond, tcond) = comp_expr_raw(cond, tyctxt);
            let cond = type_cast_to_bool(cond, tcond);
            format!("{spaces}if ({}) {{\n{}{spaces}}} else {{\n{}{spaces}}}\n", cond, comp_body(then_, tyctxt, level+1), comp_body(else_, tyctxt, level+1))
        },
        Stmt::While(cond, body) => {
            let (cond, tcond) = comp_expr_raw(cond, tyctxt);
            let cond = type_cast_to_bool(cond, tcond);
            format!("{spaces}while ({}) {{\n{}{spaces}}}\n", cond, comp_body(body, tyctxt, level+1))
        },
        Stmt::Print(e) => {
            let e = comp_expr(e, tyctxt);
            format!("{spaces}print_value({e});\n")
        },
    }
}


fn comp_body(body: &Body, tyctxt: &TyCtxt, level: usize) -> String {
    let mut out = String::new();
    for stmt in body {
        out.push_str(&comp_stmt(stmt, tyctxt, level));
    }
    out
}
