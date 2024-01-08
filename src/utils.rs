use std::ops::Range;

use yansi::{Color, Paint, Style};

pub trait StrPaint {
    fn fg(&self, paint: Color) -> Paint<&Self> { paint.paint(self) }
    fn bold(&self) -> Paint<&Self> { Style::default().bold().paint(self) }

    fn paint_at(&self, style: Style, i: usize) -> String;
    fn paint_range_msg(
        &self,
        style: Style,
        range: Range<usize>,
        msg: &str,
    ) -> String;
}

impl StrPaint for str {
    fn paint_at(&self, style: Style, i: usize) -> String {
        let mut res = self[..i].to_owned();
        let mut it = self[i..].chars();
        res.extend(it.next().map(|c| style.paint(c).to_string()));
        res.extend(it);
        res
    }

    fn paint_range_msg(
        &self,
        style: Style,
        range: Range<usize>,
        msg: &str,
    ) -> String {
        let Range { start, end } = range;
        let mut res = format!(
            "{}{}{}\n",
            &self[..start],
            style.paint(&self[start..end]),
            &self[end..]
        );
        let left = if end - start > 2 { 1 } else { 0 };
        let right = end - start - (left + 1);
        let line1 = format!(
            "{0}{1}{2}{3}",
            " ".repeat(start),
            "─".repeat(left),
            "┬",
            "─".repeat(right)
        );
        let line2 = format!(
            "{0}{1}{2}{3}",
            " ".repeat(start),
            " ".repeat(left),
            "╰",
            "─".repeat(2)
        );
        let color = Style::default().fg(style.fg_color()).bg(style.bg_color());
        res += &format!("{}\n{} {msg}", color.paint(line1), color.paint(line2));
        res
    }
}

pub trait IterDiffIndex<T: PartialEq>: Iterator<Item = T> {
    fn iter_diff_index<I: Iterator<Item = T>>(self, other: I) -> Option<usize>;
}

impl<T: PartialEq, I: Iterator<Item = T>> IterDiffIndex<T> for I {
    fn iter_diff_index<J: Iterator<Item = T>>(self, other: J) -> Option<usize> {
        self.zip(other).enumerate().find(|(_, (l, r))| l != r).map(|x| x.0)
    }
}

/// Lexicographically minimum `(mu, lambda)` which `x_{mu+i} == x_{mu+lambda+i}`
/// for any non-negative `i`.
pub fn cycle_mu_lambda<T: PartialEq>(
    x0: T,
    f: impl Fn(&T) -> T,
) -> (usize, usize) {
    let mut tor = f(&x0);
    let mut har = f(&tor);

    while tor != har {
        tor = f(&tor);
        har = f(&f(&har));
    }

    let mut tor = x0;
    let mut mu = 0;
    while tor != har {
        tor = f(&tor);
        har = f(&har);
        mu += 1;
    }

    let mut lambda = 1;
    har = f(&tor);
    while tor != har {
        har = f(&har);
        lambda += 1;
    }

    (mu, lambda)
}
