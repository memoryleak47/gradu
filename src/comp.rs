use crate::*;
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
        Expr::IntLit(_) => {},
        Expr::StringLit(_) => {},
        Expr::BoolLit(_) => {},
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
            Stmt::While(cond, body) => {
                vars.extend(get_vars_expr(cond));
                vars.extend(get_vars(body));
            },
            Stmt::Print(e) => {
                vars.extend(get_vars_expr(e));
            },
        }
    }
    vars
}

// always produces "Value" type
fn comp_expr(e: &Expr) -> String {
    let (s, ty) = comp_expr_raw(e);
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

fn comp_expr_raw(e: &Expr) -> (String, LayoutType) {
    match e {
        Expr::BinOp(op, e1, e2) => {
            let (e1, t1) = comp_expr_raw(e1);
            let (e2, t2) = comp_expr_raw(e2);
            if let BinOpKind::Equ = op {
                let e1 = type_cast_to_value(e1, t1);
                let e2 = type_cast_to_value(e2, t2);
                return (format!("is_equal({e1}, {e2})"), LayoutType::Bool)
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
        Expr::Var(v) => (format!("{v}"), LayoutType::Value),
        Expr::Input => {
            (format!("input()"), LayoutType::Value)
        },
    }
}

fn comp_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Assign(v, e) => {
            format!("    {v} = {};\n", comp_expr(e))
        },
        Stmt::If(cond, then_, else_) => {
            let (cond, tcond) = comp_expr_raw(cond);
            let cond = type_cast_to_bool(cond, tcond);
            format!("    if ({}) {{\n{}    }} else {{\n{}    }}\n", cond, comp_ast(then_), comp_ast(else_))
        },
        Stmt::While(cond, body) => {
            let (cond, tcond) = comp_expr_raw(cond);
            let cond = type_cast_to_bool(cond, tcond);
            format!("    while ({}) {{\n{}    }}\n", cond, comp_ast(body))
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
