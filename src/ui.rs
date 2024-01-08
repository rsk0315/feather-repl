use std::ops::Range;

use combine::{easy::Errors, stream::PointerOffset};
use yansi::Style;

use crate::{
    ast::{EvalError, EvalOptions, ValueTy},
    constants::{DARK_COLOR, EMPH_COLOR, ERR_COLOR},
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

pub fn frontmatter(filename: &str, lineno: usize) {
    eprintln!(
        "{}{filename}:{lineno}{}",
        " ╭─[".fg(DARK_COLOR),
        "]".fg(DARK_COLOR)
    );
}

pub fn backmatter(s: &str, result: Result<(ValueTy, Range<usize>), EvalError>) {
    match result {
        Ok(_) => eprintln!("{}", "─╯".fg(DARK_COLOR).dimmed()),
        Err(e) => {
            let mut out = "\n".to_owned();
            out += &match e {
                EvalError::ZeroDivision(range) => s.paint_range_msg(
                    ERR_COLOR.style().bold(),
                    range,
                    "divide by zero",
                ),
            };
            lined(&out, |_| ERR_COLOR.style().dimmed());
            eprintln!("{}", "─╯".fg(ERR_COLOR).dimmed());
        }
    }
}

fn lined(lines: &str, style: impl Fn(usize) -> Style) {
    for (i, line) in lines.lines().enumerate() {
        eprintln!(" {} {line}", style(i).paint("│"));
    }
}

pub fn estimate(
    expr: &ValueTy,
    range: Range<usize>,
    s: &str,
    opts: &EvalOptions,
) {
    let (rat, flt) = expr;

    let msg =
        format!("default output: {}", EMPH_COLOR.style().bold().paint(flt));

    let mut out = "\n".to_owned();
    out += &format!(
        "{}",
        s.paint_range_msg(EMPH_COLOR.style().bold(), range, &msg)
    );

    lined(&out, |i| {
        if i == 1 { DARK_COLOR.style() } else { DARK_COLOR.style().dimmed() }
    });
}

pub fn error_report(err: Errors<char, &str, PointerOffset<str>>, s: &str) {
    let pos = err.position.translate_position(s);
    eprintln!("{}", s.paint_at(ERR_COLOR.style().bold(), pos));
    eprintln!("{0:>1$}", "^".fg(ERR_COLOR), pos + 1);
    eprintln!("errors:");
    for e in err.errors {
        println!("{}", e.to_string().fg(ERR_COLOR));
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
