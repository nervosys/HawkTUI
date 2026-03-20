//! Text reflow: word wrapping and line truncation.
//!
//! This module provides algorithms for reflowing styled text to fit within a
//! given width. Two strategies are available:
//!
//! - [`WordWrapper`] — breaks at word boundaries (spaces), falling back to
//!   character breaks for words wider than the available width.
//! - [`CharWrapper`] — breaks at character (grapheme cluster) boundaries.
//! - [`LineTruncator`] — truncates lines that exceed the width.
//!
//! All three operate on [`Line`]s of [`Span`]s, preserving per-span styling
//! through the reflow process.

use crate::core::style::Style;
use crate::core::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A single styled grapheme (one visible character with its style).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledGrapheme {
    pub grapheme: String,
    pub style: Style,
}

impl StyledGrapheme {
    pub fn width(&self) -> usize {
        self.grapheme.width()
    }
}

/// Explode a `Line` into individual styled graphemes.
fn line_to_graphemes(line: &Line, base_style: Style) -> Vec<StyledGrapheme> {
    let mut out = Vec::new();
    for span in &line.spans {
        let style = base_style.patch(span.style);
        for g in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
            out.push(StyledGrapheme {
                grapheme: g.to_string(),
                style,
            });
        }
    }
    out
}

/// Reassemble a slice of styled graphemes into a `Line`, merging adjacent
/// graphemes that share the same style into a single `Span`.
fn graphemes_to_line(graphemes: &[StyledGrapheme]) -> Line {
    if graphemes.is_empty() {
        return Line::default();
    }
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut current_style = graphemes[0].style;

    for sg in graphemes {
        if sg.style == current_style {
            current_text.push_str(&sg.grapheme);
        } else {
            if !current_text.is_empty() {
                spans.push(Span::styled(current_text.clone(), current_style));
                current_text.clear();
            }
            current_style = sg.style;
            current_text.push_str(&sg.grapheme);
        }
    }
    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }
    Line::from(spans)
}

/// Word-wrap lines to fit within `max_width` columns.
///
/// Breaks lines at whitespace boundaries. If a single word is wider than
/// `max_width`, it is broken at character boundaries as a fallback.
/// Trailing whitespace at line breaks is consumed (not carried to the next line).
pub struct WordWrapper;

impl WordWrapper {
    pub fn wrap(lines: &[Line], max_width: u16, base_style: Style) -> Vec<Line> {
        let max = max_width as usize;
        if max == 0 {
            return vec![];
        }
        let mut result = Vec::new();

        for line in lines {
            let graphemes = line_to_graphemes(line, base_style);
            if graphemes.is_empty() {
                result.push(Line::default());
                continue;
            }

            let mut current_line: Vec<StyledGrapheme> = Vec::new();
            let mut current_width: usize = 0;
            let mut word: Vec<StyledGrapheme> = Vec::new();
            let mut word_width: usize = 0;

            for sg in &graphemes {
                let gw = sg.width();
                let is_space = sg.grapheme.trim().is_empty();

                if is_space {
                    // Flush word to current line
                    if word_width + current_width <= max {
                        current_line.append(&mut word);
                        current_width += word_width;
                        word.clear();
                        word_width = 0;
                    } else {
                        // Word doesn't fit — start a new line
                        if !current_line.is_empty() {
                            result.push(graphemes_to_line(&current_line));
                            current_line.clear();
                            current_width = 0;
                        }
                        // Word itself might be too wide — char-break it
                        if word_width > max {
                            Self::char_break_word(&mut result, &word, max);
                            word.clear();
                            word_width = 0;
                            continue;
                        } else {
                            current_line.append(&mut word);
                            current_width = word_width;
                            word.clear();
                            word_width = 0;
                        }
                    }
                    // Add the space if it fits
                    if current_width + gw <= max {
                        current_line.push(sg.clone());
                        current_width += gw;
                    } else {
                        // Space at end of line — consume it, start new line
                        result.push(graphemes_to_line(&current_line));
                        current_line.clear();
                        current_width = 0;
                    }
                } else {
                    word.push(sg.clone());
                    word_width += gw;
                }
            }

            // Flush remaining word
            if !word.is_empty() {
                if word_width + current_width <= max {
                    current_line.append(&mut word);
                } else {
                    if !current_line.is_empty() {
                        result.push(graphemes_to_line(&current_line));
                        current_line.clear();
                    }
                    if word_width > max {
                        Self::char_break_word(&mut result, &word, max);
                        word.clear();
                    } else {
                        current_line = std::mem::take(&mut word);
                    }
                }
            }

            // Flush remaining line
            if !current_line.is_empty() {
                result.push(graphemes_to_line(&current_line));
            } else if result.is_empty() || word.is_empty() {
                // Ensure at least one line per input line if everything was flushed
                if graphemes.iter().all(|g| g.grapheme.trim().is_empty()) {
                    result.push(Line::default());
                }
            }
        }

        result
    }

    /// Break an oversized word into lines of at most `max` columns.
    fn char_break_word(result: &mut Vec<Line>, word: &[StyledGrapheme], max: usize) {
        let mut current: Vec<StyledGrapheme> = Vec::new();
        let mut w = 0usize;
        for sg in word {
            let gw = sg.width();
            if w + gw > max && !current.is_empty() {
                result.push(graphemes_to_line(&current));
                current.clear();
                w = 0;
            }
            current.push(sg.clone());
            w += gw;
        }
        if !current.is_empty() {
            result.push(graphemes_to_line(&current));
        }
    }
}

