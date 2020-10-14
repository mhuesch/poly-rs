macro_rules! check_parse_expr {
    ( $a: expr, $b: expr ) => {
        let result = expr().parse($a);
        match result {
            Ok((v, "")) => assert_eq!(v, $b),
            Ok((_, _)) => assert!(false, "parse left unconsumed input"),
            Err(err) => assert!(false, "parse error: {}", err),
        }
    };
}

pub mod parse_unit {
    use combine::parser::Parser;

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
        If(
            Box::new(e0()),
            Box::new(e0()),
            Box::new(e0()),
        )
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
    fn ex_huh() {
        check_parse_expr!("(x x)", App(Box::new(e0()), Box::new(e0())));
    }
}