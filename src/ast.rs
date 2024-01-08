use std::ops::Range;

use combine::stream::PointerOffset;
use num::{FromPrimitive, Zero};
use num_rational::BigRational;

use crate::{number::DecimalTuple, ui::estimate};

#[derive(Clone, Copy, Default, Eq, PartialEq)]
struct EstimateContext {
    literal: bool,
    paren: bool,
    binary: bool,
}

impl EstimateContext {
    pub fn update(&mut self, arg: Vec<String>) {
        for arg in arg {
            for s in arg.split(",").map(|s| s.trim()) {
                match s {
                    "lit" | "+lit" => self.literal = true,
                    "par" | "+par" => self.paren = true,
                    "bin" | "+bin" => self.binary = true,
                    "-lit" => self.literal = false,
                    "-par" => self.paren = false,
                    "-bin" => self.binary = false,
                    _ => eprintln!("unexpected value: {s}"),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct EvalOptions {
    estimate: EstimateContext,
}

impl EvalOptions {
    pub fn new() -> Self { Self::default() }

    pub fn with_estimate(mut self, arg: Vec<String>) -> Self {
        self.set_estimate(arg);
        self
    }
    pub fn set_estimate(&mut self, arg: Vec<String>) {
        self.estimate.update(arg);
    }

    pub fn update(&mut self, arg: &str) {
        for s in arg.split(";").map(|s| s.trim()) {
            let mut it = s.splitn(2, "=").map(|s| s.trim());
            if let Some(key) = it.next() {
                let rem: String =
                    it.next().into_iter().map(|s| s.to_owned()).collect();
                match key {
                    "estimate" => self.set_estimate(vec![rem]),
                    _ => eprintln!("unexpected key: {key}"),
                }
            }
        }
    }

    pub fn do_estimate(&self, ctx: &EvalContext) -> bool {
        if ctx.depth == 0 {
            return true;
        }
        match ctx.expr_ty {
            ExprTy::Literal => self.estimate.literal,
            ExprTy::Paren => self.estimate.paren,
            ExprTy::Binary => self.estimate.binary,
        }
    }
}

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
    Paren(Box<Expr>, Range<PointerOffset<str>>),
    NegParen(Box<Expr>, Range<PointerOffset<str>>),
}

pub type ValueTy = (BigRational, f64);

#[derive(Debug)]
pub enum EvalError {
    ZeroDivision(Range<usize>),
}

pub enum ExprTy {
    Literal,
    Binary,
    Paren,
}

pub struct EvalContext {
    expr_ty: ExprTy,
    depth: usize,
}

impl Expr {
    pub fn eval(
        self,
        s: &str,
        opts: &EvalOptions,
        depth: usize,
    ) -> Result<(ValueTy, Range<usize>), EvalError> {
        let ctx = EvalContext {
            expr_ty: match self {
                Expr::Literal(..) => ExprTy::Literal,
                Expr::Add(..)
                | Expr::Sub(..)
                | Expr::Mul(..)
                | Expr::Div(..) => ExprTy::Binary,
                Expr::Paren(..) | Expr::NegParen(..) => ExprTy::Paren,
            },
            depth,
        };

        let (val, range) = match self {
            Expr::Literal(lit, range) => {
                let start = range.start.translate_position(s);
                let end = range.end.translate_position(s);
                (lit.eval(), start..end)
            }
            Expr::Mul(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts, depth + 1)?;
                let rhs = rhs.eval(s, opts, depth + 1)?;
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 * rhs.0.0, lhs.0.1 * rhs.0.1), range)
            }
            Expr::Div(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts, depth + 1)?;
                let rhs = rhs.eval(s, opts, depth + 1)?;
                let range = lhs.1.start..rhs.1.end;
                if rhs.0.0.is_zero() {
                    return Err(EvalError::ZeroDivision(range));
                }
                ((lhs.0.0 / rhs.0.0, lhs.0.1 / rhs.0.1), range)
            }
            Expr::Add(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts, depth + 1)?;
                let rhs = rhs.eval(s, opts, depth + 1)?;
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 + rhs.0.0, lhs.0.1 + rhs.0.1), range)
            }
            Expr::Sub(lhs, rhs, _) => {
                let lhs = lhs.eval(s, opts, depth + 1)?;
                let rhs = rhs.eval(s, opts, depth + 1)?;
                let range = lhs.1.start..rhs.1.end;
                ((lhs.0.0 - rhs.0.0, lhs.0.1 - rhs.0.1), range)
            }
            Expr::Paren(inner, range) => {
                let inner = inner.eval(s, opts, depth + 1)?;
                let start = range.start.translate_position(s);
                let end = range.end.translate_position(s);
                (inner.0, start..end)
            }
            Expr::NegParen(inner, range) => {
                let inner = inner.eval(s, opts, depth + 1)?;
                let start = range.start.translate_position(s);
                let end = range.end.translate_position(s);
                ((-inner.0.0, -inner.0.1), start..end)
            }
        };

        estimate(&val, range.clone(), s, opts, &ctx);
        Ok((val, range))
    }
}
