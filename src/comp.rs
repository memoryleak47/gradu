use crate::*;
use std::process::Command;

pub fn comp(ast: &AST, nameres: &Nameres, lctxt: &LCtxt) {
    let compiled = compile_ast(ast, nameres, lctxt);

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

fn compile_ast(ast: &AST, nameres: &Nameres, lctxt: &LCtxt) -> String {
    let mut compiled = String::from(include_str!("preamble.h"));

    // lists
    let item_ty = get_ty(LayoutLocation::ListItem, lctxt);
    let item_ty = stringify_layout(&item_ty);
    let list_h = include_str!("list.h").replace("$T", &item_ty);
    compiled.push_str(&list_h);

    // dicts
    let kty = get_ty(LayoutLocation::DictKey, lctxt);
    let vty = get_ty(LayoutLocation::DictValue, lctxt);
    let k_equ_expr = comp_equ(String::from("k1"), kty, String::from("k2"), kty);
    let kty = stringify_layout(&kty);
    let vty = stringify_layout(&vty);
    let k_equ = format!("bool k_equ({kty} k1, {kty} k2) {{\n    return {k_equ_expr};\n}}\n");
    compiled.push_str(&k_equ);
    let dict_h = include_str!("dict.h").replace("$K", &kty).replace("$V", &vty);
    compiled.push_str(&dict_h);

    // globals
    compiled.push_str("\n");
    for &v in &nameres.globals {
        let layout = get_ty(LayoutLocation::GlobalVar(v), lctxt);
        compiled.push_str(&format!("{};\n", make_decl(&layout, v)));
    }
    compiled.push_str("\n");

    // compile each fn
    for fid in 0..ast.fns.len() {
        compiled.push_str(&compile_fn(fid, ast, nameres, lctxt));
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
        LayoutType::Dict => "dict*",
        LayoutType::Fn(_) => "void*",
        LayoutType::Nil => panic!(),
    })
}

fn compile_fn(fid: FnId, ast: &AST, nameres: &Nameres, lctxt: &LCtxt) -> String {
    let f = &ast.fns[fid];

    let tag = lctxt.fn_to_tag[&fid];

    // retval
    let retval = get_ty(LayoutLocation::RetVal(tag), lctxt);
    let retval = stringify_layout(&retval);

    // args
    let mut args_s = String::new();
    for (i, arg) in f.args.iter().enumerate() {
        let argty = get_ty(LayoutLocation::Arg(tag, i), lctxt);
        args_s.push_str(&make_decl(&argty, *arg));
        if i != f.args.len() - 1 {
            args_s.push_str(", ");
        }
    }

    // local vars
    let mut varprefix = String::new();
    for (v, kind) in &nameres.vars[fid] {
        let VarKind::Local = kind else { continue };
        if ast.fns[fid].args.contains(&v) { continue }
        let varty = get_ty(LayoutLocation::Var(fid, *v), lctxt);
        varprefix.push_str(&format!("    {};\n", make_decl(&varty, *v)));
    }

    // body
    let body_s = comp_body(&f.body, fid, ast, nameres, lctxt, 0);

    format!("{retval} fn_{fid}({args_s}) {{\n{varprefix}{body_s}}}\n\n")
}

fn make_decl(ty: &LayoutType, v: Symbol) -> String {
    let ty = stringify_layout(ty);
    format!("{ty} {v}")
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
            LayoutType::Dict => format!("dict_to_value({e})"),
            LayoutType::Nil => format!("nil_to_value({e})"),
            LayoutType::Fn(tag) => format!("tagged_fn_to_value({e}, {tag})"),
            LayoutType::Value => unreachable!(),
        }
    } else if old == LayoutType::Value {
        match new {
            LayoutType::Bool => format!("value_to_bool({e})"),
            LayoutType::Int => format!("value_to_int({e})"),
            LayoutType::Str => format!("value_to_str({e})"),
            LayoutType::List => format!("value_to_list({e})"),
            LayoutType::Dict => format!("value_to_dict({e})"),
            LayoutType::Nil => todo!(),
            LayoutType::Fn(tag) => format!("value_to_fn_with_tag({e}, {tag})"),
            LayoutType::Value => unreachable!(),
        }
    } else {
        // This cast will always fail at runtime.
        let e = type_cast_to(e, old, LayoutType::Value);
        let e = type_cast_to(e, LayoutType::Value, new);
        e
    }
}

