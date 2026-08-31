//! Syntax highlighting: classification, cross-line state, and the invariant
//! that tokens tile a line exactly.

use hawktui::core::buffer::Buffer;
use hawktui::core::rect::Rect;
use hawktui::core::style::{Color, Style};
use hawktui::widget::editor::{Editor, EditorState};
use hawktui::widget::highlight::{
    highlight, HighlightTheme, Highlighter, Language, TokenKind, JSON, PYTHON, RUST,
};
use hawktui::widget::markdown::Markdown;
use hawktui::widget::StatefulWidget;
use hawktui::widget::Widget;

/// Every token kind present in `line`, in order, deduplicated by run.
fn kinds(hl: &mut Highlighter, line: &str) -> Vec<TokenKind> {
    hl.tokens(line).into_iter().map(|t| t.kind).collect()
}

/// The text of the first token of the given kind.
fn first_of<'a>(hl: &mut Highlighter, line: &'a str, kind: TokenKind) -> Option<&'a str> {
    hl.tokens(line)
        .into_iter()
        .find(|t| t.kind == kind)
        .map(|t| &line[t.start..t.end])
}

fn assert_tiles(hl: &mut Highlighter, line: &str) {
    let tokens = hl.tokens(line);
    let mut cursor = 0usize;
    for token in &tokens {
        assert_eq!(
            token.start, cursor,
            "gap or overlap before {token:?} in {line:?}"
        );
        assert!(token.end > token.start, "empty token {token:?} in {line:?}");
        assert!(
            line.is_char_boundary(token.start),
            "start not on a boundary"
        );
        assert!(line.is_char_boundary(token.end), "end not on a boundary");
        cursor = token.end;
    }
    assert_eq!(
        cursor,
        line.len(),
        "tokens do not reach the end of {line:?}"
    );
    let rebuilt: String = tokens.iter().map(|t| &line[t.start..t.end]).collect();
    assert_eq!(rebuilt, line, "tokens do not reconstruct {line:?}");
}

#[test]
fn tokens_tile_every_line_of_a_mixed_corpus() {
    let corpus: &[(&str, &str)] = &[
        (
            "rust",
            "#[derive(Debug)]\npub fn main() -> i32 { /* nested /* here */ */ 0x1F_u8 as i32 }\nlet s = \"escaped \\\" quote\";\n",
        ),
        (
            "python",
            "@decorator\ndef f(x: int = 3.14e-2) -> str:\n    # comment\n    return f'{x!r}'\n",
        ),
        ("json", "{\"a\": [1, 2.5, true, null], \"b\": \"\\u00e9\"}\n"),
        ("sql", "SELECT id FROM t WHERE name LIKE '%x%' -- trailing\n"),
        ("shell", "for f in *.txt; do echo \"$f\"; done # loop\n"),
        ("toml", "[package]\nname = \"hawktui\" # inline\nedition = 2021\n"),
        ("go", "func main() {\n\ts := `raw\nstring`\n}\n"),
        ("typescript", "const x: Map<string, number> = new Map();\n"),
        // Non-ASCII inside every construct, to prove byte offsets stay on
        // character boundaries.
        ("rust", "let 日本 = \"héllo 🌍\"; // コメント\n"),
        ("text", "no language rules at all — everything is plain 🌍\n"),
    ];

    for (lang, source) in corpus {
        let mut hl = Highlighter::from_name(lang).unwrap_or_else(|| panic!("no language {lang}"));
        for line in source.lines() {
            assert_tiles(&mut hl, line);
        }
    }
}

#[test]
fn rust_classifies_keywords_types_and_calls() {
    let mut hl = Highlighter::new(&RUST);
    let line = "pub fn render(area: Rect, count: u32) { helper(count); }";
    assert_eq!(first_of(&mut hl, line, TokenKind::Keyword), Some("pub"));
    assert_eq!(first_of(&mut hl, line, TokenKind::Type), Some("Rect"));
    assert_eq!(first_of(&mut hl, line, TokenKind::Function), Some("render"));
    // `u32` is a built-in type, not a bare identifier.
    let u32_token = hl
        .tokens(line)
        .into_iter()
        .find(|t| &line[t.start..t.end] == "u32")
        .unwrap();
    assert_eq!(u32_token.kind, TokenKind::Type);
}

#[test]
fn rust_recognizes_attributes_including_inner_ones() {
    let mut hl = Highlighter::new(&RUST);
    assert_eq!(
        first_of(&mut hl, "#[derive(Debug, Clone)]", TokenKind::Attribute),
        Some("#[derive(Debug, Clone)]")
    );
    hl.reset();
    assert_eq!(
        first_of(&mut hl, "#![allow(dead_code)]", TokenKind::Attribute),
        Some("#![allow(dead_code)]")
    );
}

