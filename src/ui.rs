use std::ops::Range;

use combine::{easy::Errors, stream::PointerOffset};

use crate::{
    ast::{EvalOptions, ValueTy},
    constants::{DARK_COLOR, EMPH_COLOR},
    number::DecimalTuple,
    utils::StrPaint,
};

pub fn str_emph_correct(approx: &DecimalTuple, truth: &DecimalTuple) -> String {
    let s = approx.to_string();
    let len = match approx.lcp_len(truth) {
        Some(len) => len,
        None => return s.fg(EMPH_COLOR).to_string(),
    };

    if len < s.len() {
        format!("{}{}", s[..len].bold(), s[len..].fg(DARK_COLOR))
    } else if approx.is_integer() {
        let s0 = format!("{0:0<1$}", s + ".", len);
        format!("{}{}", s0.bold(), "(0...)".fg(DARK_COLOR))
    } else {
        let s0 = format!("{0:0<1$}", s, len);
        format!("{}{}", s0.bold(), "(0...)".fg(DARK_COLOR))
    }
}

pub fn estimate(
    expr: &ValueTy,
    range: Range<usize>,
    s: &str,
    opts: &EvalOptions,
) {
    let (rat, flt) = expr;
    eprintln!("{s}");
    eprintln!(
        "{0}{1:~^2$}",
        " ".repeat(range.start),
        '^',
        range.end - range.start
    );
}

pub fn error_report(err: Errors<char, &str, PointerOffset<str>>, s: &str) {
    eprintln!("position: {}", err.position.translate_position(s));
    eprintln!("errors:");
    for e in err.errors {
        println!("{e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SUITE: &[((&str, &str), (&str, &str))] = &[
        (("1.23", "1.24"), ("1.2", "3")),
        (("1.2", "1.3"), ("1.", "2")),
        (("-10", "-2"), ("-", "10")),
        (("1", "1.(001)"), ("1.00", "(0...)")),
        (("1.1", "1.(100)"), ("1.100", "(0...)")),
    ];

    #[test]
    fn test() {
        for &((approx, truth), (bold, dark)) in TEST_SUITE {
            let approx = approx.parse().unwrap();
            let truth = truth.parse().unwrap();
            let actual = str_emph_correct(&approx, &truth);
            let expected = format!("{}{}", bold.bold(), dark.fg(DARK_COLOR));
            assert_eq!(
                actual, expected,
                "\nactual:   {actual}\nexpected: {expected}"
            );
        }
    }
}
