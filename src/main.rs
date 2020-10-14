use poly::{
    parse::{
        expr,
    },
    pretty::*,
    syntax::*,
    syntax::Lit,
};
use Expr::*;
use combine::parser::Parser;

fn main() {
    println!("hello, poly & Rust!");

    let n = Name("hi".to_string());
    let e = Lam(n.clone(), Box::new(Var(n)));
    let e2 = App(Box::new(e.clone()), Box::new(e));
    let e3 = Fix(Box::new(Prim(PrimOp::Add)));
    let e4 = If(Box::new(Lit(Lit::LBool(true))), Box::new(e2), Box::new(e3));
    let e5 = Var(Name("free".to_string()));
    let e6 = Let(Name("x".to_string()), Box::new(e5), Box::new(e4));
    println!("{}", to_pretty(e6.ppr(), 80));

    let result = expr()
        .parse("[[], (hello, world), [rust]]");
    println!("{:?}", result);
}