#[test]
fn numbers_keep_their_prefix_separators_and_suffix() {
    let mut hl = Highlighter::new(&RUST);
    for (line, expected) in [
        ("let a = 0xFF_u8;", "0xFF_u8"),
        ("let b = 1_000_000;", "1_000_000"),
        ("let c = 3.14f64;", "3.14f64"),
        ("let d = 1e-9;", "1e-9"),
        ("let e = 0b1010;", "0b1010"),
    ] {
        hl.reset();
        assert_eq!(
            first_of(&mut hl, line, TokenKind::Number),
            Some(expected),
            "in {line:?}"
        );
    }
}

#[test]
fn a_dot_after_a_number_is_not_swallowed_as_a_decimal_point() {
    let mut hl = Highlighter::new(&RUST);
    let line = "let x = 1.max(2);";
    assert_eq!(first_of(&mut hl, line, TokenKind::Number), Some("1"));
    assert_eq!(first_of(&mut hl, line, TokenKind::Function), Some("max"));
}

#[test]
fn escaped_quotes_do_not_end_a_string() {
    let mut hl = Highlighter::new(&RUST);
    let line = r#"let s = "a \" b"; let t = 1;"#;
    assert_eq!(
        first_of(&mut hl, line, TokenKind::String),
        Some(r#""a \" b""#)
    );
    // The tail after the string is lexed normally, not as more string.
    assert_eq!(first_of(&mut hl, line, TokenKind::Number), Some("1"));
}

#[test]
fn block_comments_span_lines_and_nest_in_rust() {
    let mut hl = Highlighter::new(&RUST);
    let opened = kinds(&mut hl, "let a = 1; /* open");
    assert_eq!(opened.last(), Some(&TokenKind::Comment));
    assert!(!hl.state().is_clean());
    hl.reset();

    hl.tokens("/* outer /* inner");
    assert!(!hl.state().is_clean(), "two comments should still be open");
    hl.tokens("still comment */");
    assert!(!hl.state().is_clean(), "outer comment is still open");
    let after = hl.tokens("done */ let x = 1;");
    assert!(hl.state().is_clean(), "both comments closed");
    assert_eq!(after[0].kind, TokenKind::Comment);
    assert_eq!(after[0].end, "done */".len());
    assert!(after.iter().any(|t| t.kind == TokenKind::Keyword));
}

#[test]
fn a_line_comment_inside_a_block_comment_does_not_end_it() {
    let mut hl = Highlighter::new(&RUST);
    hl.tokens("/* open");
    let tokens = hl.tokens("// this is still the block comment");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Comment);
    assert!(!hl.state().is_clean());
}

#[test]
fn unterminated_strings_carry_over_only_where_the_language_allows_it() {
    let mut rust = Highlighter::new(&RUST);
    rust.tokens("let s = \"start");
    assert!(
        !rust.state().is_clean(),
        "Rust string literals may span lines"
    );

    let mut python = Highlighter::new(&PYTHON);
    python.tokens("s = 'start");
    assert!(
        python.state().is_clean(),
        "a bare Python quote does not continue past the line"
    );
}

#[test]
fn state_can_be_captured_and_restored_to_resume_mid_document() {
    let source = ["/* header", " * body", " */ fn main() {}"];

    let mut top = Highlighter::new(&RUST);
    top.tokens(source[0]);
    let saved = top.state();

    // A viewport starting at line 1 restores the state instead of re-lexing.
    let mut viewport = Highlighter::new(&RUST);
    viewport.set_state(saved);
    assert_eq!(viewport.tokens(source[1])[0].kind, TokenKind::Comment);
    assert_eq!(viewport.tokens(source[2])[0].kind, TokenKind::Comment);
    assert!(viewport.state().is_clean());
}

#[test]
fn sql_keywords_match_regardless_of_case() {
    let mut hl = Highlighter::from_name("sql").unwrap();
    let upper = hl.tokens("SELECT * FROM t")[0];
    assert_eq!(upper.kind, TokenKind::Keyword);
    hl.reset();
    let lower = hl.tokens("select * from t")[0];
    assert_eq!(lower.kind, TokenKind::Keyword);
}

#[test]
fn json_has_no_comments_and_no_single_quotes() {
    let mut hl = Highlighter::new(&JSON);
    let line = "{\"k\": 'v' // not a comment}";
    let tokens = hl.tokens(line);
    assert!(
        !tokens.iter().any(|t| t.kind == TokenKind::Comment),
        "JSON has no comment syntax"
    );
    // Only the double-quoted key is a string.
    let strings: Vec<_> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::String)
        .map(|t| &line[t.start..t.end])
        .collect();
    assert_eq!(strings, vec!["\"k\""]);
}

#[test]
fn languages_resolve_by_name_alias_and_extension() {
    assert_eq!(Language::from_name("rust").unwrap().name, "rust");
    assert_eq!(Language::from_name("rs").unwrap().name, "rust");
    assert_eq!(Language::from_name("RS").unwrap().name, "rust");
    assert_eq!(Language::from_name("main.rs").unwrap().name, "rust");
    assert_eq!(Language::from_name("c++").unwrap().name, "cpp");
    assert_eq!(Language::from_name("yml").unwrap().name, "yaml");
    assert_eq!(Language::from_name("bash").unwrap().name, "shell");
    assert!(Language::from_name("cobol").is_none());
}

