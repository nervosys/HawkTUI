//! Multi-line text editor widget.
//!
//! A line-based editor with cursor movement, insert/delete, viewport scrolling,
//! and full agent discoverability.

use crate::core::buffer::Buffer;
use crate::core::rect::Rect;
use crate::core::style::Style;
use crate::ontology::{
    ActionParam, ActionParamType, AgentAction, AgentCapability, Discoverable, PropertySchema,
    PropertyType, SemanticRole, WidgetSchema,
};
use crate::widget::block::Block;
use crate::widget::highlight::{HighlightTheme, Highlighter, Language};
use crate::widget::{StatefulWidget, Widget};

/// A multi-line text editor widget.
#[derive(Debug, Clone)]
pub struct Editor {
    block: Option<Block>,
    style: Style,
    cursor_style: Style,
    line_number_style: Style,
    show_line_numbers: bool,
    syntax: Option<&'static Language>,
    highlight_theme: HighlightTheme,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            block: None,
            style: Style::default(),
            cursor_style: Style::default().reversed(),
            line_number_style: Style::default().dim(),
            show_line_numbers: true,
            syntax: None,
            highlight_theme: HighlightTheme::default(),
        }
    }

    /// Syntax-highlight the buffer as `language`.
    ///
    /// ```
    /// use hawktui::widget::editor::Editor;
    /// use hawktui::widget::highlight::RUST;
    ///
    /// let editor = Editor::new().syntax(&RUST);
    /// ```
    pub fn syntax(mut self, language: &'static Language) -> Self {
        self.syntax = Some(language);
        self
    }

    /// Syntax-highlight by language name, alias, or file extension.
    ///
    /// An unrecognized name leaves highlighting off rather than failing, so a
    /// file browser can pass an arbitrary extension.
    pub fn syntax_named(mut self, name: &str) -> Self {
        self.syntax = Language::from_name(name);
        self
    }

    /// Palette for syntax highlighting.
    pub fn highlight_theme(mut self, theme: HighlightTheme) -> Self {
        self.highlight_theme = theme;
        self
    }

    pub fn block(mut self, block: Block) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn cursor_style(mut self, style: Style) -> Self {
        self.cursor_style = style;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

/// State for an [`Editor`] widget.
#[derive(Debug, Clone, Default)]
pub struct EditorState {
    /// Lines of text.
    pub lines: Vec<String>,
    /// Cursor row (line index).
    pub cursor_row: usize,
    /// Cursor column (byte offset within the line).
    pub cursor_col: usize,
    /// Vertical scroll offset (first visible line).
    pub scroll_row: usize,
    /// Horizontal scroll offset.
    pub scroll_col: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            ..Default::default()
        }
    }

    pub fn with_text(text: &str) -> Self {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        let lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        Self {
            lines,
            ..Default::default()
        }
    }

    /// Get the full text content.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, ch: char) {
        if ch == '\n' {
            self.insert_newline();
            return;
        }
        let line = &mut self.lines[self.cursor_row];
        let col = self.cursor_col.min(line.len());
        line.insert(col, ch);
        self.cursor_col = col + ch.len_utf8();
    }

    /// Insert a newline, splitting the current line.
    pub fn insert_newline(&mut self) {
        let line = &mut self.lines[self.cursor_row];
        let col = self.cursor_col.min(line.len());
        let rest = line[col..].to_string();
        line.truncate(col);
        self.cursor_row += 1;
        self.lines.insert(self.cursor_row, rest);
        self.cursor_col = 0;
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_before(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_row];
            let prev = line[..self.cursor_col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            line.remove(prev);
            self.cursor_col = prev;
        } else if self.cursor_row > 0 {
            // Merge with previous line
            let current = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
            self.lines[self.cursor_row].push_str(&current);
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_after(&mut self) {
        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            self.lines[self.cursor_row].remove(self.cursor_col);
        } else if self.cursor_row + 1 < self.lines.len() {
            // Merge next line into this one
            let next = self.lines.remove(self.cursor_row + 1);
            self.lines[self.cursor_row].push_str(&next);
        }
    }

    /// Move cursor left.
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            let line = &self.lines[self.cursor_row];
            self.cursor_col = line[..self.cursor_col]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.lines[self.cursor_row].len();
        }
    }

    /// Move cursor right.
    pub fn move_right(&mut self) {
        let line = &self.lines[self.cursor_row];
        if self.cursor_col < line.len() {
            self.cursor_col = line[self.cursor_col..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_col + i)
                .unwrap_or(line.len());
        } else if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    /// Move cursor up.
    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor down.
    pub fn move_down(&mut self) {
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_row].len());
        }
    }

    /// Move cursor to start of line.
    pub fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    /// Move cursor to end of line.
    pub fn move_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_row].len();
    }

    fn ensure_cursor_visible(&mut self, visible_rows: usize, visible_cols: usize) {
        // Vertical scrolling
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        } else if self.cursor_row >= self.scroll_row + visible_rows {
            self.scroll_row = self.cursor_row - visible_rows + 1;
        }
        // Horizontal scrolling
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        } else if self.cursor_col >= self.scroll_col + visible_cols {
            self.scroll_col = self.cursor_col - visible_cols + 1;
        }
    }
}