/// Character-wrap lines to fit within `max_width` columns.
///
/// Breaks at grapheme cluster boundaries without regard for word boundaries.
pub struct CharWrapper;

impl CharWrapper {
    pub fn wrap(lines: &[Line], max_width: u16, base_style: Style) -> Vec<Line> {
        let max = max_width as usize;
        if max == 0 {
            return vec![];
        }
        let mut result = Vec::new();

        for line in lines {
            let graphemes = line_to_graphemes(line, base_style);
            if graphemes.is_empty() {
                result.push(Line::default());
                continue;
            }

            let mut current: Vec<StyledGrapheme> = Vec::new();
            let mut w = 0usize;

            for sg in &graphemes {
                let gw = sg.width();
                if w + gw > max && !current.is_empty() {
                    result.push(graphemes_to_line(&current));
                    current.clear();
                    w = 0;
                }
                current.push(sg.clone());
                w += gw;
            }
            if !current.is_empty() {
                result.push(graphemes_to_line(&current));
            }
        }

        result
    }
}

/// Truncate lines to fit within `max_width` columns.
///
/// Each input line is shortened to at most `max_width` visible columns.
/// No new lines are created.
pub struct LineTruncator;

impl LineTruncator {
    pub fn truncate(lines: &[Line], max_width: u16, base_style: Style) -> Vec<Line> {
        let max = max_width as usize;
        let mut result = Vec::with_capacity(lines.len());

        for line in lines {
            let graphemes = line_to_graphemes(line, base_style);
            let mut kept: Vec<StyledGrapheme> = Vec::new();
            let mut w = 0usize;
            for sg in &graphemes {
                let gw = sg.width();
                if w + gw > max {
                    break;
                }
                kept.push(sg.clone());
                w += gw;
            }
            result.push(graphemes_to_line(&kept));
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::style::Color;

    fn plain_line(s: &str) -> Line {
        Line::raw(s.to_string())
    }

    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn word_wrap_simple() {
        let lines = vec![plain_line("hello world foo")];
        let wrapped = WordWrapper::wrap(&lines, 11, Style::default());
        assert_eq!(wrapped.len(), 2);
        assert_eq!(line_text(&wrapped[0]), "hello world");
        assert_eq!(line_text(&wrapped[1]), "foo");
    }

    #[test]
    fn word_wrap_long_word_falls_back_to_char() {
        let lines = vec![plain_line("abcdefghij")];
        let wrapped = WordWrapper::wrap(&lines, 4, Style::default());
        assert_eq!(wrapped.len(), 3);
        assert_eq!(line_text(&wrapped[0]), "abcd");
        assert_eq!(line_text(&wrapped[1]), "efgh");
        assert_eq!(line_text(&wrapped[2]), "ij");
    }

    #[test]
    fn word_wrap_preserves_styles() {
        let styled = Line::from(vec![
            Span::styled("hello ".to_string(), Style::default().fg(Color::Red)),
            Span::styled("world".to_string(), Style::default().fg(Color::Blue)),
        ]);
        let wrapped = WordWrapper::wrap(&[styled], 5, Style::default());
        assert_eq!(wrapped.len(), 2);
        // First line should be "hello" with Red
        assert_eq!(wrapped[0].spans[0].style.fg, Some(Color::Red));
        // Second line should be "world" with Blue
        assert_eq!(wrapped[1].spans[0].style.fg, Some(Color::Blue));
    }

    #[test]
    fn word_wrap_empty_line() {
        let lines = vec![plain_line("")];
        let wrapped = WordWrapper::wrap(&lines, 10, Style::default());
        assert_eq!(wrapped.len(), 1);
    }

    #[test]
    fn char_wrap_simple() {
        let lines = vec![plain_line("abcdefghij")];
        let wrapped = CharWrapper::wrap(&lines, 4, Style::default());
        assert_eq!(wrapped.len(), 3);
        assert_eq!(line_text(&wrapped[0]), "abcd");
        assert_eq!(line_text(&wrapped[1]), "efgh");
        assert_eq!(line_text(&wrapped[2]), "ij");
    }

    #[test]
    fn char_wrap_preserves_styles() {
        let styled = Line::from(vec![
            Span::styled("abc".to_string(), Style::default().fg(Color::Red)),
            Span::styled("def".to_string(), Style::default().fg(Color::Blue)),
        ]);
        let wrapped = CharWrapper::wrap(&[styled], 4, Style::default());
        assert_eq!(wrapped.len(), 2);
        // "abcd" — first 3 chars Red, fourth char Blue
        assert_eq!(wrapped[0].spans.len(), 2);
        assert_eq!(wrapped[0].spans[0].content.as_ref(), "abc");
        assert_eq!(wrapped[0].spans[1].content.as_ref(), "d");
    }

    #[test]
    fn truncate_clips_long_lines() {
        let lines = vec![plain_line("hello world")];
        let truncated = LineTruncator::truncate(&lines, 5, Style::default());
        assert_eq!(truncated.len(), 1);
        assert_eq!(line_text(&truncated[0]), "hello");
    }

    #[test]
    fn truncate_preserves_short_lines() {
        let lines = vec![plain_line("hi")];
        let truncated = LineTruncator::truncate(&lines, 10, Style::default());
        assert_eq!(truncated.len(), 1);
        assert_eq!(line_text(&truncated[0]), "hi");
    }
}
