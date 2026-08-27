use crate::*;

use std::collections::{BTreeSet, BTreeMap};
use crate::ir::{Terminator, BlkDef, FnDef, ValueId, Ty, BlkId, FnId, AppliedBlk};

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

fn lower_expr(e: &ast::Expr, ctxt: &mut FnCtxt) -> ValueId {
    let e = match e {
        ast::Expr::StringLit(x) => {
            let a = mk_expr(ir::Expr::StringLit(x.clone()), ctxt);
            ir::Expr::TToValue(a, Ty::String)
        },
        ast::Expr::IntLit(x) => {
            let a = mk_expr(ir::Expr::IntLit(*x), ctxt);
            ir::Expr::TToValue(a, Ty::Int)
        },
        ast::Expr::BoolLit(x) => {
            let a = mk_expr(ir::Expr::BoolLit(*x), ctxt);
            ir::Expr::TToValue(a, Ty::Bool)
        },
        ast::Expr::BinOp(kind, l, r) => {
            let l = lower_expr(l, ctxt);
            let r = lower_expr(r, ctxt);
            lower_binop(*kind, l, r, ctxt)
        },
        ast::Expr::Var(v) => {
            let i = var_idx(*v, ctxt);
            return ctxt.blockvars[&ctxt.current][i]
        },
        ast::Expr::Fn(args, body) => {
            let mut ir = std::mem::take(&mut ctxt.ir);
            let (f, ir) = lower_fn(args, body, ir);
            ctxt.ir = ir;

            ir::Expr::Fn(f)
        },
        ast::Expr::FnCall(f, args) => {
            let f = lower_expr(f, ctxt);
            let args: Vec<_> = args.iter().map(|x| lower_expr(x, ctxt)).collect();

            ir::Expr::FnCall(f, args.into())
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
        ir::Expr::Fn(..) => Ty::Value,

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

fn lower_blk(stmts: &[ast::Stmt], post: BlkId, ctxt: &mut FnCtxt) -> BlkId {
    let old = ctxt.current;
    let new = ctxt.fresh_blk();

    ctxt.focus_blk(new);

    for st in stmts {
        match st {
            ast::Stmt::Print(x) => {
                let v = lower_expr(x, ctxt);
                ctxt.push_stmt(ir::Stmt::Print(v));
            },
            ast::Stmt::If(cond, then_, else_) => {
                let cond = lower_expr(cond,  ctxt);
                let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), ctxt);

                let post = ctxt.fresh_blk();

                let then_ = lower_blk(then_, post, ctxt);
                let else_ = lower_blk(else_, post, ctxt);

                ctxt.set_terminator(Terminator::IfGoto(cond, ctxt.mk_applied_blk(then_), ctxt.mk_applied_blk(else_)));

                ctxt.focus_blk(post);
            },

            ast::Stmt::While(cond, body) => {
                let head = ctxt.fresh_blk();
                let post = ctxt.fresh_blk();

                ctxt.set_terminator(Terminator::Goto(ctxt.mk_applied_blk(head)));

                let body = lower_blk(body, head, ctxt);

                ctxt.focus_blk(head);

                let cond = lower_expr(cond, ctxt);
                let cond = mk_expr(ir::Expr::ValueToT(cond, Ty::Bool), ctxt);

                ctxt.set_terminator(Terminator::IfGoto(cond, ctxt.mk_applied_blk(body), ctxt.mk_applied_blk(post)));

                ctxt.focus_blk(post);
            },
            ast::Stmt::Assign(var, val) => {
                let val = lower_expr(val, ctxt);

                let i = var_idx(*var, ctxt);
                ctxt.blockvars.get_mut(&ctxt.current).unwrap()[i] = val;
            },
            ast::Stmt::Return(e) => {
                let v = lower_expr(e, ctxt);
                ctxt.set_terminator(Terminator::Return(v));
                break
            },
            x => todo!("{x:?}"),
        }
    }

    ctxt.set_terminator(Terminator::Goto(ctxt.mk_applied_blk(post)));
    ctxt.focus_blk(old);

    new
}

pub fn lower(ast: &AST) -> IR {
    let ir = IR {
        fns: HashMap::new(),
        global_types: HashMap::new(),
        start: 0,
    };
    let (start, mut ir) = lower_fn(&[], ast, ir);
    ir.start = start;
    ir
}

fn var_idx(v: Symbol, ctxt: &FnCtxt) -> usize {
    ctxt.vars.iter().position(|v2| v == *v2).unwrap()
}

fn lower_fn(args: &[Symbol], body: &[ast::Stmt], mut ir: IR) -> (FnId, IR) {
    // compute vars.
    let vars = get_vars(args, body);

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

    let b2 = lower_blk(body, end, &mut ctxt);

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
    let nil1 = mk_expr(ir::Expr::NilLit, &mut ctxt);
    let nil2 = mk_expr(ir::Expr::TToValue(nil1, Ty::Nil), &mut ctxt);
    ctxt.set_terminator(Terminator::Return(nil2));

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
    let mut out = args.iter().copied().collect();
    get_vars2(body, &mut out);
    out.into_iter().collect()
}

fn get_vars2(body: &[ast::Stmt], out: &mut BTreeSet<Symbol>) {
    for a in body {
        use ast::Stmt::*;
        match a {
            Global(s) => { out.insert(*s); },
            Return(e)|Print(e) => get_vars2_expr(e, out),
            Assign(v, e) => { out.insert(*v); get_vars2_expr(e, out); },
            Push(e1, e2) => { get_vars2_expr(e1, out); get_vars2_expr(e2, out); },
            ListStore(e1, e2, e3) | DictStore(e1, e2, e3) => { get_vars2_expr(e1, out); get_vars2_expr(e2, out); get_vars2_expr(e3, out); },
            If(e, b1, b2) => { get_vars2_expr(e, out); get_vars2(b1, out); get_vars2(b2, out); },
            While(e, b) => { get_vars2_expr(e, out); get_vars2(b, out); },
        }
    }
}

fn get_vars2_expr(e: &ast::Expr, out: &mut BTreeSet<Symbol>) {
    use ast::Expr::*;
    match e {
        Var(v) => { out.insert(*v); },
        IndexList(e1, e2) | IndexDict(e1, e2) | BinOp(_, e1, e2) => {
            get_vars2_expr(e1, out); get_vars2_expr(e2, out);
        },
        Length(e) => {
            get_vars2_expr(e, out);
        },
        FnCall(e, args) => {
            get_vars2_expr(e, out);
            for a in args {
                get_vars2_expr(a, out);
            }
        },
        Input|NewList|NewDict|Fn(..)|IntLit(_)|StringLit(_)|BoolLit(_)|NilLit => {},
    }
}

