use std::ops::Range;

use combine::stream::PointerOffset;
use num::FromPrimitive;
use num_rational::BigRational;

use crate::number::DecimalTuple;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LitComponent {
    digits: String,
    exponent: i32,
}

impl LitComponent {
    pub fn new(digits: String, exponent: i32) -> Self {
        Self { digits, exponent }
    }

    pub fn eval(&self) -> ValueTy {
        let rat: BigRational =
            self.digits.parse::<DecimalTuple>().unwrap().into();
        let exp = BigRational::from_i32(10).unwrap().pow(self.exponent);
        let flt: f64 =
            format!("{}E{}", self.digits, self.exponent).parse().unwrap();
        (rat * exp, flt)
    }
}

#[derive(Debug)]
pub enum Expr {
    Literal(LitComponent, Range<PointerOffset<str>>),
    Mul(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Div(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Add(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Sub(Box<Expr>, Box<Expr>, Range<PointerOffset<str>>),
    Neg(Box<Expr>, Range<PointerOffset<str>>),
}

type EvalOptions = (); // temporary

type ValueTy = (BigRational, f64);

impl Expr {
    pub fn eval(self, s: &str, opts: &EvalOptions) -> (ValueTy, Range<usize>) {
        match self {
            Expr::Literal(lit, range) => {
                let start = range.start.translate_position(s);
                let end = range.end.translate_position(s);
                (lit.eval(), start..end)
            }
            Expr::Mul(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts);
                let rhs = rhs.eval(s, opts);
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 * rhs.0.0, lhs.0.1 * rhs.0.1), range)
            }
            Expr::Div(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts);
                let rhs = rhs.eval(s, opts);
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 / rhs.0.0, lhs.0.1 / rhs.0.1), range)
            }
            Expr::Add(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts);
                let rhs = rhs.eval(s, opts);
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 + rhs.0.0, lhs.0.1 + rhs.0.1), range)
            }
            Expr::Sub(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts);
                let rhs = rhs.eval(s, opts);
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 - rhs.0.0, lhs.0.1 - rhs.0.1), range)
            }
            Expr::Neg(rhs, range) => {
                let rhs = rhs.eval(s, opts);
                let range = range.start.translate_position(s)..rhs.1.end;
                ((-rhs.0.0, -rhs.0.1), range)
            }
        }
    }
}
