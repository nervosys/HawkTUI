//! Dependency-free syntax highlighting.
//!
//! A small, allocation-light lexer that turns a line of source into styled
//! [`Span`]s. It is deliberately *lexical*: it knows quotes, comments, numbers,
//! and keyword tables, not grammar. That is enough to read code in a terminal
//! and cheap enough to run on every frame, which a parser-backed highlighter is
//! not.
//!
//! State that crosses line boundaries — an open block comment, an unterminated
//! multi-line string — lives in [`HighlightState`], so a viewport can start
//! highlighting at any line as long as it carries the state forward.
//!
//! ```
//! use hawktui::widget::highlight::Highlighter;
//!
//! let mut hl = Highlighter::from_name("rust").unwrap();
//! let lines = hl.lines("// greet\nfn main() { println!(\"hi\"); }");
//! assert_eq!(lines.len(), 2);
//! ```

use crate::core::style::{Color, Style};
use crate::core::text::{Line, Span};

/// The lexical class a highlighter assigns to a run of source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// Anything the lexer has no opinion about, including whitespace.
    Text,
    /// A reserved word.
    Keyword,
    /// A built-in or conventionally-capitalized type name.
    Type,
    /// A language constant: `true`, `false`, `null`, `None`.
    Literal,
    /// An identifier immediately followed by `(`.
    Function,
    /// A quoted string, including its quotes.
    String,
    /// A numeric literal with its prefix, separators, and suffix.
    Number,
    /// A line or block comment, including its delimiters.
    Comment,
    /// An attribute, annotation, or decorator.
    Attribute,
    /// An operator character run.
    Operator,
    /// Brackets, commas, semicolons.
    Punctuation,
}

impl TokenKind {
    /// Every kind, in theme-table order.
    pub const ALL: [TokenKind; Self::COUNT] = [
        Self::Text,
        Self::Keyword,
        Self::Type,
        Self::Literal,
        Self::Function,
        Self::String,
        Self::Number,
        Self::Comment,
        Self::Attribute,
        Self::Operator,
        Self::Punctuation,
    ];

    /// Number of distinct kinds.
    pub const COUNT: usize = 11;

    const fn index(self) -> usize {
        match self {
            Self::Text => 0,
            Self::Keyword => 1,
            Self::Type => 2,
            Self::Literal => 3,
            Self::Function => 4,
            Self::String => 5,
            Self::Number => 6,
            Self::Comment => 7,
            Self::Attribute => 8,
            Self::Operator => 9,
            Self::Punctuation => 10,
        }
    }
}

/// A classified byte range within one line.
///
/// Ranges are byte offsets into the line that produced them, are non-empty,
/// and tile the line completely with no gaps and no overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub kind: TokenKind,
}

impl Token {
    /// The slice of `line` this token covers.
    pub fn text<'a>(&self, line: &'a str) -> &'a str {
        &line[self.start..self.end]
    }
}

/// Lexer state that survives a line break.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HighlightState {
    /// Nesting depth of open block comments; 0 when none is open.
    block_comment_depth: u16,
    /// Quote character of a string left open at the end of the previous line.
    string: Option<char>,
}

impl HighlightState {
    /// True when no construct spans into the next line, so highlighting may
    /// restart from here without reading anything before it.
    pub fn is_clean(&self) -> bool {
        self.block_comment_depth == 0 && self.string.is_none()
    }
}

/// The lexical rules of one language.
///
/// Constructed only by this module; pick one with [`Language::from_name`] or
/// the constants below.
#[derive(Debug, Clone, Copy)]
pub struct Language {
    /// Canonical name, as it would appear after a Markdown fence.
    pub name: &'static str,
    /// Other names and file extensions that select this language.
    pub aliases: &'static [&'static str],
    keywords: &'static [&'static str],
    types: &'static [&'static str],
    literals: &'static [&'static str],
    line_comments: &'static [&'static str],
    block_comment: Option<(&'static str, &'static str)>,
    nested_block_comments: bool,
    quotes: &'static [char],
    multiline_strings: bool,
    escape: Option<char>,
    attribute_prefixes: &'static [&'static str],
    ident_extra: &'static [char],
    uppercase_is_type: bool,
}

