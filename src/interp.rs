use std::collections::HashMap;
use crate::ast::*;

#[derive(Debug, Clone)]
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
        Expr::BinOp(BinOpKind::Gt, e1, e2) => {
            let Value::Int(e1) = eval_expr(e1, ctxt) else { panic!() };
            let Value::Int(e2) = eval_expr(e2, ctxt) else { panic!() };
            Value::Bool(e1 < e2)
        },
        Expr::IntLit(i) => Value::Int(*i),
        Expr::StringLit(s) => Value::Str(s.to_string()),
        Expr::Var(v) => ctxt.vars[&*v].clone(),
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
