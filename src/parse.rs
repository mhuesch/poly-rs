use combine::stream::position;
use combine::error::{ParseError, StdParseResult};
use combine::parser::char::{char, letter, spaces, string};
use combine::stream::{Positioned, Stream};
use combine::{
    between, choice, many1, parser, satisfy, sep_by, skip_many, skip_many1, token, EasyParser,
    Parser,
};

use super::syntax::*;

// `impl Parser` can be used to create reusable parsers with zero overhead
pub fn expr_<Input>() -> impl Parser< Input, Output = Expr>
    where Input: Stream<Token = char>,
          // Necessary due to rust-lang/rust#24159
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // A parser which skips past whitespace.
    // Since we aren't interested in knowing that our expression parser
    // could have accepted additional whitespace between the tokens we also silence the error.
    let skip_spaces = || spaces().silent();

    // Creates a parser which parses a char and skips any trailing whitespace
    let lex_char = |c| char(c).skip(skip_spaces());

    let word = || many1(letter());
    let str_ = |x| string(x).skip(skip_spaces());

    let name = || word().map(Name);
    let var = name().map(Expr::Var);

    let app = (expr(), expr()).map(|t| Expr::App(Box::new(t.0), Box::new(t.1)));

    let lam = (str_("lam"), lex_char('['), name(), lex_char(']'), expr()).map(|t| Expr::Lam(t.2, Box::new(t.4)));

    let let_ = (str_("let"), lex_char('('), lex_char('['), name(), expr(), lex_char(']'), lex_char(')'), expr()).map(|t| Expr::Let(t.3, Box::new(t.4), Box::new(t.7)));

    let parenthesized = choice((
            lam,
            let_,
            app,
            ));

    choice((
        var,
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
parser!{
    pub fn expr[Input]()(Input) -> Expr
    where [Input: Stream<Token = char>]
    {
        expr_()
    }
}