/// Neutral defaults every language overrides in part.
const BASE: Language = Language {
    name: "text",
    aliases: &[],
    keywords: &[],
    types: &[],
    literals: &[],
    line_comments: &[],
    block_comment: None,
    nested_block_comments: false,
    quotes: &[],
    multiline_strings: false,
    escape: Some('\\'),
    attribute_prefixes: &[],
    ident_extra: &[],
    uppercase_is_type: false,
};

/// Plain text: every character is [`TokenKind::Text`].
pub const PLAIN: Language = Language {
    name: "text",
    aliases: &["txt", "plain", "plaintext"],
    ..BASE
};

/// Rust.
pub const RUST: Language = Language {
    name: "rust",
    aliases: &["rs"],
    keywords: &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "type",
        "union", "unsafe", "use", "where", "while", "yield",
    ],
    types: &[
        "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "str", "u8",
        "u16", "u32", "u64", "u128", "usize",
    ],
    literals: &["true", "false", "None", "Some", "Ok", "Err"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    nested_block_comments: true,
    quotes: &['"'],
    multiline_strings: true,
    attribute_prefixes: &["#![", "#["],
    uppercase_is_type: true,
    ..BASE
};

/// Python.
pub const PYTHON: Language = Language {
    name: "python",
    aliases: &["py", "python3"],
    keywords: &[
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
        "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with",
        "yield", "match", "case",
    ],
    types: &[
        "bool",
        "bytes",
        "dict",
        "float",
        "frozenset",
        "int",
        "list",
        "object",
        "set",
        "str",
        "tuple",
        "type",
    ],
    literals: &["True", "False", "None", "self", "cls"],
    line_comments: &["#"],
    quotes: &['"', '\''],
    attribute_prefixes: &["@"],
    ..BASE
};

/// JavaScript.
pub const JAVASCRIPT: Language = Language {
    name: "javascript",
    aliases: &["js", "jsx", "mjs", "cjs"],
    keywords: &[
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "get",
        "if",
        "import",
        "in",
        "instanceof",
        "let",
        "new",
        "of",
        "return",
        "set",
        "static",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ],
    types: &[
        "Array", "BigInt", "Boolean", "Date", "Error", "Map", "Number", "Object", "Promise",
        "RegExp", "Set", "String", "Symbol",
    ],
    literals: &["true", "false", "null", "undefined", "NaN", "Infinity"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\'', '`'],
    multiline_strings: true,
    ..BASE
};

/// TypeScript.
pub const TYPESCRIPT: Language = Language {
    name: "typescript",
    aliases: &["ts", "tsx"],
    keywords: &[
        "abstract",
        "as",
        "async",
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "declare",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "get",
        "if",
        "implements",
        "import",
        "in",
        "infer",
        "instanceof",
        "interface",
        "keyof",
        "let",
        "namespace",
        "new",
        "of",
        "private",
        "protected",
        "public",
        "readonly",
        "return",
        "satisfies",
        "set",
        "static",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "type",
        "typeof",
        "var",
        "void",
        "while",
        "yield",
    ],
    types: &[
        "any", "bigint", "boolean", "never", "number", "object", "string", "symbol", "unknown",
        "Array", "Date", "Map", "Promise", "Record", "Set",
    ],
    literals: &["true", "false", "null", "undefined"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\'', '`'],
    multiline_strings: true,
    ..BASE
};

/// Go.
pub const GO: Language = Language {
    name: "go",
    aliases: &["golang"],
    keywords: &[
        "break",
        "case",
        "chan",
        "const",
        "continue",
        "default",
        "defer",
        "else",
        "fallthrough",
        "for",
        "func",
        "go",
        "goto",
        "if",
        "import",
        "interface",
        "map",
        "package",
        "range",
        "return",
        "select",
        "struct",
        "switch",
        "type",
        "var",
    ],
    types: &[
        "bool",
        "byte",
        "complex64",
        "complex128",
        "error",
        "float32",
        "float64",
        "int",
        "int8",
        "int16",
        "int32",
        "int64",
        "rune",
        "string",
        "uint",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "uintptr",
    ],
    literals: &["true", "false", "nil", "iota"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '`'],
    multiline_strings: true,
    ..BASE
};

