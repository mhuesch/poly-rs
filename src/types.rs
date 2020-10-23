#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct TV(pub String);

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Type {
    TVar(TV),
    TCon(String),
    TArr(Box<Type>, Box<Type>),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Scheme(Vec<TV>, Type);


// type constructors

pub fn type_int() -> Type {
    Type::TCon("Int".to_string())
}

pub fn type_bool() -> Type {
    Type::TCon("Bool".to_string())
}
