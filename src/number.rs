use std::str::FromStr;
use std::sync::OnceLock;

use num::{One, Signed, Zero};
use num_bigint::{
    BigInt, BigUint, ParseBigIntError,
    Sign::{self, Minus, NoSign, Plus},
};
use num_rational::BigRational;
use regex::Regex;

use crate::utils::{cycle_mu_lambda, IterDiffIndex};

/// Tuple representing a decimal number.
///
/// For example, 8.451(923076...) = 879/104 is equivalent to the following:
/// ```text
/// Decimal { int: 8, frac_once, [4, 5, 1], frac_rep: [9, 2, 3, 0, 7, 6] }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecimalTuple {
    sign: Sign,
    int: BigUint,
    frac_once: Vec<u8>,
    frac_rep: Vec<u8>,
}

impl Default for DecimalTuple {
    fn default() -> Self {
        DecimalTuple {
            sign: NoSign,
            int: BigUint::zero(),
            frac_once: vec![],
            frac_rep: vec![],
        }
    }
}

impl DecimalTuple {
    pub fn new(
        sign: Sign,
        int: impl Into<BigUint>,
        frac_once: impl Into<Vec<u8>>,
        frac_rep: impl Into<Vec<u8>>,
    ) -> Self {
        Self::to_rational(sign, int.into(), frac_once.into(), frac_rep.into())
            .into()
    }

    pub fn to_rational(
        sign: Sign,
        int: BigUint,
        frac_once: Vec<u8>,
        frac_rep: Vec<u8>,
    ) -> BigRational {
        let rat_once = frac_once
            .iter()
            .fold((BigInt::zero(), BigInt::one()), |(xn, xd), y| {
                (xn * 10 + y, xd * 10)
            });
        let mut rat_rep = frac_rep
            .iter()
            .fold((BigInt::zero(), BigInt::one()), |(xn, xd), y| {
                (xn * 10 + y, xd * 10)
            });
        if !rat_rep.0.is_zero() {
            rat_rep = (rat_rep.0, (rat_rep.1 - 1_u32) * &rat_once.1);
        }

        let mag = BigRational::from_integer(BigInt::from_biguint(Plus, int))
            + BigRational::from(rat_once)
            + BigRational::from(rat_rep);

        if sign == Minus { -mag } else { mag }
    }

    fn zero() -> Self {
        Self {
            sign: NoSign,
            int: 0_u32.into(),
            frac_once: vec![],
            frac_rep: vec![],
        }
    }

    /// The length of the longest common prefix of the decimal representation.
    ///
    /// In other words, the correct length of decimal representation (the
    /// decimal point is also counted). If they are exactly same, returns `None`.
    ///
    /// For example, `(0.9999, 1.0000)` and `(1.0, -1.0)` have 0 common digits.
    /// In addition, `(10.0, 1.0)` has 0 common digits (with properly padded).
    ///
    /// note:
    ///
    /// - `lcp_len(0.9999, 1.0000)`: 0
    /// - `lcp_len(1.0, -1.0)`: 0
    /// - `lcp_len(10.0, 1.0)`: 0 (with properly padded)
    /// - `lcp_len(-1.2, -3.4)`: 1 (minus sign)
    /// - `lcp_len(-1.0, -10.0)`: 1 (minus sign)
    pub fn lcp_len(&self, other: &DecimalTuple) -> Option<usize> {
        let (sgn_l, uint_l) = (self.sign, &self.int);
        let (sgn_r, uint_r) = (other.sign, &other.int);

        if sgn_l != sgn_r {
            // XXX: 0 (NoSign) and 0.1 (Plus)
            return Some(0);
        }

        let s_uint_l = uint_l.to_string();
        let s_uint_r = uint_r.to_string();

        let tmp = if s_uint_l.len() != s_uint_r.len() {
            Some(0)
        } else if uint_l != uint_r {
            s_uint_l.bytes().iter_diff_index(s_uint_r.bytes())
        } else {
            let len_l =
                s_uint_l.len() + 1 + self.frac_once.len() + self.frac_rep.len();
            let len_r = s_uint_r.len()
                + 1
                + other.frac_once.len()
                + other.frac_rep.len();
            let bound = 2 * len_l.max(len_r);
            let left = s_uint_l
                .bytes()
                .map(|b| b - b'0')
                .chain(Some(b'.'))
                .chain(self.frac_once.iter().copied())
                .chain(self.frac_rep.iter().copied().cycle())
                .chain(std::iter::repeat(0))
                .take(bound);
            let right = s_uint_r
                .bytes()
                .map(|b| b - b'0')
                .chain(Some(b'.'))
                .chain(other.frac_once.iter().copied())
                .chain(other.frac_rep.iter().copied().cycle())
                .chain(std::iter::repeat(0))
                .take(bound);
            left.iter_diff_index(right)
        };
        tmp.map(|x| if sgn_l == Minus { x + 1 } else { x })
    }