fn comp_typed_expr(e: &Expr, ty: LayoutType, fid: FnId, ast: &AST, nameres: &Nameres, lctxt: &LCtxt) -> String {
    let (e, t) = comp_expr(e, fid, ast, nameres, lctxt);
    type_cast_to(e, t, ty)
}

fn comp_expr(e: &Expr, fid: FnId, ast: &AST, nameres: &Nameres, lctxt: &LCtxt) -> (String, LayoutType) {
    match e {
        Expr::FnId(ffid) => {
            (format!("fn_{ffid}"), LayoutType::Fn(lctxt.fn_to_tag[&ffid]))
        },
        Expr::NewList => {
            (format!("new_list()"), LayoutType::List)
        },
        Expr::NewDict => {
            (format!("new_dict()"), LayoutType::Dict)
        },
        Expr::Length(l) => {
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, nameres, lctxt);
            (format!("length({l})"), LayoutType::Int)
        },
        Expr::IndexList(l, i) => {
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, nameres, lctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fid, ast, nameres, lctxt);

            let ty = get_ty(LayoutLocation::ListItem, lctxt);

            (format!("index_list({l}, {i})"), ty)
        },
        Expr::IndexDict(t, k) => {
            let t = comp_typed_expr(t, LayoutType::Dict, fid, ast, nameres, lctxt);

            let kty = get_ty(LayoutLocation::DictKey, lctxt);
            let k = comp_typed_expr(k, kty, fid, ast, nameres, lctxt);

            let ty = get_ty(LayoutLocation::DictValue, lctxt);

            (format!("index_dict({t}, {k})"), ty)
        },
        Expr::FnCall(f, args) => {
            let Some(&tag) = lctxt.call_to_tag.get(&(e as *const Expr)) else {
                // This is only None, if no function can ever be called in this expr.
                return (format!("fail(\"runtime error: wrong arity fn call.\")"), LayoutType::Value)
            };

            let f = comp_typed_expr(f, LayoutType::Fn(tag), fid, ast, nameres, lctxt);

            let retty = get_ty(LayoutLocation::RetVal(tag), lctxt);

            let mut ty_str = format!("{} (*)(", stringify_layout(&retty));
            let mut args_str = String::new();

            for (i, e) in args.iter().enumerate() {
                let t = get_ty(LayoutLocation::Arg(tag, i), lctxt);

                ty_str.push_str(&stringify_layout(&t));

                let e = comp_typed_expr(e, t, fid, ast, nameres, lctxt);
                args_str.push_str(&e);

                if i != args.len() - 1 {
                    ty_str.push_str(", ");
                    args_str.push_str(", ");
                }
            }
            ty_str.push(')');

            let fstr = format!("(({ty_str}) {f})");
            (format!("{fstr}({args_str})"), retty.clone())
        },
        Expr::BinOp(op, e1, e2) => {
            if let BinOpKind::Equ|BinOpKind::Ne = op {
                let (e1, t1) = comp_expr(e1, fid, ast, nameres, lctxt);
                let (e2, t2) = comp_expr(e2, fid, ast, nameres, lctxt);
                let mut out = comp_equ(e1, t1, e2, t2);
                if let BinOpKind::Ne = op {
                    out = format!("(!{out})");
                }
                return (out, LayoutType::Bool)
            }

            let e1 = comp_typed_expr(e1, LayoutType::Int, fid, ast, nameres, lctxt);
            let e2 = comp_typed_expr(e2, LayoutType::Int, fid, ast, nameres, lctxt);

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
            let l = get_var_loc2(fid, *v, ast, nameres, lctxt);
            let ty = get_ty(l, lctxt);
            (format!("{v}"), ty)
        },
        Expr::Input => {
            (format!("input()"), LayoutType::Value)
        },
    }
}

