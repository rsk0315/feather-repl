use crate::{
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
