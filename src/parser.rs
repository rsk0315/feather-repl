use combine::{
    attempt, between, chainl1, choice, many1, parser,
    parser::char::{char, digit, spaces},
    stream::PointerOffset,
    Parser, Stream, StreamOnce,
};

use crate::ast::Expr;

fn parse_expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let tok =
        attempt(spaces().with(choice([char('+'), char('-')])).skip(spaces()))
            .map(|op| {
                move |l, r| match op {
                    '+' => Expr::Add(Box::new(l), Box::new(r)),
                    '-' => Expr::Sub(Box::new(l), Box::new(r)),
                    _ => unreachable!(),
                }
            });
    chainl1(parse_term(), tok)
}

fn parse_term_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let tok =
        attempt(spaces().with(choice([char('*'), char('/')])).skip(spaces()))
            .map(|op| {
                move |l, r| match op {
                    '*' => Expr::Mul(Box::new(l), Box::new(r)),
                    '/' => Expr::Div(Box::new(l), Box::new(r)),
                    _ => unreachable!(),
                }
            });
    chainl1(parse_factor(), tok)
}

fn parse_factor_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let literal = many1(digit()).map(Expr::Literal);
    let parens = between(char('('), char(')'), parse_expr());
    literal.or(parens)
}

parser! {
    fn parse_expr[Input]()(Input) -> Expr
    where
        [Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>]
    {
        parse_expr_()
    }
}

parser! {
    fn parse_term[Input]()(Input) -> Expr
    where
        [Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>]
    {
        parse_term_()
    }
}

parser! {
    fn parse_factor[Input]()(Input) -> Expr
    where
        [Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>]
    {
        parse_factor_()
    }
}

#[cfg(test)]
mod tests {
    use combine::EasyParser;

    use super::*;

    #[test]
    fn test() {
        let s = "1 * (2 - 3 + 4) / 5 ";
        let actual = parse_expr().easy_parse(s);
        assert!(actual.is_ok());
    }

    // see: <https://github.com/Marwes/combine/pull/346>
    // NOTE: spaces, source positions, ASTs
}
