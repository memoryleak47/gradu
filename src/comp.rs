use crate::*;
use std::process::Command;

pub fn comp(ast: &AST) {
    let compiled = compile_ast(ast);

    let root = env!("CARGO_MANIFEST_DIR");
    let exe = &format!("{root}/exe");
    let exe_c = &format!("{root}/exe.c");

    std::fs::write(exe_c, compiled).unwrap();
    let co = Command::new("gcc").args([exe_c, "-o", exe]).output().unwrap().stderr;
    let co2 = String::from_utf8_lossy(&co);
    if !co2.is_empty() {
        println!("compiler error: {co2:?}");
    }

    let out = Command::new(exe).output().unwrap().stdout;
    let out2 = String::from_utf8_lossy(&out);
    println!("{out2}");
}

fn compile_ast(ast: &AST) -> String {
    let tyctxt = ty_infer(ast);

    let mut compiled = String::from(include_str!("preamble.h"));
    for f in &ast.fns {
        compiled.push_str(&compile_fn(f, ast, &tyctxt));
    }

    compiled.push_str("int main() { fn_main(); return 0; }");

    compiled
}

fn stringify_layout(ty: &LayoutType) -> String {
    String::from(match ty {
        LayoutType::Value => "Value",
        LayoutType::Int => "int",
        LayoutType::Bool => "bool",
        LayoutType::Str => "char*",
        LayoutType::List(_) => "list*",
        LayoutType::Nil => panic!(),
    })
}

fn compile_fn(f: &FnDef, ast: &AST, tyctxt: &TyCtxt) -> String {
    let name = f.name;

    // retval
    let retval = tyctxt.get(&Location::RetVal(f.name)).unwrap().clone();
    let retval = stringify_layout(&retval);

    // args
    let mut args_s = String::new();
    for (i, arg) in f.args.iter().enumerate() {
        let l = Location::Var(f.name, *arg);
        let argty = get_ty(l, tyctxt);
        let argty = stringify_layout(&argty);
        args_s.push_str(&format!("{argty} {arg}"));
        if i != f.args.len() - 1 {
            args_s.push_str(", ");
        }
    }

    // local vars
    let mut varprefix = String::new();
    for (loc, varty) in tyctxt.iter() {
        if let Location::Var(ff, x) = loc && *ff == name && !f.args.contains(x) {
            let varty = stringify_layout(varty);
            varprefix.push_str(&format!("    {varty} {x};\n"));
        }
    }

    // body
    let body_s = comp_body(&f.body, name, ast, tyctxt, 0);

    format!("{retval} fn_{name}({args_s}) {{\n{varprefix}{body_s}}}\n\n")
}

fn comp_equ(e1: String, t1: LayoutType, e2: String, t2: LayoutType) -> String {
    if t1 == t2 && (t1 == LayoutType::Int || t1 == LayoutType::Bool) {
        return format!("({e1} == {e2})")
    }

    let e1 = type_cast_to(e1, t1, LayoutType::Value);
    let e2 = type_cast_to(e2, t2, LayoutType::Value);
    format!("is_equal({e1}, {e2})")
}

fn type_cast_to(e: String, old: LayoutType, new: LayoutType) -> String {
    if old == new {
        e
    } else if new == LayoutType::Value {
        match old {
            LayoutType::Bool => format!("bool_to_value({e})"),
            LayoutType::Int => format!("int_to_value({e})"),
            LayoutType::Str => format!("str_to_value({e})"),
            LayoutType::List(_) => format!("list_to_value({e})"),
            LayoutType::Nil => format!("nil_to_value({e})"),
            LayoutType::Value => unreachable!(),
        }
    } else if old == LayoutType::Value {
        match new {
            LayoutType::Bool => format!("value_to_bool({e})"),
            LayoutType::Int => format!("value_to_int({e})"),
            LayoutType::Str => format!("value_to_str({e})"),
            LayoutType::List(_) => format!("value_to_list({e})"),
            LayoutType::Nil => todo!(),
            LayoutType::Value => unreachable!(),
        }
    } else {
        dbg!(&old);
        dbg!(&new);
        panic!("This cast *has* to fail!")
    }
}

fn comp_typed_expr(e: &Expr, ty: LayoutType, fname: Symbol, ast: &AST, tyctxt: &TyCtxt) -> String {
    let (e, t) = comp_expr(e, fname, ast, tyctxt);
    type_cast_to(e, t, ty)
}

