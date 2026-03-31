use super::style::{Style, Stylize};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

/// A styled segment of text (a single style applied to a string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub content: Cow<'static, str>,
    pub style: Style,
}

impl Span {
    pub fn raw(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            content: content.into(),
            style: Style::default(),
        }
    }

    pub fn styled(content: impl Into<Cow<'static, str>>, style: Style) -> Self {
        Self {
            content: content.into(),
            style,
        }
    }

    pub fn width(&self) -> usize {
        self.content.width()
    }
}

impl Stylize for Span {
    fn style(mut self, style: Style) -> Self {
        self.style = self.style.patch(style);
        self
    }
}

impl From<&'static str> for Span {
    fn from(s: &'static str) -> Self {
        Span::raw(s)
    }
}

impl From<String> for Span {
    fn from(s: String) -> Self {
        Span::raw(s)
    }
}

/// A single line of styled text, composed of multiple [`Span`]s.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Line {
    pub spans: Vec<Span>,
    pub alignment: Option<Alignment>,
}

/// Text alignment within a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

impl Line {
    pub fn raw(content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            spans: vec![Span::raw(content)],
            alignment: None,
        }
    }

    pub fn styled(content: impl Into<Cow<'static, str>>, style: Style) -> Self {
        Self {
            spans: vec![Span::styled(content, style)],
            alignment: None,
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn left(self) -> Self {
        self.alignment(Alignment::Left)
    }

    pub fn centered(self) -> Self {
        self.alignment(Alignment::Center)
    }

    pub fn right(self) -> Self {
        self.alignment(Alignment::Right)
    }

    /// Total visible width of this line.
    pub fn width(&self) -> usize {
        self.spans.iter().map(|s| s.width()).sum()
    }
}

impl Stylize for Line {
    fn style(mut self, style: Style) -> Self {
        for span in &mut self.spans {
            span.style = span.style.patch(style);
        }
        self
    }
}

impl From<&'static str> for Line {
    fn from(s: &'static str) -> Self {
        Line::raw(s)
    }
}

impl From<String> for Line {
    fn from(s: String) -> Self {
        Line::raw(s)
    }
}

impl From<Span> for Line {
    fn from(span: Span) -> Self {
        Self {
            spans: vec![span],
            alignment: None,
        }
    }
}

impl From<Vec<Span>> for Line {
    fn from(spans: Vec<Span>) -> Self {
        Self {
            spans,
            alignment: None,
        }
    }
}

/// Multi-line styled text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Text {
    pub lines: Vec<Line>,
    pub style: Style,
    pub alignment: Option<Alignment>,
}

impl Text {
    pub fn raw(content: impl Into<Cow<'static, str>>) -> Self {
        let content = content.into();
        let lines = content
            .split('\n')
            .map(|l| Line::raw(l.to_string()))
            .collect();
        Self {
            lines,
            style: Style::default(),
            alignment: None,
        }
    }

    pub fn styled(content: impl Into<Cow<'static, str>>, style: Style) -> Self {
        let content = content.into();
        let lines = content
            .split('\n')
            .map(|l| Line::styled(l.to_string(), style))
            .collect();
        Self {
            lines,
            style,
            alignment: None,
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Total number of lines.
    pub fn height(&self) -> usize {
        self.lines.len()
    }

    /// Width of the widest line.
    pub fn width(&self) -> usize {
        self.lines.iter().map(|l| l.width()).max().unwrap_or(0)
    }
}

impl Stylize for Text {
    fn style(mut self, style: Style) -> Self {
        self.style = self.style.patch(style);
        self
    }
}

impl From<&'static str> for Text {
    fn from(s: &'static str) -> Self {
        Text::raw(s)
    }
}

impl From<String> for Text {
    fn from(s: String) -> Self {
        Text::raw(s)
    }
}

impl From<Span> for Text {
    fn from(span: Span) -> Self {
        Self {
            lines: vec![Line::from(span)],
            style: Style::default(),
            alignment: None,
        }
    }
}

impl From<Line> for Text {
    fn from(line: Line) -> Self {
        Self {
            lines: vec![line],
            style: Style::default(),
            alignment: None,
        }
    }
}

impl From<Vec<Line>> for Text {
    fn from(lines: Vec<Line>) -> Self {
        Self {
            lines,
            style: Style::default(),
            alignment: None,
        }
    }
}

impl<T: Into<Line>> FromIterator<T> for Text {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            lines: iter.into_iter().map(Into::into).collect(),
            style: Style::default(),
            alignment: None,
        }
    }
}
