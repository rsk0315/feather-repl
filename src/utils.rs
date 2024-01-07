use yansi::{Color, Paint, Style};

pub trait StrPaint {
    fn fg(&self, paint: Color) -> Paint<&Self> { paint.paint(self) }
    fn bold(&self) -> Paint<&Self> { Style::default().bold().paint(self) }
}

impl StrPaint for str {}

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