fn comp_expr(e: &Expr, fname: Symbol, ast: &AST, tyctxt: &TyCtxt) -> (String, LayoutType) {
    match e {
        Expr::NewList => {
            let ty = get_ty(Location::ListItem(e as *const Expr), tyctxt);
            (format!("new_list()"), LayoutType::List(Box::new(ty)))
        },
        Expr::Length(l) => {
            let l = comp_typed_expr(l, LayoutType::List(todo!()), fname, ast, tyctxt);
            (format!("length({l})"), LayoutType::Int)
        },
        Expr::IndexList(l, i) => {
            let l = comp_typed_expr(l, LayoutType::List(todo!()), fname, ast, tyctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fname, ast, tyctxt);
            (format!("index_list({l}, {i})"), LayoutType::Value)
        },
        Expr::FnCall(f, args) => {
            let ff = ast.fns.iter().find(|x| &x.name == f).unwrap();
            let mut args_str = String::new();
            for (i, (x, e)) in ff.args.iter().zip(args).enumerate() {
                let ty = get_ty(Location::Var(*f, *x), tyctxt);
                let e = comp_typed_expr(e, ty, fname, ast, tyctxt);
                args_str.push_str(&e);
                if i != ff.args.len() - 1 {
                    args_str.push_str(", ");
                }
            }

            let l = Location::RetVal(*f);
            let ty = get_ty(l, tyctxt);
            (format!("fn_{f}({args_str})"), ty)
        },
        Expr::BinOp(op, e1, e2) => {
            if let BinOpKind::Equ = op {
                let (e1, t1) = comp_expr(e1, fname, ast, tyctxt);
                let (e2, t2) = comp_expr(e2, fname, ast, tyctxt);
                return (comp_equ(e1, t1, e2, t2), LayoutType::Bool)
            }

            let e1 = comp_typed_expr(e1, LayoutType::Int, fname, ast, tyctxt);
            let e2 = comp_typed_expr(e2, LayoutType::Int, fname, ast, tyctxt);

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
        Expr::Var(v) => {
            let l = Location::Var(fname, *v);
            let ty = get_ty(l, tyctxt);
            (format!("{v}"), ty)
        },
        Expr::Input => {
            (format!("input()"), LayoutType::Value)
        },
    }
}

fn comp_stmt(stmt: &Stmt, fname: Symbol, ast: &AST, tyctxt: &TyCtxt, level: usize) -> String {
    let spaces = "    ".repeat(level+1);
    match stmt {
        Stmt::ListStore(l, i, v) => {
            let l = comp_typed_expr(l, LayoutType::List(todo!()), fname, ast, tyctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fname, ast, tyctxt);
            let v = comp_typed_expr(v, LayoutType::Value, fname, ast, tyctxt);
            format!("{spaces}store_list({l}, {i}, {v});\n")
        },
        Stmt::Push(l, v) => {
            let l = comp_typed_expr(l, LayoutType::List(todo!()), fname, ast, tyctxt);
            let v = comp_typed_expr(v, LayoutType::Value, fname, ast, tyctxt);
            format!("{spaces}push_list({l}, {v});\n")
        },
        Stmt::Assign(v, e) => {
            let loc = Location::Var(fname, *v);
            let ty = get_ty(loc, tyctxt);
            let e = comp_typed_expr(e, ty, fname, ast, tyctxt);
            format!("{spaces}{v} = {e};\n")
        },
        Stmt::Return(e) => {
            let loc = Location::RetVal(fname);
            let ty = get_ty(loc, tyctxt);
            let e = comp_typed_expr(e, ty, fname, ast, tyctxt);
            format!("{spaces}return {e};\n")
        },
        Stmt::If(cond, then_, else_) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fname, ast, tyctxt);
            format!("{spaces}if ({}) {{\n{}{spaces}}} else {{\n{}{spaces}}}\n", cond, comp_body(then_, fname, ast, tyctxt, level+1), comp_body(else_, fname, ast, tyctxt, level+1))
        },
        Stmt::While(cond, body) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fname, ast, tyctxt);
            format!("{spaces}while ({}) {{\n{}{spaces}}}\n", cond, comp_body(body, fname, ast, tyctxt, level+1))
        },
        Stmt::Print(e) => {
            let e = comp_typed_expr(e, LayoutType::Value, fname, ast, tyctxt);
            format!("{spaces}print_value({e});\n")
        },
    }
}

fn get_ty(loc: Location, tyctxt: &TyCtxt) -> LayoutType {
    tyctxt.get(&loc).cloned().unwrap_or(LayoutType::Value)
}

fn comp_body(body: &Body, fname: Symbol, ast: &AST, tyctxt: &TyCtxt, level: usize) -> String {
    let mut out = String::new();
    for stmt in body {
        out.push_str(&comp_stmt(stmt, fname, ast, tyctxt, level));
    }
    out
}
