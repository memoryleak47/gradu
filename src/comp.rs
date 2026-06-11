use crate::*;
use std::process::Command;

pub fn comp(ast: &AST) {
    let compiled = compile_ast(ast);

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

fn compile_ast(ast: &AST) -> String {
    let tyctxt = ty_infer(ast);

    let mut compiled = String::from(include_str!("preamble.h"));

    // lists
    let item_ty = get_ty(Location::ListItem, &tyctxt);
    let item_ty = stringify_layout(&item_ty);
    let list_h = include_str!("list.h").replace("T", &item_ty);
    compiled.push_str(&list_h);

    // globals
    compiled.push_str("\n");
    for (loc, layout) in &tyctxt {
        if let Location::GlobalVar(v) = loc {
            compiled.push_str(&format!("{};\n", make_decl(layout, *v)));
        }
    }
    compiled.push_str("\n");

    // compile each fn
    for fid in 0..ast.fns.len() {
        compiled.push_str(&compile_fn(fid, ast, &tyctxt));
    }

    // entry point
    compiled.push_str(&format!("int main() {{ fn_{}(); return 0; }}", ast.main_fn));

    compiled
}

fn stringify_layout(ty: &LayoutType) -> String {
    String::from(match ty {
        LayoutType::Value => "Value",
        LayoutType::Int => "int",
        LayoutType::Bool => "bool",
        LayoutType::Str => "char*",
        LayoutType::List => "list*",
        LayoutType::Fn(args, ret) => {
            let ret = stringify_layout(ret);
            let mut s = format!("{ret} (*)(");
            for (i, a) in args.iter().enumerate() {
                s.push_str(&stringify_layout(a));
                if i != args.len()-1 {
                    s.push_str(", ");
                }
            }
            s.push(')');
            return s
        },
        LayoutType::Nil => panic!(),
    })
}

fn compile_fn(fid: FnId, ast: &AST, tyctxt: &TyCtxt) -> String {
    let f = &ast.fns[fid];

    // retval
    let retval = tyctxt.get(&Location::RetVal(fid)).unwrap().clone();
    let retval = stringify_layout(&retval);

    // args
    let mut args_s = String::new();
    for (i, arg) in f.args.iter().enumerate() {
        let l = Location::Var(fid, *arg);
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
        if let Location::Var(ff, x) = loc && *ff == fid && !f.args.contains(x) && !is_global_var(fid, *x, ast) {
            let varty = stringify_layout(varty);
            varprefix.push_str(&format!("    {varty} {x};\n"));
        }
    }

    // body
    let body_s = comp_body(&f.body, fid, ast, tyctxt, 0);

    format!("{retval} fn_{fid}({args_s}) {{\n{varprefix}{body_s}}}\n\n")
}

fn make_decl(ty: &LayoutType, v: Symbol) -> String {
    if let LayoutType::Fn(args, ret) = ty {
        let ret = stringify_layout(ret);
        let mut s = format!("{ret} (*{v})(");
        for (i, a) in args.iter().enumerate() {
            s.push_str(&stringify_layout(a));
            if i != args.len()-1 {
                s.push_str(", ");
            }
        }
        s.push(')');
        s
    } else {
        let ty = stringify_layout(ty);
        format!("{ty} {v}")
    }
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
            LayoutType::List => format!("list_to_value({e})"),
            LayoutType::Nil => format!("nil_to_value({e})"),
            LayoutType::Fn(..) => format!("fn_to_value({e})"),
            LayoutType::Value => unreachable!(),
        }
    } else if old == LayoutType::Value {
        match new {
            LayoutType::Bool => format!("value_to_bool({e})"),
            LayoutType::Int => format!("value_to_int({e})"),
            LayoutType::Str => format!("value_to_str({e})"),
            LayoutType::List => format!("value_to_list({e})"),
            LayoutType::Nil => todo!(),
            LayoutType::Fn(..) => todo!(),
            LayoutType::Value => unreachable!(),
        }
    } else if let LayoutType::Fn(..) = &old && let LayoutType::Fn(..) = &new {
        let new_str = stringify_layout(&new);
        format!("(({new_str}) ({e}))")
    } else {
        dbg!(&old);
        dbg!(&new);
        panic!("This cast *has* to fail!")
    }
}

fn comp_typed_expr(e: &Expr, ty: LayoutType, fid: FnId, ast: &AST, tyctxt: &TyCtxt) -> String {
    let (e, t) = comp_expr(e, fid, ast, tyctxt);
    type_cast_to(e, t, ty)
}

