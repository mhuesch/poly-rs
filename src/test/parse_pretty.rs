macro_rules! check_parse_expr {
    ( $a: expr, $b: expr ) => {
        let result = expr().parse(easy::Stream($a));
        match result {
            Ok((v, stream)) => {
                if stream == easy::Stream("") {
                    assert_eq!(v, $b)
                } else {
                    assert!(false, "parse left unconsumed input")
                }
            }
            Err(err) => assert!(false, "parse error: {:?}", err),
        }
    };
}

pub mod parse_unit {
    use combine::parser::Parser;
    use combine::stream::easy;

    use crate::parse::*;
    use crate::syntax::{Lit, *};
    use Expr::*;

    fn n() -> Name {
        Name("x".to_string())
    }
    fn e0() -> Expr {
        Var(n())
    }
    fn e1() -> Expr {
        Lam(n(), Box::new(e0()))
    }
    fn e2() -> Expr {
        App(Box::new(e1()), Box::new(e1()))
    }
    fn e3() -> Expr {
        Fix(Box::new(e0()))
    }
    fn e4() -> Expr {
        If(Box::new(e0()), Box::new(e0()), Box::new(e0()))
    }
    fn e5() -> Expr {
        Var(Name("free".to_string()))
    }
    fn e6() -> Expr {
        Let(Name("v".to_string()), Box::new(e0()), Box::new(e0()))
    }
    fn e7() -> Expr {
        If(
            Box::new(Lit(Lit::LBool(true))),
            Box::new(e2()),
            Box::new(e3()),
        )
    }

    #[test]
    fn ex0() {
        check_parse_expr!("x", e0());
    }

    #[test]
    fn ex1() {
        check_parse_expr!("(lam [x] x)", e1());
    }

    #[test]
    fn ex2() {
        check_parse_expr!("((lam [x] x) (lam [x] x))", e2());
    }

    #[test]
    fn ex3() {
        check_parse_expr!("(fix x)", e3());
    }

    #[test]
    fn ex4() {
        check_parse_expr!("(if x x x)", e4());
    }

    #[test]
    fn ex5() {
        check_parse_expr!("free", e5());
    }

    #[test]
    fn ex6() {
        check_parse_expr!("(let ([v x]) x)", e6());
    }

    #[test]
    fn ex_lit_1() {
        check_parse_expr!("1", Expr::Lit(Lit::LInt(1)));
    }

    #[test]
    fn ex_lit_2() {
        check_parse_expr!("true", Expr::Lit(Lit::LBool(true)));
    }

    #[test]
    fn ex_lit_3() {
        check_parse_expr!("-2", Expr::Lit(Lit::LInt(-2)));
    }

    #[test]
    fn ex_huh() {
        check_parse_expr!("(x x)", App(Box::new(e0()), Box::new(e0())));
    }

    #[test]
    fn ex_prim_1() {
        check_parse_expr!("+", Prim(PrimOp::Add));
    }

    #[test]
    fn ex_prim_2() {
        check_parse_expr!("-", Prim(PrimOp::Sub));
    }

    #[test]
    fn ex_prim_3() {
        check_parse_expr!("*", Prim(PrimOp::Mul));
    }

    #[test]
    fn ex_prim_4() {
        check_parse_expr!("==", Prim(PrimOp::Eql));
    }

    #[test]
    fn ex_prim_5() {
        check_parse_expr!("(x +)", App(Box::new(e0()), Box::new(Prim(PrimOp::Add))));
    }

    #[test]
    fn ex_prim_6() {
        let f1 = App(Box::new(Prim(PrimOp::Add)), Box::new(Lit(Lit::LInt(4))));
        let f2 = App(Box::new(f1), Box::new(Lit(Lit::LInt(9))));
        check_parse_expr!("((+ 4) 9)", f2);
    }
}