/// C.
pub const C: Language = Language {
    name: "c",
    aliases: &["h"],
    keywords: &[
        "auto", "break", "case", "const", "continue", "default", "do", "else", "enum", "extern",
        "for", "goto", "if", "inline", "register", "restrict", "return", "sizeof", "static",
        "struct", "switch", "typedef", "union", "volatile", "while",
    ],
    types: &[
        "bool", "char", "double", "float", "int", "long", "short", "signed", "size_t", "unsigned",
        "void",
    ],
    literals: &["true", "false", "NULL"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\''],
    attribute_prefixes: &["#"],
    ..BASE
};

/// C++.
pub const CPP: Language = Language {
    name: "cpp",
    aliases: &["c++", "cc", "cxx", "hpp", "hxx"],
    keywords: &[
        "auto",
        "break",
        "case",
        "catch",
        "class",
        "co_await",
        "co_return",
        "co_yield",
        "const",
        "constexpr",
        "consteval",
        "continue",
        "decltype",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "explicit",
        "export",
        "extern",
        "for",
        "friend",
        "goto",
        "if",
        "inline",
        "mutable",
        "namespace",
        "new",
        "noexcept",
        "operator",
        "override",
        "private",
        "protected",
        "public",
        "return",
        "sizeof",
        "static",
        "static_cast",
        "struct",
        "switch",
        "template",
        "this",
        "throw",
        "try",
        "typedef",
        "typename",
        "union",
        "using",
        "virtual",
        "volatile",
        "while",
    ],
    types: &[
        "bool",
        "char",
        "double",
        "float",
        "int",
        "long",
        "short",
        "signed",
        "size_t",
        "std::string",
        "unsigned",
        "void",
        "wchar_t",
    ],
    literals: &["true", "false", "nullptr", "NULL"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\''],
    attribute_prefixes: &["#"],
    ..BASE
};

/// Java.
pub const JAVA: Language = Language {
    name: "java",
    aliases: &[],
    keywords: &[
        "abstract",
        "assert",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "default",
        "do",
        "else",
        "enum",
        "extends",
        "final",
        "finally",
        "for",
        "goto",
        "if",
        "implements",
        "import",
        "instanceof",
        "interface",
        "native",
        "new",
        "package",
        "private",
        "protected",
        "public",
        "record",
        "return",
        "sealed",
        "static",
        "strictfp",
        "super",
        "switch",
        "synchronized",
        "this",
        "throw",
        "throws",
        "transient",
        "try",
        "var",
        "volatile",
        "while",
        "yield",
    ],
    types: &[
        "boolean", "byte", "char", "double", "float", "int", "long", "short", "void", "String",
        "Integer", "Long", "Double", "Boolean", "Object", "List", "Map", "Set",
    ],
    literals: &["true", "false", "null"],
    line_comments: &["//"],
    block_comment: Some(("/*", "*/")),
    quotes: &['"', '\''],
    attribute_prefixes: &["@"],
    ..BASE
};

/// JSON. No comments, no single quotes — deliberately strict.
pub const JSON: Language = Language {
    name: "json",
    aliases: &[],
    literals: &["true", "false", "null"],
    quotes: &['"'],
    ..BASE
};

/// TOML.
pub const TOML: Language = Language {
    name: "toml",
    aliases: &[],
    literals: &["true", "false"],
    line_comments: &["#"],
    quotes: &['"', '\''],
    ident_extra: &['-'],
    ..BASE
};

/// YAML.
pub const YAML: Language = Language {
    name: "yaml",
    aliases: &["yml"],
    literals: &["true", "false", "null", "yes", "no", "~"],
    line_comments: &["#"],
    quotes: &['"', '\''],
    ident_extra: &['-'],
    ..BASE
};

