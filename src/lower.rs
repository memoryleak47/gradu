use crate::*;

use std::collections::{BTreeSet, BTreeMap};
use crate::ir::{Terminator, BlkDef, FnDef, ValueId, GlobalId, Ty, BlkId, FnId, AppliedBlk};

// The context that governs lowering a particular function.
struct FnCtxt {
    fn_id: FnId,
    vars: Vec<Symbol>, // list of variables.
    blockvars: HashMap<BlkId, Vec<ValueId>>, // expresses which variable corresponds to which ValueId at the last position of any block.
    current: BlkId, // do we want this?
    ir: IR,
}

fn lower_binop(kind: BinOpKind, l: ValueId, r: ValueId, ctxt: &mut FnCtxt) -> ir::Expr {
    use BinOpKind::*;

    match kind {
        Plus|Mul|Minus|Mod => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), ctxt);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), ctxt);
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), ctxt);
            ir::Expr::TToValue(o, Ty::Int)
        },
        Gt|Lt => {
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::Int), ctxt);
            let r = mk_expr(ir::Expr::ValueToT(r, Ty::Int), ctxt);
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), ctxt);
            ir::Expr::TToValue(o, Ty::Bool)
        },
        Equ|Ne => {
            let o = mk_expr(ir::Expr::BinOp(kind, l, r), ctxt);
            ir::Expr::TToValue(o, Ty::Bool)
        },
    }
}

fn lower_expr(e: &ast::Expr, ctxt: &mut FnCtxt, gctxt: &mut Vec<Symbol>) -> ValueId {
    let e = match e {
        ast::Expr::StringLit(x) => {
            let a = mk_expr(ir::Expr::StringLit(x.clone()), ctxt);
            ir::Expr::TToValue(a, Ty::String)
        },
        ast::Expr::IntLit(x) => {
            let a = mk_expr(ir::Expr::IntLit(*x), ctxt);
            ir::Expr::TToValue(a, Ty::Int)
        },
        ast::Expr::NilLit => {
            let a = mk_expr(ir::Expr::NilLit, ctxt);
            ir::Expr::TToValue(a, Ty::Nil)
        },
        ast::Expr::BoolLit(x) => {
            let a = mk_expr(ir::Expr::BoolLit(*x), ctxt);
            ir::Expr::TToValue(a, Ty::Bool)
        },
        ast::Expr::NewList => {
            let a = mk_expr(ir::Expr::NewList, ctxt);
            ir::Expr::TToValue(a, Ty::List)
        },
        ast::Expr::BinOp(kind, l, r) => {
            let l = lower_expr(l, ctxt, gctxt);
            let r = lower_expr(r, ctxt, gctxt);
            lower_binop(*kind, l, r, ctxt)
        },
        ast::Expr::Var(v) => {
            if let Some(i) = var_idx(*v, ctxt) {
                return ctxt.blockvars[&ctxt.current][i]
            } else {
                let i = gvar_idx(*v, ctxt, gctxt);
                ir::Expr::LoadGlobal(i)
            }
        },
        ast::Expr::Fn(args, body) => {
            let mut ir = std::mem::take(&mut ctxt.ir);
            let (f, ir) = lower_fn(false, args, body, ir, gctxt);
            ctxt.ir = ir;

            let a = mk_expr(ir::Expr::Fn(f), ctxt);
            ir::Expr::TToValue(a, Ty::Fn)
        },
        ast::Expr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt, gctxt);
            let args: Vec<_> = args.iter().map(|x| lower_expr(x, ctxt, gctxt)).collect();

            let f = mk_expr(ir::Expr::ValueToT(f, Ty::Fn), ctxt);
            ir::Expr::FnCall(f, args.into())
        },
        ast::Expr::IndexList(l, i) => {
            let l = lower_expr(l, ctxt, gctxt);
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::List), ctxt);

            let i = lower_expr(i, ctxt, gctxt);
            let i = mk_expr(ir::Expr::ValueToT(i, Ty::Int), ctxt);

            ir::Expr::IndexList(l, i)
        },
        ast::Expr::Length(l) => {
            let l = lower_expr(l, ctxt, gctxt);
            let l = mk_expr(ir::Expr::ValueToT(l, Ty::List), ctxt);

            let out = mk_expr(ir::Expr::Length(l), ctxt);
            ir::Expr::TToValue(out, Ty::Int)
        },
        x => todo!("{x:?}"),
    };

    mk_expr(e, ctxt)
}

fn ty_of(e: &ir::Expr) -> Ty {
    use BinOpKind::*;
    match e {
        ir::Expr::StringLit(_) => Ty::String,
        ir::Expr::IntLit(_) => Ty::Int,
        ir::Expr::BoolLit(_) => Ty::Bool,
        ir::Expr::NilLit => Ty::Nil,

        ir::Expr::TToValue(_, _) => Ty::Value,
        ir::Expr::ValueToT(_, o) => *o,

        ir::Expr::BinOp(Plus|Minus|Mod|Mul, _, _) => Ty::Int,
        ir::Expr::BinOp(Lt|Gt|Equ|Ne, _, _) => Ty::Bool,

        ir::Expr::FnCall(..) => Ty::Value,
        ir::Expr::Fn(..) => Ty::Fn,

        ir::Expr::LoadGlobal(..) => Ty::Value,
        ir::Expr::NewList => Ty::List,
        ir::Expr::IndexList(..) => Ty::Value,
        ir::Expr::Length(..) => Ty::Int,

        x => todo!("{x:?}")
    }
}