impl StatefulWidget for Editor {
    type State = EditorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut EditorState) {
        if area.is_empty() {
            return;
        }

        buf.set_style(area, self.style);

        let inner = if let Some(block) = self.block {
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner.is_empty() {
            return;
        }

        let gutter_width: u16 = if self.show_line_numbers {
            let max_line = state.lines.len();
            let digits = format!("{max_line}").len() as u16;
            digits + 1 // digits + space
        } else {
            0
        };

        let text_width = inner.width.saturating_sub(gutter_width) as usize;
        let visible_rows = inner.height as usize;

        state.ensure_cursor_visible(visible_rows, text_width);

        // Highlighting state runs forward from the top of the document, so the
        // lines scrolled off the top still have to be lexed — a block comment
        // opened above the viewport colors what is visible below it.
        let theme = self.highlight_theme;
        let mut highlighter = self.syntax.map(|language| {
            let mut hl = Highlighter::new(language).theme(theme);
            for prior in state.lines.iter().take(state.scroll_row) {
                hl.tokens(prior);
            }
            hl
        });

        for row_offset in 0..visible_rows {
            let line_idx = state.scroll_row + row_offset;
            let y = inner.y + row_offset as u16;

            if line_idx >= state.lines.len() {
                // Draw tilde for empty lines beyond content
                buf.set_string(inner.x, y, "~", self.line_number_style);
                continue;
            }

            // Line numbers
            if self.show_line_numbers {
                let num = format!(
                    "{:>width$} ",
                    line_idx + 1,
                    width = (gutter_width - 1) as usize
                );
                buf.set_string(inner.x, y, &num, self.line_number_style);
            }

            // Line content
            let line = &state.lines[line_idx];
            let text_x = inner.x + gutter_width;
            if let Some(hl) = highlighter.as_mut() {
                let tokens = hl.tokens(line);
                let mut skipped = 0usize;
                let mut taken = 0usize;
                let mut x = text_x;
                for token in tokens {
                    // Clip each token to the horizontal viewport, counting in
                    // characters to match the unhighlighted path exactly.
                    let mut piece = String::new();
                    for ch in token.text(line).chars() {
                        if skipped < state.scroll_col {
                            skipped += 1;
                            continue;
                        }
                        if taken >= text_width {
                            break;
                        }
                        taken += 1;
                        piece.push(ch);
                    }
                    if !piece.is_empty() {
                        let style = self.style.patch(theme.style(token.kind));
                        buf.set_string(x, y, &piece, style);
                        x += piece.chars().count() as u16;
                    }
                    if taken >= text_width {
                        break;
                    }
                }
            } else {
                let visible: String = line
                    .chars()
                    .skip(state.scroll_col)
                    .take(text_width)
                    .collect();
                buf.set_string(text_x, y, &visible, self.style);
            }

            // Cursor
            if line_idx == state.cursor_row {
                let cursor_display_col = state.cursor_col.saturating_sub(state.scroll_col);
                if cursor_display_col < text_width {
                    let cx = text_x + cursor_display_col as u16;
                    buf.set_style(Rect::new(cx, y, 1, 1), self.cursor_style);
                }
            }
        }
    }
}

impl Discoverable for Editor {
    fn schema() -> WidgetSchema {
        WidgetSchema {
            name: "Editor".into(),
            description:
                "A multi-line text editor with cursor movement, line editing, and scrolling.".into(),
            default_role: SemanticRole::Input,
            properties: vec![PropertySchema {
                name: "show_line_numbers".into(),
                description: "Whether to display line numbers in the gutter.".into(),
                property_type: PropertyType::Boolean,
                required: false,
                default_value: Some(serde_json::json!(true)),
                constraints: vec![],
            }],
            actions: vec![],

            usage_hint: Some("Editor::new().show_line_numbers(true)".into()),
            tags: vec![
                "editor".into(),
                "text".into(),
                "input".into(),
                "multiline".into(),
                "code".into(),
            ],
        }
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::Focusable,
            AgentCapability::TextInput {
                multiline: true,
                max_length: None,
            },
            AgentCapability::Scrollable {
                vertical: true,
                horizontal: true,
            },
            AgentCapability::Copyable,
            AgentCapability::HasKeyBindings {
                bindings: vec![
                    ("Up".into(), "Move cursor up".into()),
                    ("Down".into(), "Move cursor down".into()),
                    ("Left".into(), "Move cursor left".into()),
                    ("Right".into(), "Move cursor right".into()),
                    ("Home".into(), "Move to start of line".into()),
                    ("End".into(), "Move to end of line".into()),
                    ("Enter".into(), "Insert newline".into()),
                    ("Backspace".into(), "Delete char before cursor".into()),
                    ("Delete".into(), "Delete char at cursor".into()),
                ],
            },
        ]
    }

    fn actions(&self) -> Vec<AgentAction> {
        vec![
            AgentAction {
                name: "set_text".into(),
                description: "Replace the entire editor content.".into(),
                params: vec![ActionParam {
                    name: "text".into(),
                    description: "The text to set.".into(),
                    param_type: ActionParamType::String,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "get_text".into(),
                description: "Get the full editor text content.".into(),
                params: vec![],
                returns: Some("The editor text as a string.".into()),
                mutates: false,
                idempotent: true,
                shortcut: None,
            },
            AgentAction {
                name: "insert_text".into(),
                description: "Insert text at the current cursor position.".into(),
                params: vec![ActionParam {
                    name: "text".into(),
                    description: "The text to insert.".into(),
                    param_type: ActionParamType::String,
                    required: true,
                    default_value: None,
                }],
                returns: None,
                mutates: true,
                idempotent: false,
                shortcut: None,
            },
        ]
    }

    fn semantic_role(&self) -> SemanticRole {
        SemanticRole::Input
    }

    fn agent_state(&self) -> serde_json::Value {
        serde_json::json!({
            "widget": "Editor",
        })
    }

    fn execute_action(
        &mut self,
        action: &str,
        _params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        Err(format!(
            "Editor actions require EditorState; use stateful dispatch. Action: {action}"
        ))
    }
}