/// POSIX shell and Bash.
pub const SHELL: Language = Language {
    name: "shell",
    aliases: &["sh", "bash", "zsh"],
    keywords: &[
        "case", "do", "done", "elif", "else", "esac", "export", "fi", "for", "function", "if",
        "in", "local", "readonly", "return", "select", "then", "until", "while",
    ],
    types: &[
        "cat", "cd", "cp", "echo", "grep", "ls", "mkdir", "mv", "printf", "rm", "sed", "set",
        "test", "unset",
    ],
    literals: &["true", "false"],
    line_comments: &["#"],
    quotes: &['"', '\''],
    ..BASE
};

/// SQL. Keywords are matched case-insensitively.
pub const SQL: Language = Language {
    name: "sql",
    aliases: &[],
    keywords: &[
        "alter", "and", "as", "asc", "between", "by", "case", "create", "delete", "desc",
        "distinct", "drop", "else", "end", "exists", "from", "group", "having", "in", "index",
        "inner", "insert", "into", "join", "left", "like", "limit", "not", "offset", "on", "or",
        "order", "outer", "right", "select", "set", "table", "then", "union", "update", "values",
        "view", "when", "where", "with",
    ],
    types: &[
        "bigint",
        "blob",
        "boolean",
        "date",
        "decimal",
        "double",
        "float",
        "int",
        "integer",
        "numeric",
        "real",
        "smallint",
        "text",
        "timestamp",
        "varchar",
    ],
    literals: &["true", "false", "null"],
    line_comments: &["--"],
    block_comment: Some(("/*", "*/")),
    quotes: &['\'', '"'],
    ..BASE
};

/// Every language this module can highlight.
pub const LANGUAGES: &[&Language] = &[
    &RUST,
    &PYTHON,
    &JAVASCRIPT,
    &TYPESCRIPT,
    &GO,
    &C,
    &CPP,
    &JAVA,
    &JSON,
    &TOML,
    &YAML,
    &SHELL,
    &SQL,
    &PLAIN,
];

impl Language {
    /// Look up a language by name, alias, or file extension, ignoring case.
    ///
    /// ```
    /// use hawktui::widget::highlight::Language;
    ///
    /// assert_eq!(Language::from_name("RS").unwrap().name, "rust");
    /// assert_eq!(Language::from_name("c++").unwrap().name, "cpp");
    /// assert!(Language::from_name("cobol").is_none());
    /// ```
    pub fn from_name(name: &str) -> Option<&'static Language> {
        let name = name.trim();
        // An extension is as good as a name: "main.rs" and "rs" both work.
        let stem = name.rsplit('.').next().unwrap_or(name);
        LANGUAGES.iter().copied().find(|lang| {
            lang.name.eq_ignore_ascii_case(name)
                || lang.name.eq_ignore_ascii_case(stem)
                || lang
                    .aliases
                    .iter()
                    .any(|a| a.eq_ignore_ascii_case(name) || a.eq_ignore_ascii_case(stem))
        })
    }

    /// True when SQL-style case-insensitive keyword matching applies.
    fn case_insensitive_keywords(&self) -> bool {
        self.name == "sql"
    }

    fn is_ident_start(&self, ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || ch == '$'
    }

    fn is_ident_continue(&self, ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || ch == '$' || self.ident_extra.contains(&ch)
    }

    fn matches(&self, table: &[&str], word: &str) -> bool {
        if self.case_insensitive_keywords() {
            table.iter().any(|w| w.eq_ignore_ascii_case(word))
        } else {
            table.contains(&word)
        }
    }
}

/// Which [`Style`] each [`TokenKind`] renders with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighlightTheme {
    styles: [Style; TokenKind::COUNT],
}

