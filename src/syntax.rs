#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name(pub String);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Var(Name),
    App(Box<Expr>, Box<Expr>),
    Lam(Name, Box<Expr>),
    Let(Name, Box<Expr>, Box<Expr>),
    Lit(Lit),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Fix(Box<Expr>),
    Prim(PrimOp),
}

#[macro_export]
macro_rules! app {
    ( $a: expr, $b: expr ) => {
        Expr::App(Box::new($a), Box::new($b))
    };
}

#[macro_export]
macro_rules! lam {
    ( $a: expr, $b: expr ) => {
        Expr::Lam($a, Box::new($b))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum Lit {
    LInt(i64),
    LBool(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimOp {
    Add,
    Sub,
    Mul,
    Eql,
    Null,
    Map,
    Foldl,
    Pair,
    Fst,
    Snd,
    Cons,
    Nil,
}

#[derive(Clone, Debug)]
pub struct Defn(pub Name, pub Expr);

#[derive(Clone, Debug)]
pub struct Program {
    pub p_defns: Vec<Defn>,
    pub p_body: Expr,
}

// helpers

pub fn primop_arity(op: PrimOp) -> u8 {
    match op {
        PrimOp::Add => 2,
        PrimOp::Sub => 2,
        PrimOp::Mul => 2,
        PrimOp::Eql => 2,
        PrimOp::Null => 1,
        PrimOp::Map => 2,
        PrimOp::Foldl => 3,
        PrimOp::Pair => 2,
        PrimOp::Fst => 1,
        PrimOp::Snd => 1,
        PrimOp::Cons => 2,
        PrimOp::Nil => 0,
    }
}