fn mk_expr(e: ir::Expr, ctxt: &mut FnCtxt) -> ValueId {
    let fresh = fresh();
    let bdef = ctxt.get_bdef();
    bdef.types.insert(fresh, ty_of(&e));
    bdef.stmts.push(ir::Stmt::Compute(fresh, e));
    fresh
}

fn lower_blk(stmts: &[ast::Stmt], post: BlkId, ctxt: &mut FnCtxt, gctxt: &mut Vec<Symbol>) -> BlkId {
    let old = ctxt.current;
    let new = ctxt.fresh_blk();

    let mut terminator_defined = false;

    ctxt.focus_blk(new);

    for st in stmts {
        match st {
            ast::Stmt::Print(x) => {
                let v = lower_expr(x, ctxt, gctxt);
                ctxt.push_stmt(ir::Stmt::Print(v));
            },
            ast::Stmt::If(cond, then_, else_) => {
                let cond = lower_expr(cond, ctxt, gctxt);
                let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), ctxt);

                let post = ctxt.fresh_blk();

                let then_ = lower_blk(then_, post, ctxt, gctxt);
                let else_ = lower_blk(else_, post, ctxt, gctxt);

                ctxt.set_terminator(Terminator::IfGoto(cond, ctxt.mk_applied_blk(then_), ctxt.mk_applied_blk(else_)));

                ctxt.focus_blk(post);
            },

            ast::Stmt::While(cond, body) => {
                let head = ctxt.fresh_blk();
                let post = ctxt.fresh_blk();

                ctxt.set_terminator(Terminator::Goto(ctxt.mk_applied_blk(head)));

                let body = lower_blk(body, head, ctxt, gctxt);

                ctxt.focus_blk(head);

                let cond = lower_expr(cond, ctxt, gctxt);
                let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), ctxt);

                ctxt.set_terminator(Terminator::IfGoto(cond, ctxt.mk_applied_blk(body), ctxt.mk_applied_blk(post)));

                ctxt.focus_blk(post);
            },
            ast::Stmt::Assign(var, val) => {
                let val = lower_expr(val, ctxt, gctxt);

                if let Some(i) = var_idx(*var, ctxt) {
                    ctxt.blockvars.get_mut(&ctxt.current).unwrap()[i] = val;
                } else {
                    let i = gvar_idx(*var, ctxt, gctxt);
                    ctxt.push_stmt(ir::Stmt::WriteGlobal(i, val));
                }
            },
            ast::Stmt::Return(e) => {
                let v = lower_expr(e, ctxt, gctxt);
                ctxt.set_terminator(Terminator::Return(v));
                terminator_defined = true;
                break
            },
            ast::Stmt::Global(_) => {}, // The Global stmt is respected in "get_vars", thus it can be ignored here.
            ast::Stmt::Push(l, v) => {
                let l = lower_expr(l, ctxt, gctxt);
                let l = mk_expr(ir::Expr::ValueToT(l, Ty::List), ctxt);
                let v = lower_expr(v, ctxt, gctxt);
                ctxt.push_stmt(ir::Stmt::Push(l, v));
            },
            ast::Stmt::ListStore(l, i, v) => {
                let l = lower_expr(l, ctxt, gctxt);
                let l = mk_expr(ir::Expr::ValueToT(l, Ty::List), ctxt);

                let i = lower_expr(i, ctxt, gctxt);
                let i = mk_expr(ir::Expr::ValueToT(i, Ty::Int), ctxt);

                let v = lower_expr(v, ctxt, gctxt);

                ctxt.push_stmt(ir::Stmt::ListStore(l, i, v));
            },
            x => todo!("{x:?}"),
        }
    }

    if !terminator_defined {
        ctxt.set_terminator(Terminator::Goto(ctxt.mk_applied_blk(post)));
    }
    ctxt.focus_blk(old);

    new
}

pub fn lower(ast: &AST) -> IR {
    let ir = IR {
        fns: HashMap::new(),
        global_types: HashMap::new(),
        start: 0,
    };
    let mut gctxt = Vec::new();
    let (start, mut ir) = lower_fn(true, &[], ast, ir, &mut gctxt);
    ir.start = start;
    ir
}

// if this returns None, we have a global variable.
fn var_idx(v: Symbol, ctxt: &FnCtxt) -> Option<usize> {
    ctxt.vars.iter().position(|v2| v == *v2)
}