impl HighlightTheme {
    /// Palette for dark terminals.
    pub const fn dark() -> Self {
        Self {
            styles: [
                Style::new(),                              // Text
                Style::new().fg(Color::Magenta),           // Keyword
                Style::new().fg(Color::Cyan),              // Type
                Style::new().fg(Color::LightYellow),       // Literal
                Style::new().fg(Color::Blue),              // Function
                Style::new().fg(Color::Green),             // String
                Style::new().fg(Color::LightMagenta),      // Number
                Style::new().fg(Color::DarkGray).italic(), // Comment
                Style::new().fg(Color::Yellow),            // Attribute
                Style::new().fg(Color::LightRed),          // Operator
                Style::new().fg(Color::Gray),              // Punctuation
            ],
        }
    }

    /// Palette for light terminals.
    pub const fn light() -> Self {
        Self {
            styles: [
                Style::new(),
                Style::new().fg(Color::Magenta).bold(),
                Style::new().fg(Color::Blue),
                Style::new().fg(Color::Red),
                Style::new().fg(Color::Blue).bold(),
                Style::new().fg(Color::Green),
                Style::new().fg(Color::Magenta),
                Style::new().fg(Color::Gray).italic(),
                Style::new().fg(Color::Yellow),
                Style::new().fg(Color::Red),
                Style::new().fg(Color::DarkGray),
            ],
        }
    }

    /// Override the style for one kind.
    pub fn set(mut self, kind: TokenKind, style: Style) -> Self {
        self.styles[kind.index()] = style;
        self
    }

    /// The style for one kind.
    pub fn style(&self, kind: TokenKind) -> Style {
        self.styles[kind.index()]
    }
}

impl Default for HighlightTheme {
    fn default() -> Self {
        Self::dark()
    }
}

/// A reusable highlighter for one language.
///
/// The highlighter carries [`HighlightState`] between calls, so lines must be
/// fed in order. Call [`reset`](Self::reset) before restarting at the top of a
/// document.
#[derive(Debug, Clone)]
pub struct Highlighter {
    language: &'static Language,
    theme: HighlightTheme,
    state: HighlightState,
}

impl Highlighter {
    /// Highlight with the given language.
    pub fn new(language: &'static Language) -> Self {
        Self {
            language,
            theme: HighlightTheme::default(),
            state: HighlightState::default(),
        }
    }

    /// Highlight with the language named by `name`, if one is known.
    pub fn from_name(name: &str) -> Option<Self> {
        Language::from_name(name).map(Self::new)
    }

    /// Use a different palette.
    pub fn theme(mut self, theme: HighlightTheme) -> Self {
        self.theme = theme;
        self
    }

    /// The language being highlighted.
    pub fn language(&self) -> &'static Language {
        self.language
    }

    /// The carried-over state, for a viewport that resumes mid-document.
    pub fn state(&self) -> HighlightState {
        self.state
    }

    /// Restore carried-over state captured earlier by [`state`](Self::state).
    pub fn set_state(&mut self, state: HighlightState) {
        self.state = state;
    }

    /// Forget any open comment or string, as if starting a new document.
    pub fn reset(&mut self) {
        self.state = HighlightState::default();
    }

    /// Classify one line, advancing the carried-over state.
    ///
    /// The returned tokens tile the line exactly: they are in order, non-empty,
    /// and together cover every byte.
    pub fn tokens(&mut self, line: &str) -> Vec<Token> {
        let mut out = Vec::new();
        tokenize(self.language, line, &mut self.state, &mut out);
        out
    }

    /// Classify one line and style it.
    pub fn spans(&mut self, line: &str) -> Vec<Span> {
        let theme = self.theme;
        self.tokens(line)
            .into_iter()
            .map(|t| Span::styled(t.text(line).to_string(), theme.style(t.kind)))
            .collect()
    }

    /// Highlight a whole document, resetting state first.
    pub fn lines(&mut self, source: &str) -> Vec<Line> {
        self.reset();
        source
            .lines()
            .map(|l| Line {
                spans: self.spans(l),
                alignment: None,
            })
            .collect()
    }
}

/// Highlight `source` as `language`, or return `None` if the name is unknown.
///
/// ```
/// use hawktui::widget::highlight::highlight;
///
/// let lines = highlight("SELECT 1;", "sql").unwrap();
/// assert_eq!(lines.len(), 1);
/// ```
pub fn highlight(source: &str, language: &str) -> Option<Vec<Line>> {
    Highlighter::from_name(language).map(|mut hl| hl.lines(source))
}

