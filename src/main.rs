use combine::parser::Parser;
use combine::stream::easy;
use poly::{env::*, infer::*, parse::expr};

fn main() {
    println!("hello, poly & Rust!");

    let prog = "((lam [x] x) 1)";
    let e = match expr().parse(easy::Stream(prog)) {
        Ok((e, _)) => e,
        _ => panic!("wat"),
    };
    println!("{:?}", e);

    let env = Env::new();
    let res = constraints_expr(env, e);
    println!("{:?}", res);
}
