use crate::*;

pub type LCtxt = HashMap<Location, LayoutType>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnCallLayout {
    pub argtys: Vec<LayoutType>,
    pub retty: Box<LayoutType>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    List,
    Fn(FnCallLayout),
    Value, // "any"
}

fn inherit_fn_analysis(from: FnId, to: FnId, ast: &AST, vactxt: &mut ACtxt) {
    let from_fdef = &ast.fns[from];
    let to_fdef = &ast.fns[to];

    // retval
    add(Location::RetVal(to), &get(Location::RetVal(from), vactxt), vactxt);

    // args
    for (&from_a, &to_a) in from_fdef.args.iter().zip(to_fdef.args.iter()) {
        add(Location::Var(to, to_a), &get(Location::Var(from, from_a), vactxt), vactxt);
    }
}

pub fn layout_all(actxt: &ACtxt, ast: &AST) -> (ACtxt, LCtxt) {
    // 1. UF
    let uf = call_layout_uf(actxt, ast);

    // 2. varied actxt
    // follower -> leader to accumulate at the leader.
    let mut vactxt = actxt.clone();
    for &follower in uf.keys() {
        let leader = uf_find(follower, &uf);
        inherit_fn_analysis(follower, leader, ast, &mut vactxt);
    }

    // leader -> follower to share back.
    for &follower in uf.keys() {
        let leader = uf_find(follower, &uf);
        inherit_fn_analysis(leader, follower, ast, &mut vactxt);
    }


    let lctxt = vactxt.iter()
     .map(|(v, ty)| (*v, layout(ty.clone(), &vactxt, ast)))
     .collect();
    (vactxt.clone(), lctxt)
}

pub fn layout(x: TypeLattice, actxt: &ACtxt, ast: &AST) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) + (x.might_be_list as u8) + ((x.fn_options.len() > 0) as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else if x.might_be_list { LayoutType::List }
    else if let Some(&fid) = x.fn_options.iter().next() {
        // We only return a particular layout, if all fns agree on that layout.
        let opt = fn_type_of(fid, ast, actxt);
        for &other in x.fn_options.iter() {
            if opt != fn_type_of(other, ast, actxt) {
                return LayoutType::Value
            }
        }
        LayoutType::Fn(opt)
    } else { LayoutType::Value }
}

pub fn fn_type_of(fid: FnId, ast: &AST, actxt: &ACtxt) -> FnCallLayout {
    let mut argtys = Vec::new();
    for &a in &ast.fns[fid].args {
        argtys.push(layout(get(Location::Var(fid, a), actxt), actxt, ast));
    }
    let retty = Box::new(layout(get(Location::RetVal(fid), actxt), actxt, ast));

    FnCallLayout {
        argtys,
        retty,
    }
}




// non-reflexive unionfind! so, leaders are missing as keys.
fn uf_find(mut x: FnId, uf: &HashMap<FnId, FnId>) -> FnId {
    while let Some(y) = uf.get(&x) {
        x = *y;
    }
    x
}

fn uf_union(x: FnId, y: FnId, uf: &mut HashMap<FnId, FnId>) {
    let x = uf_find(x, uf);
    let y = uf_find(y, uf);
    if x == y { return }

    uf.insert(x, y);
}

fn call_layout_uf(actxt: &ACtxt, ast: &AST) -> HashMap<FnId, FnId> {
    let mut actxt = actxt.clone();
    let mut uf = HashMap::new();

    for (fid, f) in ast.fns.iter().enumerate() {
        visit_body(&f.body, &mut |e: &Expr|{
            if let Expr::FnCall(callee_expr, args) = e {
                let mut callees = Vec::new();
                for callee_id in ty_infer_expr(callee_expr, fid, ast, &mut actxt).fn_options {
                    if ast.fns[callee_id].args.len() != args.len() { continue }
                    callees.push(callee_id);
                }
                if callees.len() > 1 {
                    let first = callees[0];
                    for &later in &callees[1..] {
                        uf_union(first, later, &mut uf);
                    }
                }
            }
        }, &mut |_|{});
    }

    uf
}
