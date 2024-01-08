use std::ops::Range;

use combine::stream::PointerOffset;

#[derive(Debug)]
pub enum Expr {
    Literal(String, Range<PointerOffset<str>>),
    Mul(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Div(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Add(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Sub(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Neg(Box<Expr>, Range<PointerOffset<str>>),
}