// ─────────────────────────────────────────────────────────────── the lexer ──

fn char_at(line: &str, i: usize) -> Option<char> {
    line[i..].chars().next()
}

/// Consume an already-open block comment, returning the index just past it.
fn consume_block_comment(
    lang: &Language,
    line: &str,
    mut i: usize,
    state: &mut HighlightState,
) -> usize {
    let Some((open, close)) = lang.block_comment else {
        state.block_comment_depth = 0;
        return line.len();
    };
    while i < line.len() {
        if line[i..].starts_with(close) {
            i += close.len();
            state.block_comment_depth = state.block_comment_depth.saturating_sub(1);
            if state.block_comment_depth == 0 {
                return i;
            }
            continue;
        }
        if lang.nested_block_comments && line[i..].starts_with(open) {
            i += open.len();
            state.block_comment_depth = state.block_comment_depth.saturating_add(1);
            continue;
        }
        i += char_at(line, i).map_or(1, char::len_utf8);
    }
    i
}

/// Consume a string body starting just past its opening quote.
fn consume_string(
    lang: &Language,
    line: &str,
    mut i: usize,
    quote: char,
    state: &mut HighlightState,
) -> usize {
    while i < line.len() {
        let ch = match char_at(line, i) {
            Some(c) => c,
            None => break,
        };
        if Some(ch) == lang.escape {
            i += ch.len_utf8();
            // A trailing backslash escapes the newline, not a character.
            if let Some(next) = char_at(line, i) {
                i += next.len_utf8();
            }
            continue;
        }
        i += ch.len_utf8();
        if ch == quote {
            state.string = None;
            return i;
        }
    }
    // Unterminated at end of line: only some languages let it continue.
    state.string = if lang.multiline_strings {
        Some(quote)
    } else {
        None
    };
    i
}

/// Consume a numeric literal, including base prefix, separators, and suffix.
fn consume_number(line: &str, mut i: usize) -> usize {
    let start = i;
    let radix_prefixed = line[i..].len() >= 2
        && line.as_bytes()[i] == b'0'
        && matches!(
            line.as_bytes()[i + 1],
            b'x' | b'X' | b'b' | b'B' | b'o' | b'O'
        );
    if radix_prefixed {
        i += 2;
        while let Some(ch) = char_at(line, i) {
            if ch.is_ascii_hexdigit() || ch == '_' {
                i += ch.len_utf8();
            } else {
                break;
            }
        }
    } else {
        let mut seen_dot = false;
        while let Some(ch) = char_at(line, i) {
            if ch.is_ascii_digit() || ch == '_' {
                i += ch.len_utf8();
            } else if ch == '.'
                && !seen_dot
                && matches!(char_at(line, i + 1), Some(d) if d.is_ascii_digit())
            {
                seen_dot = true;
                i += 1;
            } else if matches!(ch, 'e' | 'E')
                && i > start
                && matches!(char_at(line, i + 1), Some(n) if n.is_ascii_digit() || n == '+' || n == '-')
            {
                i += 1 + char_at(line, i + 1).map_or(0, char::len_utf8);
            } else {
                break;
            }
        }
    }
    // Type suffix: 1u32, 2.0f64, 3L, 4px.
    while let Some(ch) = char_at(line, i) {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    i
}

/// Consume an attribute introduced by `prefix`.
fn consume_attribute(lang: &Language, line: &str, start: usize, prefix: &str) -> usize {
    let mut i = start + prefix.len();
    if prefix.ends_with('[') {
        // Bracketed: run to the matching close bracket, or to end of line.
        let mut depth = 1usize;
        while i < line.len() && depth > 0 {
            match line.as_bytes()[i] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                _ => {}
            }
            i += char_at(line, i).map_or(1, char::len_utf8);
        }
        return i;
    }
    // Bare: the identifier (and dotted path) that follows.
    while let Some(ch) = char_at(line, i) {
        if lang.is_ident_continue(ch) || ch == '.' {
            i += ch.len_utf8();
        } else {
            break;
        }
    }
    i
}

