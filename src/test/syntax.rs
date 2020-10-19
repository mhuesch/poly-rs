use quickcheck::{empty_shrinker, Arbitrary, Gen};
use rand::Rng;

use crate::syntax::*;

impl Arbitrary for Expr {
    fn arbitrary<G: Gen>(g: &mut G) -> Expr {
        gen_expr(g, g.size())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Expr>> {
        match &*self {
            Expr::App(f, x) => {
                let pairs = (f.clone(), x.clone())
                    .shrink()
                    .map(|(f_, x_)| Expr::App(f_, x_));
                let fs = f.shrink().map(|v| *v);
                let xs = x.shrink().map(|v| *v);
                Box::new(pairs.chain(fs).chain(xs))
            }
            Expr::Lam(nm, bd) => {
                let nm_ = nm.clone();
                Box::new(
                    bd.clone()
                        .shrink()
                        .map(move |bd_| Expr::Lam(nm_.clone(), bd_)),
                )
            }
            Expr::Let(nm, e, bd) => {
                let nm_ = nm.clone();
                Box::new(
                    (e.clone(), bd.clone())
                        .shrink()
                        .map(move |(e_, bd_)| Expr::Let(nm_.clone(), e_, bd_)),
                )
            }
            Expr::If(tst, thn, els) => Box::new(
                (tst.clone(), thn.clone(), els.clone())
                    .shrink()
                    .map(|(tst_, thn_, els_)| Expr::If(tst_, thn_, els_)),
            ),
            Expr::Fix(bd) => Box::new(bd.shrink().map(|bd_| Expr::Fix(bd_))),
            _ => empty_shrinker(),
        }
    }
}

// I am pulling this out as a separate function because I do not see built-in infrastructure for
// modifying size parameters, a la Haskell-Quickcheck:
//
// https://hackage.haskell.org/package/QuickCheck-2.14.1/docs/Test-QuickCheck.html#v:scale
//
// we want this because generating a recursive type without a "reduction" in the size parameter
// for recursive calls will likely lead to unpredictably sized (potentially very large) `Expr`s.
// by passing an explicit size parameter, we can implement this directly - dividing the size
// parameter as we recur, and terminating when it hits a bound.
fn gen_expr<G: Gen>(g: &mut G, size: usize) -> Expr {
    let upper_bound = if size < 1 { 3 } else { 8 };
    match g.gen_range(0, upper_bound) {
        0 => Expr::Var(Name::arbitrary(g)),
        1 => Expr::Lit(Lit::arbitrary(g)),
        2 => Expr::Prim(PrimOp::arbitrary(g)),
        3 => {
            let f = gen_expr(g, size / 2);
            let a = gen_expr(g, size / 2);
            Expr::App(Box::new(f), Box::new(a))
        }
        4 => {
            let nm = Name::arbitrary(g);
            let bd = gen_expr(g, size * 5 / 6);
            Expr::Lam(nm, Box::new(bd))
        }
        5 => {
            let nm = Name::arbitrary(g);
            let e = gen_expr(g, size / 2);
            let bd = gen_expr(g, size / 2);
            Expr::Let(nm, Box::new(e), Box::new(bd))
        }
        6 => {
            let tst = gen_expr(g, size / 3);
            let thn = gen_expr(g, size / 3);
            let els = gen_expr(g, size / 3);
            Expr::If(Box::new(tst), Box::new(thn), Box::new(els))
        }
        7 => {
            let bd = gen_expr(g, size * 5 / 6);
            Expr::Fix(Box::new(bd))
        }
        _ => panic!("impossible: gen_expr: gen out of bounds"),
    }
}

impl Arbitrary for Lit {
    fn arbitrary<G: Gen>(g: &mut G) -> Lit {
        match g.gen_range(0, 2) {
            0 => Lit::LInt(i64::arbitrary(g)),
            1 => Lit::LBool(bool::arbitrary(g)),
            _ => panic!("impossible: Arbitrary: Lit: gen out of bounds"),
        }
    }
}

impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Name {
        let len = g.gen_range(3, 8);
        let mut s = String::new();
        for _ in 0..len {
            s.push(gen_alpha_char(g));
        }
        Name(s)
    }
}

impl Arbitrary for PrimOp {
    fn arbitrary<G: Gen>(g: &mut G) -> PrimOp {
        match g.gen_range(0, 4) {
            0 => PrimOp::Add,
            1 => PrimOp::Sub,
            2 => PrimOp::Mul,
            3 => PrimOp::Eql,
            _ => panic!("impossible: Arbitrary: PrimOp: gen out of bounds"),
        }
    }
}

fn gen_alpha_char<G: Gen>(g: &mut G) -> char {
    const ALPHA_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    const RANGE: usize = ALPHA_CHARSET.len();
    let idx = g.gen_range(0, RANGE);
    return ALPHA_CHARSET[idx as usize] as char;
}
