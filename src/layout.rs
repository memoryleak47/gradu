use crate::*;

// We group FnIds into equivalence classes represented by a common FnTag.
// Having a common FnTag implies to have the same FnCallLayout.
pub type FnTag = usize;

pub struct LCtxt {
    pub fn_map: HashMap<FnId, FnTag>,
    pub calls: HashMap<*const Expr, FnTag>,

    pub fn_tag_layout: HashMap<FnTag, FnCallLayout>,
    pub locs: HashMap<Location, LayoutType>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    List,
    Fn(FnTag),
    Value, // "any"
}

pub struct FnCallLayout {
    pub argtys: Vec<LayoutType>,
    pub retty: LayoutType,
}

pub fn layout(ast: &AST, nameres: &Nameres, actxt: &ACtxt) -> LCtxt {
    // 1. UF
    let (fn_map, actxt, calls) = call_layout_uf(ast, nameres, actxt);

    // 2. accumulate FnCallLayouts
    let mut fn_tag_a: HashMap<FnTag, FnCallLattice> = HashMap::new();
    for f in 0..ast.fns.len() {
        let tag = fn_map[&f];
        if fn_tag_a.get(&tag).is_none() {
            let arity = ast.fns[f].args.len();
            fn_tag_a.insert(tag, FnCallLattice::bot(arity));
        }
        let flat = FnCallLattice::mk(f, &actxt, ast);
        fn_tag_a.insert(tag, FnCallLattice::merge(&fn_tag_a[&tag], &flat));
    }

    let fn_tag_layout = fn_tag_a.into_iter().map(|(k, v)| {
        let argtys = v.argtys.iter().map(|x| layout_lat(x, &fn_map)).collect();
        let retty = layout_lat(&v.retty, &fn_map);
        let v = FnCallLayout { argtys, retty };
        (k, v)
    }).collect();

    let locs = actxt.iter()
     .map(|(v, ty)| (*v, layout_lat(ty, &fn_map)))
     .collect();

    LCtxt {
        fn_map,
        calls,
        fn_tag_layout,
        locs,
    }
}

fn layout_lat(x: &TypeLattice, fn_map: &HashMap<FnId, FnTag>) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) + (x.might_be_list as u8) + ((x.fn_options.len() > 0) as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else if x.might_be_list { LayoutType::List }
    else if let Some(fid) = x.fn_options.iter().next() {
        let tag = fn_map[fid];
        // We only return a particular layout, if all fns agree on that layout.
        for other in x.fn_options.iter() {
            if tag != fn_map[other] {
                return LayoutType::Value
            }
        }
        LayoutType::Fn(tag)
    } else { LayoutType::Value }
}

/// Unionfind
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

fn call_layout_uf(ast: &AST, nameres: &Nameres, actxt: &ACtxt) -> (HashMap<FnId, FnTag>, ACtxt, HashMap<*const Expr, FnTag>) {
    let mut actxt = actxt.clone();

    let mut uf = HashMap::new();
    let mut calls = HashMap::new();

    for (fid, f) in ast.fns.iter().enumerate() {
        visit_body(&f.body, &mut |e: &Expr|{
            if let Expr::FnCall(callee_expr, args) = e {
                let mut callees = Vec::new();
                for callee_id in ty_infer_expr(callee_expr, fid, ast, nameres, &mut actxt).fn_options {
                    if ast.fns[callee_id].args.len() != args.len() { continue }
                    callees.push(callee_id);
                }
                if callees.len() >= 1 {
                    let first = callees[0];
                    calls.insert(e as *const Expr, first);
                    for &later in &callees[1..] {
                        uf_union(first, later, &mut uf);
                    }
                }
            }
        }, &mut |_|{});
    }

    // 2. update actxt
    for &follower in uf.keys() {
        let leader = uf_find(follower, &uf);
        inherit_fn_analysis(follower, leader, ast, &mut actxt);
    }

    // leader -> follower to share back.
    for &follower in uf.keys() {
        let leader = uf_find(follower, &uf);
        inherit_fn_analysis(leader, follower, ast, &mut actxt);
    }

    let mut fn_map = HashMap::new();
    for follower in 0..ast.fns.len() {
        let leader = uf_find(follower, &uf);
        if fn_map.get(&leader).is_none() {
            fn_map.insert(leader, fn_map.len() + 10);
        }
        let v = fn_map[&leader];
        fn_map.insert(follower, v);
    }

    let mut calls2 = HashMap::new();
    for (e, v) in calls {
        calls2.insert(e, fn_map[&v]);
    }

    (fn_map, actxt, calls2)
}


#[derive(Clone)]
struct FnCallLattice {
    argtys: Vec<TypeLattice>,
    retty: TypeLattice,
}


impl FnCallLattice {
    fn bot(n: usize) -> FnCallLattice {
        FnCallLattice {
            argtys: vec![TypeLattice::bot(); n],
            retty: TypeLattice::bot(),
        }
    }

    fn merge(l1: &FnCallLattice, l2: &FnCallLattice) -> FnCallLattice {
        let mut l1: FnCallLattice = l1.clone();
        for i in 0..l1.argtys.len() {
            l1.argtys[i] = TypeLattice::merge(&l1.argtys[i], &l2.argtys[i]);
        }
        l1.retty = TypeLattice::merge(&l1.retty, &l2.retty);

        l1
    }

    fn mk(fid: FnId, actxt: &ACtxt, ast: &AST) -> FnCallLattice {
        let mut argtys = Vec::new();
        for &x in &ast.fns[fid].args {
            argtys.push(get(Location::Var(fid, x), actxt));
        }

        let retty = get(Location::RetVal(fid), actxt);

        FnCallLattice {
            argtys,
            retty,
        }
    }
}


fn inherit_fn_analysis(from: FnId, to: FnId, ast: &AST, actxt: &mut ACtxt) {
    let from_fdef = &ast.fns[from];
    let to_fdef = &ast.fns[to];

    // retval
    add(Location::RetVal(to), &get(Location::RetVal(from), actxt), actxt);

    // args
    for (&from_a, &to_a) in from_fdef.args.iter().zip(to_fdef.args.iter()) {
        add(Location::Var(to, to_a), &get(Location::Var(from, from_a), actxt), actxt);
    }
}