    pub fn is_integer(&self) -> bool {
        self.frac_once.is_empty() && self.frac_rep.is_empty()
    }

    pub fn is_repetitive(&self) -> bool { !self.frac_rep.is_empty() }
}

impl std::fmt::Display for DecimalTuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.sign == Minus {
            write!(f, "-")?;
        }
        write!(f, "{}", self.int)?;
        let mut tmp: String =
            self.frac_once.iter().map(|&b| (b + b'0') as char).collect();
        if !self.frac_rep.is_empty() {
            tmp += "(";
            tmp.extend(self.frac_rep.iter().map(|&b| (b + b'0') as char));
            tmp += "...)";
        }
        if !tmp.is_empty() {
            write!(f, ".{tmp}")?;
        }
        Ok(())
    }
}

impl From<DecimalTuple> for BigRational {
    fn from(dec: DecimalTuple) -> Self {
        DecimalTuple::to_rational(
            dec.sign,
            dec.int,
            dec.frac_once,
            dec.frac_rep,
        )
    }
}

impl From<BigRational> for DecimalTuple {
    fn from(rat: BigRational) -> Self {
        let (sgn, mag) = (rat.signum(), rat.abs());
        if sgn.is_zero() {
            return Self::zero();
        }
        let sign = if sgn.is_negative() { Minus } else { Plus };

        let (int, frac) = (mag.to_integer().into_parts().1, mag.fract());
        let (num, den) = (frac.numer(), frac.denom());

        let div_iter = |num: BigInt, den: BigInt| {
            std::iter::successors(Some((BigInt::zero(), num)), move |(_, x)| {
                Some((x * 10 / &den, x * 10 % &den))
            })
            .skip(1)
            .map(|x| x.0.try_into().unwrap())
        };
        let (mu, lambda) =
            cycle_mu_lambda(num % den, |x: &BigInt| x * 10 % den);

        let mut it = div_iter(num.to_owned(), den.to_owned());
        let frac_once: Vec<_> = it.by_ref().take(mu).collect();
        let mut frac_rep: Vec<_> = it.take(lambda).collect();
        if frac_rep == [0] {
            frac_rep.clear();
        }

        Self { sign, int, frac_once, frac_rep }
    }
}

const DECIMAL_PATTERN: &str = r"(?x)
^
(?P<SIGN>[+-])?
(?P<INT>-?[0-9]+)
(?:
    \.(?P<ONCE>[0-9]+)?
    (?P<REP>\([0-9]+\.*\))?
)?
$
";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecimalTupleParseError {
    MatchFailed,
    BigIntError(ParseBigIntError),
}
use DecimalTupleParseError::*;

impl FromStr for DecimalTuple {
    type Err = DecimalTupleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(DECIMAL_PATTERN).unwrap());
        let caps = re.captures(s).ok_or(MatchFailed)?;

        let neg = caps.name("SIGN").map(|c| c.as_str() == "-").unwrap_or(false);
        let sign = if neg { Minus } else { Plus };
        let cap_int = caps.name("INT").unwrap().as_str();
        let int: BigUint = cap_int.parse().map_err(BigIntError)?;

        let collect_digits = |name: &str| {
            caps.name(name)
                .map(|c| c.as_str().bytes())
                .into_iter()
                .flatten()
                .filter(|b| (b'0'..=b'9').contains(b))
                .map(|b| b - b'0')
                .collect::<Vec<_>>()
        };
        let once = collect_digits("ONCE");
        let rep = collect_digits("REP");

        Ok(Self::new(sign, int, once, rep))
    }
}