#[test]
fn the_theme_maps_every_kind_and_accepts_overrides() {
    let theme = HighlightTheme::dark().set(TokenKind::Keyword, Style::default().fg(Color::Red));
    assert_eq!(theme.style(TokenKind::Keyword).fg, Some(Color::Red));
    // Every kind resolves to something, including the ones left at default.
    for kind in TokenKind::ALL {
        let _ = theme.style(kind);
    }
    assert_ne!(HighlightTheme::light(), HighlightTheme::dark());
}

#[test]
fn adjacent_runs_of_one_kind_collapse_into_a_single_token() {
    let mut hl = Highlighter::new(&RUST);
    // Whitespace and punctuation should not shatter the line into one token
    // per character.
    let line = "    let x = [1, 2, 3];";
    let tokens = hl.tokens(line);
    // The trailing `];` is two punctuation characters but one token.
    let last = tokens.last().unwrap();
    assert_eq!(last.kind, TokenKind::Punctuation);
    assert_eq!(&line[last.start..last.end], "];");
    let mut prev: Option<TokenKind> = None;
    for token in &tokens {
        assert_ne!(prev, Some(token.kind), "adjacent tokens share a kind");
        prev = Some(token.kind);
    }
}

#[test]
fn the_convenience_function_returns_one_line_per_source_line() {
    let lines = highlight("fn a() {}\nfn b() {}", "rust").unwrap();
    assert_eq!(lines.len(), 2);
    assert!(highlight("x", "cobol").is_none());
}

#[test]
fn markdown_highlights_a_fenced_block_that_names_a_language() {
    let source = "text\n```rust\nlet x = 1;\n```\n";
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 6));
    Markdown::new(source).render(Rect::new(0, 0, 30, 6), &mut buf);

    // Row 1 is the code line; `let` should carry the keyword color.
    let keyword_fg = HighlightTheme::default()
        .style(TokenKind::Keyword)
        .fg
        .unwrap();
    assert_eq!(buf[(2, 1)].symbol(), "l");
    assert_eq!(buf[(2, 1)].fg, keyword_fg);
    // The fence lines themselves are not rendered.
    assert_eq!(buf[(0, 0)].symbol(), "t");
}

#[test]
fn markdown_falls_back_to_code_style_without_a_language_or_when_disabled() {
    let plain = "```\nlet x = 1;\n```\n";
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 4));
    Markdown::new(plain).render(Rect::new(0, 0, 30, 4), &mut buf);
    assert_eq!(buf[(2, 0)].fg, Color::Green);

    let tagged = "```rust\nlet x = 1;\n```\n";
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 4));
    Markdown::new(tagged)
        .highlight(false)
        .render(Rect::new(0, 0, 30, 4), &mut buf);
    assert_eq!(buf[(2, 0)].fg, Color::Green);
}

#[test]
fn the_editor_highlights_and_clips_to_the_horizontal_viewport() {
    let mut state = EditorState::with_text("let value = 42;");
    let area = Rect::new(0, 0, 20, 1);
    let mut buf = Buffer::empty(area);
    Editor::new()
        .show_line_numbers(false)
        .syntax(&RUST)
        .render(area, &mut buf, &mut state);

    let theme = HighlightTheme::default();
    assert_eq!(buf[(0, 0)].symbol(), "l");
    assert_eq!(buf[(0, 0)].fg, theme.style(TokenKind::Keyword).fg.unwrap());
    assert_eq!(buf[(12, 0)].symbol(), "4");
    assert_eq!(buf[(12, 0)].fg, theme.style(TokenKind::Number).fg.unwrap());
}

#[test]
fn the_editor_colors_lines_below_a_block_comment_opened_off_screen() {
    let mut state = EditorState::with_text("/* opened above\nline two\nline three\nline four");
    state.cursor_row = 2;
    state.scroll_row = 2;
    let area = Rect::new(0, 0, 20, 2);
    let mut buf = Buffer::empty(area);
    Editor::new()
        .show_line_numbers(false)
        .syntax(&RUST)
        .render(area, &mut buf, &mut state);

    let comment_fg = HighlightTheme::default()
        .style(TokenKind::Comment)
        .fg
        .unwrap();
    assert_eq!(buf[(0, 0)].symbol(), "l");
    assert_eq!(
        buf[(0, 0)].fg,
        comment_fg,
        "the comment opened above the viewport still applies"
    );
}

#[test]
fn an_unhighlighted_editor_renders_exactly_as_before() {
    let text = "let value = 42;";
    let area = Rect::new(0, 0, 20, 1);

    let mut plain_state = EditorState::with_text(text);
    let mut plain = Buffer::empty(area);
    Editor::new()
        .show_line_numbers(false)
        .render(area, &mut plain, &mut plain_state);

    let mut named_state = EditorState::with_text(text);
    let mut named = Buffer::empty(area);
    Editor::new()
        .show_line_numbers(false)
        .syntax_named("no-such-language")
        .render(area, &mut named, &mut named_state);

    assert_eq!(plain, named);
}