fn comp_stmt(stmt: &Stmt, fid: FnId, ast: &AST, nameres: &Nameres, lctxt: &LCtxt, level: usize) -> String {
    let spaces = "    ".repeat(level+1);
    match stmt {
        Stmt::Global(_) => String::new(),
        Stmt::ListStore(l, i, v) => {
            let ty = get_ty(LayoutLocation::ListItem, lctxt);
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, nameres, lctxt);
            let i = comp_typed_expr(i, LayoutType::Int, fid, ast, nameres, lctxt);
            let v = comp_typed_expr(v, ty, fid, ast, nameres, lctxt);
            format!("{spaces}store_list({l}, {i}, {v});\n")
        },
        Stmt::DictStore(t, k, v) => {
            let kty = get_ty(LayoutLocation::DictKey, lctxt);
            let vty = get_ty(LayoutLocation::DictValue, lctxt);
            let t = comp_typed_expr(t, LayoutType::Dict, fid, ast, nameres, lctxt);
            let k = comp_typed_expr(k, kty, fid, ast, nameres, lctxt);
            let v = comp_typed_expr(v, vty, fid, ast, nameres, lctxt);
            format!("{spaces}store_dict({t}, {k}, {v});\n")
        },
        Stmt::Push(l, v) => {
            let ty = get_ty(LayoutLocation::ListItem, lctxt);
            let l = comp_typed_expr(l, LayoutType::List, fid, ast, nameres, lctxt);
            let v = comp_typed_expr(v, ty, fid, ast, nameres, lctxt);
            format!("{spaces}push_list({l}, {v});\n")
        },
        Stmt::Assign(v, e) => {
            let loc = get_var_loc2(fid, *v, ast, nameres, lctxt);
            let ty = get_ty(loc, lctxt);
            let e = comp_typed_expr(e, ty, fid, ast, nameres, lctxt);
            format!("{spaces}{v} = {e};\n")
        },
        Stmt::Return(e) => {
            let loc = LayoutLocation::RetVal(lctxt.fn_to_tag[&fid]);
            let ty = get_ty(loc, lctxt);
            let e = comp_typed_expr(e, ty, fid, ast, nameres, lctxt);
            format!("{spaces}return {e};\n")
        },
        Stmt::If(cond, then_, else_) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fid, ast, nameres, lctxt);
            format!("{spaces}if ({}) {{\n{}{spaces}}} else {{\n{}{spaces}}}\n", cond, comp_body(then_, fid, ast, nameres, lctxt, level+1), comp_body(else_, fid, ast, nameres, lctxt, level+1))
        },
        Stmt::While(cond, body) => {
            let cond = comp_typed_expr(cond, LayoutType::Bool, fid, ast, nameres, lctxt);
            format!("{spaces}while ({}) {{\n{}{spaces}}}\n", cond, comp_body(body, fid, ast, nameres, lctxt, level+1))
        },
        Stmt::Print(e) => {
            let e = comp_typed_expr(e, LayoutType::Value, fid, ast, nameres, lctxt);
            format!("{spaces}print_value({e});\n")
        },
    }
}

fn get_ty(loc: LayoutLocation, lctxt: &LCtxt) -> LayoutType {
    lctxt.locs.get(&loc).cloned().unwrap_or(LayoutType::Value)
}

fn comp_body(body: &Body, fid: FnId, ast: &AST, nameres: &Nameres, lctxt: &LCtxt, level: usize) -> String {
    let mut out = String::new();
    for stmt in body {
        out.push_str(&comp_stmt(stmt, fid, ast, nameres, lctxt, level));
    }
    out
}

fn get_var_loc2(fid: FnId, v: Symbol, ast: &AST, nameres: &Nameres, lctxt: &LCtxt) -> LayoutLocation {
    let loc = get_var_loc(fid, v, nameres);
    to_layout_location(loc, ast, &lctxt.fn_to_tag)
}
