use crate::*;

// We group FnIds into equivalence classes represented by a common FnTag.
// Having a common FnTag implies to have the same FnCallLayout.
pub type FnTag = usize;

pub struct LCtxt {
    pub fn_to_tag: HashMap<FnId, FnTag>,
    pub call_to_tag: HashMap<*const Expr, FnTag>,

    pub locs: HashMap<LayoutLocation, LayoutType>,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum LayoutLocation {
    Arg(FnTag, usize),
    RetVal(FnTag),

    Var(/*fn*/ FnId, /*var*/ Symbol), // excluding fn args (those are governed by FnTag)
    GlobalVar(/*var*/ Symbol),

    ListItem,
    DictKey,
    DictValue,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    List,
    Dict,
    Fn(FnTag),
    Value, // "any"
}

pub fn layout(ast: &AST, nameres: &Nameres, actxt: &ACtxt) -> LCtxt {
    let (fn_to_tag, call_to_tag) = choose_tags(ast, nameres, actxt);
    let locs = build_locs(ast, actxt, &fn_to_tag);

    LCtxt {
        fn_to_tag,
        call_to_tag,
        locs,
    }
}

fn build_locs(ast: &AST, actxt: &ACtxt, fn_to_tag: &HashMap<FnId, FnTag>) -> HashMap<LayoutLocation, LayoutType> {
    let mut map: HashMap<LayoutLocation, TypeLattice> = HashMap::new();
    for (loc, l) in actxt.iter() {
        let loc = to_layout_location(*loc, ast, fn_to_tag);
        let r = map.entry(loc).or_insert(TypeLattice::bot());
        *r = TypeLattice::merge(r, l);
    }

    map.into_iter()
       .map(|(loc, lat)| (loc, layout_lat(&lat, fn_to_tag)))
       .collect()
}

pub fn to_layout_location(loc: Location, ast: &AST, fn_to_tag: &HashMap<FnId, FnTag>) -> LayoutLocation {
    match loc {
        Location::Var(f, var) => {
            if let Some(i) = ast.fns[f].args.iter().position(|x| *x == var) {
                LayoutLocation::Arg(fn_to_tag[&f], i)
            } else {
                LayoutLocation::Var(f, var)
            }
        },
        Location::GlobalVar(v) => LayoutLocation::GlobalVar(v),
        Location::RetVal(f) => LayoutLocation::RetVal(fn_to_tag[&f]),
        Location::ListItem => LayoutLocation::ListItem,
        Location::DictKey => LayoutLocation::DictKey,
        Location::DictValue => LayoutLocation::DictValue,
    }
}

fn layout_lat(x: &TypeLattice, fn_to_tag: &HashMap<FnId, FnTag>) -> LayoutType {
    if (x.might_be_int) as u8 + (x.might_be_bool as u8) + (x.might_be_nil as u8) + (x.might_be_str as u8) + (x.might_be_list as u8) + (x.might_be_dict as u8) + ((x.fn_options.len() > 0) as u8) != 1 {
        LayoutType::Value
    } else if x.might_be_bool { LayoutType::Bool }
    else if x.might_be_int { LayoutType::Int }
    else if x.might_be_str { LayoutType::Str }
    else if x.might_be_nil { LayoutType::Nil }
    else if x.might_be_list { LayoutType::List }
    else if x.might_be_dict { LayoutType::Dict }
    else if let Some(fid) = x.fn_options.iter().next() {
        let tag = fn_to_tag[fid];
        // We only return a particular layout, if all fns agree on that layout.
        for other in x.fn_options.iter() {
            if tag != fn_to_tag[other] {
                return LayoutType::Value
            }
        }
        LayoutType::Fn(tag)
    } else { LayoutType::Value }
}

/// Chooose Tags

fn choose_tags(ast: &AST, nameres: &Nameres, actxt: &ACtxt) -> (HashMap<FnId, FnTag>, HashMap<*const Expr, FnTag>) {
    let mut uf = HashMap::new();
    let mut call_to_fn = HashMap::new();

    // 1. find all fn call sites, and group fns based on it.
    let mut actxt = actxt.clone();
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
                    call_to_fn.insert(e as *const Expr, first);
                    for &later in &callees[1..] {
                        uf_union(first, later, &mut uf);
                    }
                }
            }
        }, &mut |_|{});
    }
    drop(actxt);

    let mut fn_to_tag = HashMap::new();
    for follower in 0..ast.fns.len() {
        let leader = uf_find(follower, &uf);
        if fn_to_tag.get(&leader).is_none() {
            fn_to_tag.insert(leader, fn_to_tag.len() + 10);
        }
        let v = fn_to_tag[&leader];
        fn_to_tag.insert(follower, v);
    }

    let mut call_to_tag = HashMap::new();
    for (e, f) in call_to_fn {
        call_to_tag.insert(e, fn_to_tag[&f]);
    }

    (fn_to_tag, call_to_tag)
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
