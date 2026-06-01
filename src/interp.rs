use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

#[derive(Default)]
struct Ctxt {
    vars: HashMap<String, Value>,
}

fn eval_expr(e: &Expr, ctxt: &mut Ctxt) -> Value {
    match e {
        Expr::BinOp(op, e1, e2) => {
            let e1 = eval_expr(e1, ctxt);
            let e2 = eval_expr(e2, ctxt);
            if let BinOpKind::Equ = op {
                return Value::Bool(e1 == e2)
            }

            let Value::Int(e1) = e1 else { panic!() };
            let Value::Int(e2) = e2 else { panic!() };

            match op {
                BinOpKind::Lt => Value::Bool(e1 < e2),
                BinOpKind::Gt => Value::Bool(e1 > e2),
                BinOpKind::Mod => Value::Int(e1 % e2),
                BinOpKind::Plus => Value::Int(e1 + e2),
                BinOpKind::Equ => unreachable!(),
            }
        },
        Expr::IntLit(i) => Value::Int(*i),
        Expr::StringLit(s) => Value::Str(s.to_string()),
        Expr::BoolLit(b) => Value::Bool(*b),
        Expr::Var(v) => ctxt.vars.get(&*v).expect(&format!("Var '{v}' not found")).clone(),
        Expr::Input => {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).unwrap();
            let mut s = s.trim().to_string();
            if s.starts_with("\"") && s.ends_with("\"") && s.chars().filter(|x| *x == '\"').count() == 2 {
                s.remove(s.len()-1);
                s.remove(0);

                Value::Str(s)
            } else if s == "true" {
                Value::Bool(true)
            } else if s == "false" {
                Value::Bool(false)
            } else if let Ok(i) = s.parse::<i64>() {
                Value::Int(i)
            } else {
                panic!("invalid value {s}!");
            }
        },
    }
}

fn exec_stmt(stmt: &Stmt, ctxt: &mut Ctxt) {
    match stmt {
        Stmt::Assign(v, e) => {
            let val = eval_expr(e, ctxt);
            ctxt.vars.insert(v.to_string(), val);
        },
        Stmt::If(cond, then_, else_) => {
            let Value::Bool(cond) = eval_expr(cond, ctxt) else {
                panic!("non-bool conditional value!")
            };
            if cond {
               exec_ast(then_, ctxt);
            } else {
               exec_ast(else_, ctxt);
            }
        },
        Stmt::While(cond, body) => {
            loop {
                let Value::Bool(b) = eval_expr(cond, ctxt) else {
                    panic!("non-bool conditional value (while)!")
                };
                if !b { break }
                exec_ast(body, ctxt);
            }
        }
        Stmt::Print(e) => {
            match eval_expr(e, ctxt) {
                Value::Int(i) => println!("{i}"),
                Value::Str(s) => println!("{s}"),
                Value::Bool(b) => println!("{b}"),
            }
        },
    }
}

fn exec_ast(ast: &AST, ctxt: &mut Ctxt) {
    for x in ast {
        exec_stmt(x, ctxt);
    }
}

pub fn interp(ast: &AST) {
    exec_ast(ast, &mut Ctxt::default());
}
