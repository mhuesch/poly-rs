use combine::stream::position;
use combine::error::{ParseError, StdParseResult};
use combine::parser::char::{char, letter, spaces};
use combine::stream::{Positioned, Stream};
use combine::{
    between, choice, many1, parser, satisfy, sep_by, skip_many, skip_many1, token, EasyParser,
    Parser,
};

#[derive(Debug, PartialEq)]
pub enum Expr {
    Id(String),
    Array(Vec<Expr>),
    Pair(Box<Expr>, Box<Expr>)
}

// `impl Parser` can be used to create reusable parsers with zero overhead
pub fn expr_<Input>() -> impl Parser< Input, Output = Expr>
    where Input: Stream<Token = char>,
          // Necessary due to rust-lang/rust#24159
          Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let word = many1(letter());

    // A parser which skips past whitespace.
    // Since we aren't interested in knowing that our expression parser
    // could have accepted additional whitespace between the tokens we also silence the error.
    let skip_spaces = || spaces().silent();

    //Creates a parser which parses a char and skips any trailing whitespace
    let lex_char = |c| char(c).skip(skip_spaces());

    let comma_list = sep_by(expr(), lex_char(','));
    let array = between(lex_char('['), lex_char(']'), comma_list);

    //We can use tuples to run several parsers in sequence
    //The resulting type is a tuple containing each parsers output
    let pair = (lex_char('('),
                expr(),
                lex_char(','),
                expr(),
                lex_char(')'))
                   .map(|t| Expr::Pair(Box::new(t.1), Box::new(t.3)));

    choice((
        word.map(Expr::Id),
        array.map(Expr::Array),
        pair,
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
