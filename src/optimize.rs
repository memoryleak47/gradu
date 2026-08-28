use crate::{*, ir::*};

pub fn optimize(ir: &mut IR) -> bool {
    if optimize_cast_around(ir) { return true }
    if optimize_unused_exprs(ir) { return true }
    false
}

fn expr_of(v: ValueId, bdef: &BlkDef) -> Option<&Expr> {
    for st in &bdef.stmts {
        if let Stmt::Compute(v2, e) = st && v == *v2 { return Some(e) }
    }
    None
}

fn optimize_cast_around(ir: &mut IR) -> bool {
    for (f, fdef) in &mut ir.fns {
        for (b, bdef) in &mut fdef.blocks {
            for (v, _) in &bdef.types {
                let Some(Expr::ValueToT(v2, ty)) = expr_of(*v, bdef) else { continue };
                let Some(Expr::TToValue(v3, ty2)) = expr_of(*v2, bdef) else { continue };
                if ty != ty2 { continue }

                let v = *v;
                let v3 = *v3;

                replace_value_id(v, v3, bdef);
                return true
            }
        }
    }
    false
}

fn replace_value_id(from: ValueId, to: ValueId, bdef: &mut BlkDef) {
    let mut idx = usize::MAX;
    for (j, st) in bdef.stmts.iter_mut().enumerate() {
        if let Stmt::Compute(from2, _) = st && from == *from2 { 
            idx = j;
        }
        for i in in_vals_stmt(st) {
            if *i == from { *i = to; }
        }
    }
    bdef.stmts.remove(idx);

    for i in in_vals_terminator(&mut bdef.terminator) {
        if *i == from { *i = to; }
    }
}

fn unused(x: ValueId, bdef: &BlkDef) -> bool {
    for st in &bdef.stmts {
        for a in in_vals_stmt(&mut st.clone()) {
            if *a == x { return false }
        }
    }
    for a in in_vals_terminator(&mut bdef.terminator.clone()) {
        if *a == x { return false }
    }
    true
}

fn optimize_unused_exprs(ir: &mut IR) -> bool {
    for (f, fdef) in &mut ir.fns {
        for (b, bdef) in &mut fdef.blocks {
            for (i, st) in bdef.stmts.iter().enumerate() {
                let Stmt::Compute(a, ex) = st else { continue };
                if !is_side_effect_free(ex) { continue }
                if !unused(*a, bdef) { continue }
                bdef.stmts.remove(i);
                return true
            }
        }
    }
    false
}

fn is_side_effect_free(e: &Expr) -> bool {
    use Expr::*;
    match e {
        // ValueToT is a side-effect as it can crash if the type fails.
        ValueToT(..)|Input|IndexList(..)|IndexDict(..)|FnCall(..) => false,
        TToValue(..)|NilLit|BoolLit(..)|StringLit(..)|IntLit(..)|Length(..)|BinOp(..)|LoadGlobal(..)|Fn(_)|NewList|NewDict => true,
    }
}

// abstract interpretation:

enum Location {
    Local(ValueId),
    Global(GlobalId),
    Ret(FnId),
}

struct AbstractValue {
    string: bool,
    int: bool,
    nil: bool,
    fns: HashSet<FnId>,
    dict: bool,
    list: bool,
}

type AIState = HashMap<Location, AbstractValue>;

fn abstract_interpretation(ir: &IR) -> AIState {
    let mut state = AIState::default();
    let fstart = ir.start;
    let fdef = &ir.fns[&fstart];
    let bstart = fdef.start;
    ai_run(bstart, fstart, ir, &mut state);
    state
}

fn ai_run(bid: BlkId, fid: FnId, ir: &IR, state: &mut AIState) {
    let bdef = &ir.fns[&fid].blocks[&bid];
    for st in &bdef.stmts {
        match st {
            x => todo!("{x:?}"),
        }
    }
}