fn gvar_idx(v: Symbol, ctxt: &mut FnCtxt, gctxt: &mut Vec<Symbol>) -> GlobalId {
    gctxt.iter().position(|v2| v == *v2).unwrap_or_else(|| {
        let i = gctxt.len();
        gctxt.push(v);
        ctxt.ir.global_types.insert(i, Ty::Value);
        i
    })
}

fn lower_fn(main: bool, args: &[Symbol], body: &[ast::Stmt], mut ir: IR, gctxt: &mut Vec<Symbol>) -> (FnId, IR) {
    // compute vars (main has no local variables, as its local vars are global vars)
    let vars = if main { Vec::new() } else { get_vars(args, body) };

    // create shallow function.
    let fname = fresh();
    ir.fns.insert(fname, FnDef { blocks: HashMap::new(), start: usize::MAX, retty: Ty::Value });

    // create context
    let mut ctxt = FnCtxt {
        blockvars: HashMap::new(),
        current: usize::MAX,
        fn_id: fname,
        vars,
        ir,
    };

    // allocate empty start block, and initialize everything to it.
    let start = ctxt.fresh_argless_blk();
    ctxt.focus_blk(start);
    ctxt.get_fdef().start = start;

    // define arguments.
    let args_v: Vec<ValueId> = args.iter().map(|_| ctxt.fresh_blkarg(Ty::Value)).collect();

    // define "nil".
    let nil1 = mk_expr(ir::Expr::NilLit, &mut ctxt);
    let nil2 = mk_expr(ir::Expr::TToValue(nil1, Ty::Nil), &mut ctxt);

    let end = ctxt.fresh_blk();

    let b2 = lower_blk(body, end, &mut ctxt, gctxt);

    let app = ctxt.vars.iter().map(|x|
        if let Some(i) = args.iter().position(|y| y == x) {
            args_v[i]
        } else {
            nil2
        }
    ).collect();
    ctxt.set_terminator(Terminator::Goto((b2, app)));

    // define end block to return Nil
    ctxt.focus_blk(end);
    let nil1_ = mk_expr(ir::Expr::NilLit, &mut ctxt);
    let nil2_ = mk_expr(ir::Expr::TToValue(nil1_, Ty::Nil), &mut ctxt);
    ctxt.set_terminator(Terminator::Return(nil2_));

    // initialize global vars to nil.
    if main {
        ctxt.focus_blk(start);
        for g in 0..gctxt.len() {
            ctxt.push_stmt(ir::Stmt::WriteGlobal(g, nil2));
        }
    }

    (fname, ctxt.ir)
}

// FnCtxt //

impl FnCtxt {
    fn get_blockvars(&self) -> &[ValueId] {
        &self.blockvars[&self.current]
    }

    fn get_fdef(&mut self) -> &mut FnDef {
        let fdef = self.ir.fns.get_mut(&self.fn_id).unwrap();
        fdef
    }

    fn get_bdef(&mut self) -> &mut BlkDef {
        let current = self.current;
        let fdef = self.get_fdef();
        fdef.blocks.get_mut(&current).unwrap()
    }

    fn set_terminator(&mut self, terminator: Terminator) {
        self.get_bdef().terminator = terminator;
    }

    fn push_stmt(&mut self, stmt: ir::Stmt) {
        self.get_bdef().stmts.push(stmt);
    }

    fn focus_blk(&mut self, bid: BlkId) {
        self.current = bid;
    }

    fn fresh_argless_blk(&mut self) -> BlkId {
        let old = self.current;

        let new = fresh();
        self.focus_blk(new);

        let fdef = self.ir.fns.get_mut(&self.fn_id).unwrap();
        fdef.blocks.insert(new, BlkDef {
            args: Vec::new(),
            stmts: Vec::new(),
            terminator: Terminator::Exit,
            types: HashMap::new(),
        });

        self.focus_blk(old);

        new
    }

    fn fresh_blk(&mut self) -> BlkId {
        let old = self.current;

        let new = self.fresh_argless_blk();
        self.focus_blk(new);

        let mut blockvars = Vec::new();
        for _ in 0..self.vars.len() {
            blockvars.push(self.fresh_blkarg(Ty::Value));
        }
        self.blockvars.insert(new, blockvars);

        self.focus_blk(old);

        new
    }

    fn fresh_blkarg(&mut self, ty: Ty) -> ValueId {
        let out = fresh();
        let bdef = self.get_bdef();
        bdef.types.insert(out, ty);
        bdef.args.push(out);
        out
    }

    fn mk_applied_blk(&self, target: BlkId) -> AppliedBlk {
        (target, self.get_blockvars().iter().copied().collect())
    }
}

// get_vars //

fn get_vars(args: &[Symbol], body: &[ast::Stmt]) -> Vec<Symbol> {
    let mut vars: BTreeSet<Symbol> = args.iter().copied().collect();
    let mut globals: BTreeSet<Symbol> = BTreeSet::new();

    for a in body {
        match a {
            ast::Stmt::Global(s) => { globals.insert(*s); },
            ast::Stmt::Assign(v, _) => { vars.insert(*v); },
            _ => {},
        }
    }

    (&vars - &globals).into_iter().collect()
}
