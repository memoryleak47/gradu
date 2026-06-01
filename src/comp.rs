use crate::ast::*;
use std::collections::HashSet;
use std::process::Command;

pub fn comp(ast: &AST) {
    let compiled = comp_str(ast);
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

fn comp_str(ast: &AST) -> String {
    let preamble = include_str!("preamble.h");

    let mut vars = get_vars(ast).into_iter().collect::<Vec<_>>();
    vars.sort();
    let mut varprefix = String::new();
    for x in vars {
        varprefix.push_str(&format!("    Value {x};\n"));
    }
    
    let s = comp_ast(ast);
    format!("{preamble}int main() {{\n{varprefix}{s}    return 0;\n}}")
}

fn get_vars_expr(expr: &Expr) -> HashSet<String> {
    let mut vars = HashSet::new();
    match expr {
        Expr::BinOp(_, e1, e2) => {
            vars.extend(get_vars_expr(e1));
            vars.extend(get_vars_expr(e2));
        },
        Expr::IntLit(i) => {},
        Expr::StringLit(s) => {},
        Expr::Var(v) => {
            vars.insert(v.to_string());
        },
        Expr::Input => {},
    }
    vars
}

fn get_vars(ast: &AST) -> HashSet<String> {
    let mut vars = HashSet::new();
    for st in ast {
        match st {
            Stmt::Assign(v, e) => {
                vars.insert(v.to_string());
                vars.extend(get_vars_expr(e));
            },
            Stmt::If(cond, then_, else_) => {
                vars.extend(get_vars_expr(cond));
                vars.extend(get_vars(then_));
                vars.extend(get_vars(else_));
            },
            Stmt::Print(e) => {
                vars.extend(get_vars_expr(e));
            },
        }
    }
    vars
}

fn comp_expr(e: &Expr) -> String {
    match e {
        Expr::BinOp(BinOpKind::Gt, e1, e2) => {
            let e1 = comp_expr(e1);
            let e2 = comp_expr(e2);
            let v = format!("(({e1}).payload.i > ({e2}).payload.i)");
            format!("mk_bool({v})")
        },
        Expr::IntLit(i) => {
            format!("mk_int({i})")
        },
        Expr::StringLit(s) => {
            format!("mk_str(\"{s}\")")
        },
        Expr::Var(v) => format!("{v}"),
        Expr::Input => {
            format!("input()")
        },
    }
}

fn comp_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Assign(v, e) => {
            format!("    {v} = {};\n", comp_expr(e))
        },
        Stmt::If(cond, then_, else_) => {
            format!("    if ({}.payload.b) {{\n{}    }} else {{\n{}    }}\n", comp_expr(cond), comp_ast(then_), comp_ast(else_))
        },
        Stmt::Print(e) => {
            let e = comp_expr(e);
            format!("    print_value({e});\n")
        },
    }
}


fn comp_ast(ast: &AST) -> String {
    let mut out = String::new();
    for stmt in ast {
        out.push_str(&comp_stmt(stmt));
    }
    out
}