/// True when the next non-space character at or after `i` is `(`.
fn call_follows(line: &str, mut i: usize) -> bool {
    while let Some(ch) = char_at(line, i) {
        if ch == ' ' || ch == '\t' {
            i += ch.len_utf8();
        } else {
            return ch == '(';
        }
    }
    false
}

fn tokenize(lang: &Language, line: &str, state: &mut HighlightState, out: &mut Vec<Token>) {
    let push = |start: usize, end: usize, kind: TokenKind, out: &mut Vec<Token>| {
        if end <= start {
            return;
        }
        // Adjacent runs of the same class become one token, so whitespace and
        // punctuation do not fragment a line into hundreds of spans.
        if let Some(last) = out.last_mut() {
            if last.kind == kind && last.end == start {
                last.end = end;
                return;
            }
        }
        out.push(Token { start, end, kind });
    };

    let mut i = 0usize;

    // Resume whatever the previous line left open.
    if state.block_comment_depth > 0 {
        let end = consume_block_comment(lang, line, i, state);
        push(i, end, TokenKind::Comment, out);
        i = end;
    }
    if let Some(quote) = state.string.take() {
        let end = consume_string(lang, line, i, quote, state);
        push(i, end, TokenKind::String, out);
        i = end;
    }

    while i < line.len() {
        let rest = &line[i..];
        let ch = match char_at(line, i) {
            Some(c) => c,
            None => break,
        };

        // Line comment: the rest of the line, whatever it contains.
        if lang.line_comments.iter().any(|m| rest.starts_with(*m)) {
            push(i, line.len(), TokenKind::Comment, out);
            i = line.len();
            continue;
        }

        // Block comment.
        if let Some((open, _)) = lang.block_comment {
            if rest.starts_with(open) {
                state.block_comment_depth = 1;
                let end = consume_block_comment(lang, line, i + open.len(), state);
                push(i, end, TokenKind::Comment, out);
                i = end;
                continue;
            }
        }

        // Attribute, annotation, or preprocessor directive.
        if let Some(prefix) = lang
            .attribute_prefixes
            .iter()
            .find(|p| rest.starts_with(**p))
        {
            let end = consume_attribute(lang, line, i, prefix);
            push(i, end, TokenKind::Attribute, out);
            i = end;
            continue;
        }

        // String.
        if lang.quotes.contains(&ch) {
            let end = consume_string(lang, line, i + ch.len_utf8(), ch, state);
            push(i, end, TokenKind::String, out);
            i = end;
            continue;
        }

        // Number.
        if ch.is_ascii_digit() {
            let end = consume_number(line, i);
            push(i, end, TokenKind::Number, out);
            i = end;
            continue;
        }

        // Identifier, keyword, type, literal, or call.
        if lang.is_ident_start(ch) {
            let start = i;
            i += ch.len_utf8();
            while let Some(next) = char_at(line, i) {
                if lang.is_ident_continue(next) {
                    i += next.len_utf8();
                } else {
                    break;
                }
            }
            let word = &line[start..i];
            let kind = if lang.matches(lang.literals, word) {
                TokenKind::Literal
            } else if lang.matches(lang.keywords, word) {
                TokenKind::Keyword
            } else if lang.matches(lang.types, word)
                || (lang.uppercase_is_type && word.starts_with(char::is_uppercase))
            {
                TokenKind::Type
            } else if call_follows(line, i) {
                TokenKind::Function
            } else {
                TokenKind::Text
            };
            push(start, i, kind, out);
            continue;
        }

        // Everything else: whitespace, punctuation, operators.
        let kind = if ch.is_whitespace() {
            TokenKind::Text
        } else if matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';') {
            TokenKind::Punctuation
        } else if ch.is_alphanumeric() {
            TokenKind::Text
        } else {
            TokenKind::Operator
        };
        push(i, i + ch.len_utf8(), kind, out);
        i += ch.len_utf8();
    }
}
