use combine::{
    attempt, between, chainl1, choice,
    error::Format,
    many1, optional, parser,
    parser::{
        char::{char, digit, spaces},
        choice::ChoiceParser,
        token::Token,
    },
    position,
    stream::PointerOffset,
    unexpected_any, value, Parser, Stream, StreamOnce,
};

use crate::ast::{Expr, LitComponent};

fn parse_literal_<Input>() -> impl Parser<Input, Output = LitComponent>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let tok = (
        many1(digit()),
        optional((char('.'), many1(digit()))),
        optional(
            choice([char('E'), char('e')])
                .with((
                    optional(choice([char('+'), char('-')])),
                    many1(digit()),
                ))
                .then(|(_, s): (Option<_>, String)| match s.parse::<i32>() {
                    Ok(x) => value(x).left(),
                    Err(e) => unexpected_any(Format(e)).right(),
                }),
        ),
    );
    tok.map(
        |(int, frac, exp): (String, Option<(char, String)>, Option<i32>)| {
            let mut digits = int;
            if let Some((_, frac)) = frac {
                digits += ".";
                digits.extend(frac.chars());
            }
            LitComponent::new(digits, exp.unwrap_or(0))
        },
    )
}

fn op<Input, const N: usize>(
    s: [char; N],
) -> impl Parser<Input, Output = (PointerOffset<str>, char, PointerOffset<str>)>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
    [Token<Input>; N]: ChoiceParser<Input, Output = char>,
{
    let choices: Vec<_> = s.iter().copied().map(char).collect();
    let choices: [_; N] = choices.try_into().ok().unwrap();
    attempt(
        spaces().with((position(), choice(choices), position())).skip(spaces()),
    )
}

fn parse_expr_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let tok = op(['+', '-']).map(|(pos_l, op, pos_r)| {
        move |l, r| match op {
            '+' => Expr::Add(Box::new(l), Box::new(r), pos_l..pos_r),
            '-' => Expr::Sub(Box::new(l), Box::new(r), pos_l..pos_r),
            _ => unreachable!(),
        }
    });
    chainl1(parse_term(), tok)
}

fn parse_term_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let tok = op(['*', '/']).map(|(pos_l, op, pos_r)| {
        move |l, r| match op {
            '*' => Expr::Mul(Box::new(l), Box::new(r), pos_l..pos_r),
            '/' => Expr::Div(Box::new(l), Box::new(r), pos_l..pos_r),
            _ => unreachable!(),
        }
    });
    chainl1(parse_factor(), tok)
}

fn parse_factor_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>,
{
    let literal = (position(), parse_literal(), position())
        .map(|(pos_l, lit, pos_r)| (Expr::Literal(lit, pos_l..pos_r)));
    let parens = between(char('('), char(')'), parse_expr());
    literal.or(parens)
}

parser! {
    fn parse_literal[Input]()(Input) -> LitComponent
    where
        [Input: Stream<Token = char> + StreamOnce<Position = PointerOffset<str>>]
    {
        parse_literal_()
    }
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
        let s = "1 * (2 - 3 + 4) / 5";
        let actual = parse_expr().easy_parse(s);
        assert!(actual.is_ok());

        assert_eq!(
            actual.unwrap().0.eval(s, &()),
            (("3/5".parse().unwrap(), 0.6), 0..s.len())
        );
    }
}