#[cfg(test)]
mod tests_parse {
    use super::*;

    const TEST_SUITE_OK: &[(&str, (Sign, u64, &[u8], &[u8]))] = &[
        // simple
        ("1.2", (Plus, 1, &[2], &[])),
        ("1.2(3)", (Plus, 1, &[2], &[3])),
        ("12.(3...)", (Plus, 12, &[], &[3])),
        ("2", (Plus, 2, &[], &[])),
        ("1.(001)", (Plus, 1, &[], &[0, 0, 1])),
        // normalization
        ("0.(9)", (Plus, 1, &[], &[])),
        ("0.9(9)", (Plus, 1, &[], &[])),
        ("0.199(9)", (Plus, 0, &[2], &[])),
        ("0.(11)", (Plus, 0, &[], &[1])),
        ("0.1(1)", (Plus, 0, &[], &[1])),
        ("0.11", (Plus, 0, &[1, 1], &[])),
        // sign
        ("0.5", (Plus, 0, &[5], &[])),
        ("+0.5", (Plus, 0, &[5], &[])),
        ("-0.5", (Minus, 0, &[5], &[])),
        ("-0.(9)", (Minus, 1, &[], &[])),
        // zero
        ("0", (NoSign, 0, &[], &[])),
        ("+0", (NoSign, 0, &[], &[])),
        ("-0", (NoSign, 0, &[], &[])),
        ("-0.0", (NoSign, 0, &[], &[])),
        ("-0.0(0)", (NoSign, 0, &[], &[])),
        // leading/trailing zeros
        ("00", (NoSign, 0, &[], &[])),
        ("001.10(0)", (Plus, 1, &[1], &[])),
    ];

    const TEST_SUITE_ERR: &[&str] = &[
        //
        "0.11()", "+-0", "@", "1.2.3", "0.999...", "0.1((1))", " 1 ",
    ];

    #[test]
    fn test_ok() {
        for &(s, (sign, int, frac_once, frac_rep)) in TEST_SUITE_OK {
            let expected = DecimalTuple {
                sign,
                int: int.into(),
                frac_once: frac_once.into(),
                frac_rep: frac_rep.into(),
            };
            assert_eq!(s.parse(), Ok(expected));
        }
    }

    #[test]
    fn test_err() {
        for s in TEST_SUITE_ERR {
            assert!(s.parse::<DecimalTuple>().is_err());
        }
    }
}

#[cfg(test)]
mod tests_lcp {
    use super::*;

    const TEST_SUITE: &[((&str, &str), Option<usize>)] = &[
        (("0.9999", "1.0000"), Some(0)),
        (("1.0", "-1.0"), Some(0)),
        (("10.0", "1.0"), Some(0)),
        (("-1.2", "-3.4"), Some(1)),
        (("-1.0", "-10.0"), Some(1)),
        (("12.0", "12.2"), Some(3)),
        (("1.0", "1.(001)"), Some(4)),
        (("-1.0", "-1.5"), Some(3)),
        (("0.001", "0"), Some(4)),
        (("0", "0.001"), Some(4)),
        (("-0.001", "0"), Some(5)),
        (("-0.001", "0.001"), Some(5)), // ?
        (("1", "1"), None),
        (("-1", "-1"), None),
    ];

    #[test]
    fn test() {
        for &((lhs, rhs), expected) in TEST_SUITE {
            let lhs: DecimalTuple = lhs.parse().unwrap();
            let rhs: DecimalTuple = rhs.parse().unwrap();
            assert_eq!(lhs.lcp_len(&rhs), expected);
        }
    }
}
