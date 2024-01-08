use std::ops::Range;

use combine::{easy::Errors, stream::PointerOffset};
use num::{FromPrimitive, One, Signed, Zero};
use num_rational::BigRational;
use yansi::Style;

use crate::{
    ast::{EvalContext, EvalError, EvalOptions, ValueTy},
    constants::{DARK_COLOR, EMPH_COLOR, ERR_COLOR},
    number::DecimalTuple,
    utils::StrPaint,
};

fn str_emph_correct(approx: &DecimalTuple, truth: &DecimalTuple) -> String {
    let s = approx.to_string();
    let len = match approx.lcp_len(truth) {
        Some(len) => len,
        None => return Style::default().bold().paint(s).to_string(),
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

fn str_approx(approx: &DecimalTuple, truth: &DecimalTuple) -> String {
    let rat_approx = BigRational::from(approx.to_owned());
    let rat_truth = BigRational::from(truth.to_owned());

    let t = if truth.is_repetitive() {
        rat_truth.to_string()
    } else {
        truth.to_string()
    };

    if rat_approx == rat_truth {
        return t;
    }

    let abs = &rat_approx - &rat_truth;
    let rel = &abs / &rat_truth;
    let num_tz = rel.numer().trailing_zeros().unwrap_or(0) as i32;
    let den_tz = rel.denom().trailing_zeros().unwrap_or(0) as i32;
    let exp = num_tz - den_tz;
    let rel = rel.abs() / BigRational::from_i32(2).unwrap().pow(exp);

    if rel.is_one() {
        format!(
            "{t} * (1 {} {})",
            if abs.is_positive() { '+' } else { '-' },
            if exp == 0 { "1".to_owned() } else { format!("2^{{{exp}}}") }
        )
    } else {
        format!(
            "{t} * (1 {} {rel}{})",
            if abs.is_positive() { '+' } else { '-' },
            if exp == 0 { "".to_owned() } else { format!(" * 2^{{{exp}}}") }
        )
    }
}

pub fn frontmatter(filename: &str, lineno: usize) {
    eprintln!(
        "\n{}{filename}:{lineno}{}",
        " ╭─[".fg(DARK_COLOR),
        "]".fg(DARK_COLOR)
    );
}

pub fn backmatter(s: &str, result: Result<(ValueTy, Range<usize>), EvalError>) {
    match result {
        Ok(_) => {
            eprintln!("{}", "─╯".fg(DARK_COLOR).dimmed());
        }
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
    ctx: &EvalContext,
) {
    if !opts.do_estimate(ctx) {
        return;
    }

    let (rat, flt) = expr;

    let msg = format!(
        "{}: {:?}\n",
        Style::default().bold().paint("{this:?}"),
        EMPH_COLOR.style().bold().paint(flt)
    );

    let mut out = "\n".to_owned();
    out += &format!(
        "{}",
        s.paint_range_msg(EMPH_COLOR.style().bold(), range, &msg)
    );

    out += "\n";
    out += &format!("truth: {rat}\n");
    if !rat.is_integer() {
        out += &format!("     = {}\n", DecimalTuple::from(rat.to_owned()));
    }

    let d_rat = DecimalTuple::from(rat.to_owned());
    let f = if flt.is_nan() {
        "nan".to_owned()
    } else if flt.is_infinite() {
        (if flt.is_positive() { "infinity" } else { "-infinity" }).to_owned()
    } else if *flt == 0.0 && flt.is_sign_negative() {
        // note: to produce -0.0 without the unary minus, e.g.
        // `1 / ((0 - 1) / (1e20 + 1 - 1e20))`.
        "-0".to_owned()
    } else {
        let d_flt = DecimalTuple::from(BigRational::from_float(*flt).unwrap());
        str_emph_correct(&d_flt, &d_rat)
    };
    out += &format!("float: {}\n", f);
    if !rat.is_zero() && flt.is_finite() {
        let d_flt = DecimalTuple::from(BigRational::from_float(*flt).unwrap());
        out += &format!("     = {}\n", str_approx(&d_flt, &d_rat));
    }

    lined(&out, |i| {
        if i == 1 { DARK_COLOR.style() } else { DARK_COLOR.style().dimmed() }
    });
}

pub fn error_report(err: Errors<char, &str, PointerOffset<str>>, s: &str) {
    let pos = err.position.translate_position(s);
    let eof = if pos >= s.len() {
        "$".fg(DARK_COLOR).dimmed().to_string()
    } else {
        "".to_owned()
    };
    let mut out = vec![
        "".to_owned(),
        format!("{}{eof}", s.paint_at(ERR_COLOR.style().bold(), pos)),
        format!("{0:>1$}", "┬".fg(ERR_COLOR), pos + 1),
        format!("{0:>1$}", "╰── parse error".fg(ERR_COLOR), pos + 15),
        "".to_owned(),
        format!("{}", "errors:".fg(DARK_COLOR)),
    ];
    for e in err.errors {
        out.push(format!(
            " {}  {}",
            "*".fg(DARK_COLOR).dimmed(),
            e.to_string().fg(DARK_COLOR)
        ));
    }
    let out: String = out.join("\n");
    lined(&out, |i| match i {
        0 => DARK_COLOR.style().dimmed(),
        1 => ERR_COLOR.style(),
        _ => ERR_COLOR.style().dimmed(),
    });
    eprintln!("{}", "─╯".fg(ERR_COLOR).dimmed());
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
        (("0", "0.01"), ("0.0", "(0...)")),
        (("0.01", "0"), ("0.0", "1")),
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
