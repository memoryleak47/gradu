pub struct TypeLattice {
    maybe_bool: bool,
    maybe_nil: bool,
    maybe_str: bool,
    maybe_int: bool,
}

pub enum LayoutType {
    Bool,
    Nil,
    Str,
    Int,
    Value, // "any"
}
