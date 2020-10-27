use combine::error::ParseError;
use combine::parser::char::{char, digit, letter, spaces, string};
use combine::stream::Stream;
use combine::{attempt, between, choice, many1, not_followed_by, optional, parser, Parser};

use super::syntax::*;

// `impl Parser` can be used to create reusable parsers with zero overhead
pub fn expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char>,
    // Necessary due to rust-lang/rust#24159
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // A parser which skips past whitespace.
    // Since we aren't interested in knowing that our expression parser
    // could have accepted additional whitespace between the tokens we also silence the error.
    let skip_spaces = || spaces().silent();

    // Creates a parser which parses a char and skips any trailing whitespace
    let lex_char = |c| char(c).skip(skip_spaces());

    let word = || many1(letter()).skip(skip_spaces());
    let integer = || many1(digit()).skip(skip_spaces());
    let str_ = |x| string(x).skip(skip_spaces());

    let name = || word().map(Name);
    let var = name().map(Expr::Var);
    let l_bool = choice((
        str_("true").map(|_| Lit::LBool(true)),
        (str_("false").map(|_| Lit::LBool(false))),
    ))
    .skip(not_followed_by(letter()));
    let l_int = (optional(char('-')), integer()).map(|t| {
        // TODO handle this error, even though it should be impossible
        let string: String = t.1;
        let num = string.parse::<i64>().unwrap();
        match t.0 {
            Some(_) => Lit::LInt(-num),
            None => Lit::LInt(num),
        }
    });
    let lit = choice((l_bool, l_int)).map(|v| Expr::Lit(v));

    let p_add = str_("+").map(|_| PrimOp::Add);
    let p_sub = str_("-").map(|_| PrimOp::Sub);
    let p_mul = str_("*").map(|_| PrimOp::Mul);
    let p_eql = str_("==").map(|_| PrimOp::Eql);
    let prim_op = choice((p_add, p_sub, p_mul, p_eql)).map(|v| Expr::Prim(v));

    let app = (expr(), expr()).map(|t| Expr::App(Box::new(t.0), Box::new(t.1)));

    let lam = (str_("lam "), lex_char('['), name(), lex_char(']'), expr())
        .map(|t| Expr::Lam(t.2, Box::new(t.4)));

    let let_ = (
        str_("let "),
        lex_char('('),
        lex_char('['),
        name(),
        expr(),
        lex_char(']'),
        lex_char(')'),
        expr(),
    )
        .map(|t| Expr::Let(t.3, Box::new(t.4), Box::new(t.7)));

    let if_ = (str_("if "), expr(), expr(), expr())
        .map(|t| Expr::If(Box::new(t.1), Box::new(t.2), Box::new(t.3)));

    let fix = (str_("fix "), expr()).map(|t| Expr::Fix(Box::new(t.1)));

    let parenthesized = choice((attempt(lam), attempt(let_), attempt(if_), attempt(fix), app));

    choice((
        attempt(lit),
        attempt(prim_op),
        attempt(var),
        between(lex_char('('), lex_char(')'), parenthesized),
    ))
    .skip(skip_spaces())
}

// As this expression parser needs to be able to call itself recursively `impl Parser` can't
// be used on its own as that would cause an infinitely large type. We can avoid this by using
// the `parser!` macro which erases the inner type and the size of that type entirely which
// lets it be used recursively.
//
// (This macro does not use `impl Trait` which means it can be used in rust < 1.26 as well to
// emulate `impl Parser`)
parser! {
    pub fn expr[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        expr_()
    }
}