fn comp_expr(e: &Expr, fid: FnId, ast: &AST, tyctxt: &TyCtxt) -> (String, LayoutType) {
    match e {
        Expr::FnId(ffid) => {
            let retty = get_ty(Location::RetVal(*ffid), tyctxt);
            let mut args = Vec::new();
            for &a in &ast.fns[*ffid].args {
                args.push(get_ty(Location::Var(*ffid, a), tyctxt));
            }
            (format!("fn_{ffid}"), LayoutType::Fn(args, Box::new(retty)))
        },
        Expr::NewList => {
            (format!("new_list()"), LayoutType::List)
        },
        Expr::Length(l) => {
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, tyctxt);
            (format!("length({l})"), LayoutType::Int)
        },
        Expr::IndexList(l, i) => {
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, tyctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fid, ast, tyctxt);

            let ty = get_ty(Location::ListItem, tyctxt);

            (format!("index_list({l}, {i})"), ty)
        },
        Expr::FnCall(f, args) => {
            let (f, lty) = comp_expr(f, fid, ast, tyctxt);

            // TODO this might not be know at compile-time!
            let LayoutType::Fn(argtys, retty) = &lty else { panic!() };

            let mut args_str = String::new();
            for (i, (t, e)) in argtys.iter().zip(args).enumerate() {
                let e = comp_typed_expr(e, t.clone(), fid, ast, tyctxt);
                args_str.push_str(&e);
                if i != args.len() - 1 {
                    args_str.push_str(", ");
                }
            }

            let lty_str = stringify_layout(&lty);
            let fstr = format!("(({lty_str}) {f})");
            (format!("{fstr}({args_str})"), (**retty).clone())
        },
        Expr::BinOp(op, e1, e2) => {
            if let BinOpKind::Equ|BinOpKind::Ne = op {
                let (e1, t1) = comp_expr(e1, fid, ast, tyctxt);
                let (e2, t2) = comp_expr(e2, fid, ast, tyctxt);
                let mut out = comp_equ(e1, t1, e2, t2);
                if let BinOpKind::Ne = op {
                    out = format!("(!{out})");
                }
                return (out, LayoutType::Bool)
            }

            let e1 = comp_typed_expr(e1, LayoutType::Int, fid, ast, tyctxt);
            let e2 = comp_typed_expr(e2, LayoutType::Int, fid, ast, tyctxt);

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
                BinOpKind::Ne => unreachable!(),
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
            let l = get_var_loc(fid, *v, ast);
            let ty = get_ty(l, tyctxt);
            (format!("{v}"), ty)
        },
        Expr::Input => {
            (format!("input()"), LayoutType::Value)
        },
    }
}

fn comp_stmt(stmt: &Stmt, fid: FnId, ast: &AST, tyctxt: &TyCtxt, level: usize) -> String {
    let spaces = "    ".repeat(level+1);
    match stmt {
        Stmt::Global(_) => String::new(),
        Stmt::ListStore(l, i, v) => {
            let ty = get_ty(Location::ListItem, tyctxt);
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, tyctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fid, ast, tyctxt);
            let v = comp_typed_expr(v, ty, fid, ast, tyctxt);
            format!("{spaces}store_list({l}, {i}, {v});\n")
        },
        Stmt::Push(l, v) => {
            let ty = get_ty(Location::ListItem, tyctxt);
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, tyctxt);
            let v = comp_typed_expr(v, ty, fid, ast, tyctxt);
            format!("{spaces}push_list({l}, {v});\n")
        },
        Stmt::Assign(v, e) => {
            let loc = get_var_loc(fid, *v, ast);
            let ty = get_ty(loc, tyctxt);
            let e = comp_typed_expr(e, ty, fid, ast, tyctxt);
            format!("{spaces}{v} = {e};\n")
        },
        Stmt::Return(e) => {
            let loc = Location::RetVal(fid);
            let ty = get_ty(loc, tyctxt);
            let e = comp_typed_expr(e, ty, fid, ast, tyctxt);
            format!("{spaces}return {e};\n")
        },
        Stmt::If(cond, then_, else_) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fid, ast, tyctxt);
            format!("{spaces}if ({}) {{\n{}{spaces}}} else {{\n{}{spaces}}}\n", cond, comp_body(then_, fid, ast, tyctxt, level+1), comp_body(else_, fid, ast, tyctxt, level+1))
        },
        Stmt::While(cond, body) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fid, ast, tyctxt);
            format!("{spaces}while ({}) {{\n{}{spaces}}}\n", cond, comp_body(body, fid, ast, tyctxt, level+1))
        },
        Stmt::Print(e) => {
            let e = comp_typed_expr(e, LayoutType::Value, fid, ast, tyctxt);
            format!("{spaces}print_value({e});\n")
        },
    }
}

fn get_ty(loc: Location, tyctxt: &TyCtxt) -> LayoutType {
    tyctxt.get(&loc).cloned().unwrap_or(LayoutType::Value)
}

fn comp_body(body: &Body, fid: FnId, ast: &AST, tyctxt: &TyCtxt, level: usize) -> String {
    let mut out = String::new();
    for stmt in body {
        out.push_str(&comp_stmt(stmt, fid, ast, tyctxt, level));
    }
    out
}
