use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
    Nil,
}

fn eval_expr(e: &Expr, vars: &mut HashMap<Symbol, Value>, ast: &AST) -> Value {
    match e {
        Expr::FnCall(f, args) => {
            let args = args.iter().map(|x| eval_expr(x, vars, ast)).collect::<Vec<_>>();
            call_fn(*f, args, ast)
        },
        Expr::BinOp(op, e1, e2) => {
            let e1 = eval_expr(e1, vars, ast);
            let e2 = eval_expr(e2, vars, ast);
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
                BinOpKind::Mul => Value::Int(e1 * e2),
                BinOpKind::Minus => Value::Int(e1 - e2),
                BinOpKind::Equ => unreachable!(),
            }
        },
        Expr::IntLit(i) => Value::Int(*i),
        Expr::StringLit(s) => Value::Str(s.to_string()),
        Expr::BoolLit(b) => Value::Bool(*b),
        Expr::Var(v) => vars.get(&*v).expect(&format!("Var '{v}' not found")).clone(),
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

fn exec_stmt(stmt: &Stmt, vars: &mut HashMap<Symbol, Value>, ast: &AST) -> Result<(), /*retval*/ Value> {
    match stmt {
        Stmt::Return(e) => {
            let val = eval_expr(e, vars, ast);
            return Err(val);
        },
        Stmt::Assign(v, e) => {
            let val = eval_expr(e, vars, ast);
            vars.insert(*v, val);
        },
        Stmt::If(cond, then_, else_) => {
            let Value::Bool(cond) = eval_expr(cond, vars, ast) else {
                panic!("non-bool conditional value!")
            };
            if cond {
               exec_body(then_, vars, ast)?;
            } else {
               exec_body(else_, vars, ast)?;
            }
        },
        Stmt::While(cond, body) => {
            loop {
                let Value::Bool(b) = eval_expr(cond, vars, ast) else {
                    panic!("non-bool conditional value (while)!")
                };
                if !b { break }
                exec_body(body, vars, ast)?;
            }
        }
        Stmt::Print(e) => {
            match eval_expr(e, vars, ast) {
                Value::Int(i) => println!("{i}"),
                Value::Str(s) => println!("{s}"),
                Value::Bool(b) => println!("{b}"),
                Value::Nil => println!("nil"),
            }
        },
    }
    Ok(())
}

fn exec_body(body: &Body, vars: &mut HashMap<Symbol, Value>, ast: &AST) -> Result<(), Value> {
    for x in body {
        exec_stmt(x, vars, ast)?;
    }
    Ok(())
}

fn call_fn(name: Symbol, args: Vec<Value>, ast: &AST) -> Value {
    let f = ast.fns.iter().find(|x| x.name == name).unwrap();

    let mut vars: HashMap<Symbol, Value> = HashMap::new();
    for (var, val) in f.args.iter().zip(args.into_iter()) {
        vars.insert(*var, val);
    }

    if let Err(v) = exec_body(&f.body, &mut vars, ast) { return v }
    Value::Nil
}

pub fn interp(ast: &AST) {
    call_fn(Symbol::new("main"), Vec::new(), ast);
}
