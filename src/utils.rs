use yansi::{Color, Paint, Style};

pub trait StrPaint {
    fn fg(&self, paint: Color) -> Paint<&Self> { paint.paint(self) }
    fn bold(&self) -> Paint<&Self> { Style::default().bold().paint(self) }
}

impl StrPaint for str {}
