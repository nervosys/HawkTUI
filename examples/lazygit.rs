//! Lazygit-style Git TUI — built with Louie.
//!
//! Replicates the layout and interaction model of lazygit, the most popular
//! terminal UI (~55k GitHub stars):
//!
//! - Left sidebar with 5 stacked panels: Status, Files, Branches, Commits, Stash
//! - Right panel showing diffs / details with syntax-coloured hunks
//! - Keyboard navigation: 1-5 switch panels, j/k navigate, Tab cycle
//! - Simulated git data (no real git commands)

use louie::ontology::registry::OntologyRegistry;
use louie::prelude::*;
use louie::runtime::{Command, Model, Program, ProgramOptions};
use louie::widget::list::{List, ListItem, ListState};
use louie::widget::paragraph::Wrap;
use louie::widget::scrollbar::{Scrollbar, ScrollbarOrientation, ScrollbarState};

// ── Palette ─────────────────────────────────────────────────────────────────

const BORDER: Color = Color::DarkGray;
const BORDER_ACTIVE: Color = Color::Green;
const ADDED: Color = Color::Green;
const REMOVED: Color = Color::Red;
const HUNK: Color = Color::Cyan;
const FILE_MOD: Color = Color::Yellow;
const FILE_ADD: Color = Color::Green;
const FILE_DEL: Color = Color::Red;
const BRANCH_HEAD: Color = Color::Green;
const BRANCH_OTHER: Color = Color::Cyan;
const HASH: Color = Color::Yellow;
const DIM: Color = Color::DarkGray;
const STASH: Color = Color::Magenta;

// ── Panel enum ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Status,
    Files,
    Branches,
    Commits,
    Stash,
}

impl Panel {
    const ALL: [Panel; 5] = [
        Panel::Status,
        Panel::Files,
        Panel::Branches,
        Panel::Commits,
        Panel::Stash,
    ];

    fn index(self) -> usize {
        match self {
            Panel::Status => 0,
            Panel::Files => 1,
            Panel::Branches => 2,
            Panel::Commits => 3,
            Panel::Stash => 4,
        }
    }

    fn from_index(i: usize) -> Self {
        Panel::ALL[i % Panel::ALL.len()]
    }

    fn title(self) -> &'static str {
        match self {
            Panel::Status => " Status ",
            Panel::Files => " Files ",
            Panel::Branches => " Local Branches ",
            Panel::Commits => " Commits ",
            Panel::Stash => " Stash ",
        }
    }

    fn number(self) -> char {
        match self {
            Panel::Status => '1',
            Panel::Files => '2',
            Panel::Branches => '3',
            Panel::Commits => '4',
            Panel::Stash => '5',
        }
    }
}

// ── Simulated git data ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct FileEntry {
    status: char, // M, A, D, ?
    path: String,
    staged: bool,
}

#[derive(Clone, Debug)]
struct Branch {
    name: String,
    is_head: bool,
    behind: u32,
    ahead: u32,
}

#[derive(Clone, Debug)]
struct Commit {
    hash: String,
    message: String,
    author: String,
    age: String,
}

#[derive(Clone, Debug)]
struct StashEntry {
    index: u32,
    message: String,
}

fn sample_files() -> Vec<FileEntry> {
    vec![
        FileEntry {
            status: 'M',
            path: "src/layout/mod.rs".into(),
            staged: true,
        },
        FileEntry {
            status: 'M',
            path: "src/runtime/mod.rs".into(),
            staged: true,
        },
        FileEntry {
            status: 'A',
            path: "examples/lazygit.rs".into(),
            staged: true,
        },
        FileEntry {
            status: 'M',
            path: "examples/opencode.rs".into(),
            staged: false,
        },
        FileEntry {
            status: '?',
            path: "TODO.md".into(),
            staged: false,
        },
        FileEntry {
            status: 'D',
            path: "src/deprecated.rs".into(),
            staged: false,
        },
    ]
}

fn sample_branches() -> Vec<Branch> {
    vec![
        Branch {
            name: "main".into(),
            is_head: true,
            behind: 0,
            ahead: 2,
        },
        Branch {
            name: "feature/tui-widgets".into(),
            is_head: false,
            behind: 1,
            ahead: 5,
        },
        Branch {
            name: "fix/layout-min-constraint".into(),
            is_head: false,
            behind: 0,
            ahead: 1,
        },
        Branch {
            name: "refactor/elm-runtime".into(),
            is_head: false,
            behind: 3,
            ahead: 0,
        },
        Branch {
            name: "experiment/canvas-plot".into(),
            is_head: false,
            behind: 12,
            ahead: 0,
        },
    ]
}

fn sample_commits() -> Vec<Commit> {
    vec![
        Commit {
            hash: "a1b2c3d".into(),
            message: "feat: add lazygit-style TUI demo".into(),
            author: "adam".into(),
            age: "2m".into(),
        },
        Commit {
            hash: "e4f5a6b".into(),
            message: "fix: layout Min(0) absorbs remaining space".into(),
            author: "adam".into(),
            age: "15m".into(),
        },
        Commit {
            hash: "c7d8e9f".into(),
            message: "feat: add opencode chat demo".into(),
            author: "adam".into(),
            age: "1h".into(),
        },
        Commit {
            hash: "0a1b2c3".into(),
            message: "fix: filter crossterm Release events".into(),
            author: "adam".into(),
            age: "2h".into(),
        },
        Commit {
            hash: "d4e5f6a".into(),
            message: "feat: markdown widget with inline styling".into(),
            author: "adam".into(),
            age: "3h".into(),
        },
        Commit {
            hash: "b7c8d9e".into(),
            message: "feat: input widget with cursor and scroll".into(),
            author: "adam".into(),
            age: "5h".into(),
        },
        Commit {
            hash: "f0a1b2c".into(),
            message: "refactor: elm architecture runtime loop".into(),
            author: "adam".into(),
            age: "1d".into(),
        },
        Commit {
            hash: "3d4e5f6".into(),
            message: "feat: scrollbar with orientation support".into(),
            author: "adam".into(),
            age: "1d".into(),
        },
        Commit {
            hash: "a7b8c9d".into(),
            message: "feat: ontology registry for agent discovery".into(),
            author: "adam".into(),
            age: "2d".into(),
        },
        Commit {
            hash: "e0f1a2b".into(),
            message: "init: louie TUI framework".into(),
            author: "adam".into(),
            age: "3d".into(),
        },
    ]
}

fn sample_stash() -> Vec<StashEntry> {
    vec![
        StashEntry {
            index: 0,
            message: "WIP: experimental canvas rendering".into(),
        },
        StashEntry {
            index: 1,
            message: "save: half-done chart widget".into(),
        },
    ]
}

/// Simulated diff for the currently selected file.
fn diff_for_file(file: &FileEntry) -> Vec<DiffLine> {
    match file.path.as_str() {
        "src/layout/mod.rs" => vec![
            DiffLine::Header("diff --git a/src/layout/mod.rs b/src/layout/mod.rs".into()),
            DiffLine::Header("index 4a2b3c1..8f9e0d2 100644".into()),
            DiffLine::Header("--- a/src/layout/mod.rs".into()),
            DiffLine::Header("+++ b/src/layout/mod.rs".into()),
            DiffLine::Hunk("@@ -189,6 +189,22 @@ impl Layout {".into()),
            DiffLine::Context("        // Phase 2: distribute remaining space to Fill constraints".into()),
            DiffLine::Context("        let filled: u16 = sizes.iter().sum();".into()),
            DiffLine::Context("        let remaining = available.saturating_sub(filled);".into()),
            DiffLine::Added("+".into()),
            DiffLine::Added("+        // Phase 2b: distribute leftover space to Min constraints".into()),
            DiffLine::Added("+        let leftover_after_fill = available.saturating_sub(sizes.iter().sum());".into()),
            DiffLine::Added("+        let min_indices: Vec<usize> = self.constraints.iter().enumerate()".into()),
            DiffLine::Added("+            .filter(|(_, c)| matches!(c, Constraint::Min(_)))".into()),
            DiffLine::Added("+            .map(|(i, _)| i)".into()),
            DiffLine::Added("+            .collect();".into()),
            DiffLine::Added("+        if !min_indices.is_empty() && leftover_after_fill > 0 {".into()),
            DiffLine::Added("+            let share = leftover_after_fill / min_indices.len() as u16;".into()),
            DiffLine::Added("+            let extra = leftover_after_fill % min_indices.len() as u16;".into()),
            DiffLine::Added("+            for (j, &idx) in min_indices.iter().enumerate() {".into()),
            DiffLine::Added("+                sizes[idx] += share + if (j as u16) < extra { 1 } else { 0 };".into()),
            DiffLine::Added("+            }".into()),
            DiffLine::Added("+        }".into()),
            DiffLine::Context("".into()),
            DiffLine::Context("        // Phase 3: build rectangles".into()),
            DiffLine::Context("        let mut pos = match self.direction {".into()),
        ],
        "src/runtime/mod.rs" => vec![
            DiffLine::Header("diff --git a/src/runtime/mod.rs b/src/runtime/mod.rs".into()),
            DiffLine::Header("--- a/src/runtime/mod.rs".into()),
            DiffLine::Header("+++ b/src/runtime/mod.rs".into()),
            DiffLine::Hunk("@@ -195,6 +195,10 @@ impl<M: Model, B: Backend> Program<M, B> {".into()),
            DiffLine::Context("                let event = self.convert_event(raw_event);".into()),
            DiffLine::Added("+                // Filter Release events (Windows crossterm dual-fires)".into()),
            DiffLine::Added("+                if matches!(&event, Event::Key(k) if k.kind == crossterm::event::KeyEventKind::Release) {".into()),
            DiffLine::Added("+                    continue;".into()),
            DiffLine::Added("+                }".into()),
            DiffLine::Context("                if let Some(msg) = self.model.handle_event(event) {".into()),
            DiffLine::Context("                    let cmd = self.model.update(msg);".into()),
        ],
        "examples/lazygit.rs" => vec![
            DiffLine::Header("diff --git a/examples/lazygit.rs b/examples/lazygit.rs".into()),
            DiffLine::Header("new file mode 100644".into()),
            DiffLine::Header("--- /dev/null".into()),
            DiffLine::Header("+++ b/examples/lazygit.rs".into()),
            DiffLine::Hunk("@@ -0,0 +1,42 @@".into()),
            DiffLine::Added("+//! Lazygit-style Git TUI — built with Louie.".into()),
            DiffLine::Added("+//!".into()),
            DiffLine::Added("+//! Replicates the layout and interaction model of lazygit,".into()),
            DiffLine::Added("+//! the most popular terminal UI.".into()),
            DiffLine::Added("+".into()),
            DiffLine::Added("+use louie::prelude::*;".into()),
            DiffLine::Added("+// ... (new file, 400+ lines)".into()),
        ],
        "examples/opencode.rs" => vec![
            DiffLine::Header("diff --git a/examples/opencode.rs b/examples/opencode.rs".into()),
            DiffLine::Header("--- a/examples/opencode.rs".into()),
            DiffLine::Header("+++ b/examples/opencode.rs".into()),
            DiffLine::Hunk("@@ -27,7 +27,7 @@".into()),
            DiffLine::Context(" const ACCENT: Color = Color::Cyan;".into()),
            DiffLine::Removed("-const SCHEMA_FG: Color = Color::White;".into()),
            DiffLine::Added("+const SCHEMA_FG: Color = Color::Gray;".into()),
            DiffLine::Context(" const DIM: Color = Color::DarkGray;".into()),
        ],
        "TODO.md" => vec![
            DiffLine::Header("diff --git a/TODO.md b/TODO.md".into()),
            DiffLine::Header("new file mode 100644".into()),
            DiffLine::Hunk("@@ -0,0 +1,5 @@".into()),
            DiffLine::Added("+# TODO".into()),
            DiffLine::Added("+- [ ] Add word-wrapping to diff panel".into()),
            DiffLine::Added("+- [ ] Implement real git integration".into()),
            DiffLine::Added("+- [ ] Add file staging toggle".into()),
            DiffLine::Added("+- [ ] Support merge conflict resolution".into()),
        ],
        "src/deprecated.rs" => vec![
            DiffLine::Header("diff --git a/src/deprecated.rs b/src/deprecated.rs".into()),
            DiffLine::Header("deleted file mode 100644".into()),
            DiffLine::Header("--- a/src/deprecated.rs".into()),
            DiffLine::Header("+++ /dev/null".into()),
            DiffLine::Hunk("@@ -1,8 +0,0 @@".into()),
            DiffLine::Removed("-//! Deprecated module — scheduled for removal.".into()),
            DiffLine::Removed("-pub fn old_render() {}".into()),
            DiffLine::Removed("-pub fn legacy_layout() {}".into()),
        ],
        _ => vec![DiffLine::Context("(no diff available)".into())],
    }
}

#[derive(Clone, Debug)]
enum DiffLine {
    Header(String),
    Hunk(String),
    Added(String),
    Removed(String),
    Context(String),
}

/// Detail text for non-file panels.
fn branch_detail(branch: &Branch) -> Vec<DiffLine> {
    let mut lines = vec![
        DiffLine::Header(format!("Branch: {}", branch.name)),
        DiffLine::Context(String::new()),
    ];
    if branch.is_head {
        lines.push(DiffLine::Added("  ★ HEAD (current branch)".into()));
    }
    if branch.ahead > 0 {
        lines.push(DiffLine::Added(format!(
            "  ↑ {} ahead of origin",
            branch.ahead
        )));
    }
    if branch.behind > 0 {
        lines.push(DiffLine::Removed(format!(
            "  ↓ {} behind origin",
            branch.behind
        )));
    }
    if branch.ahead == 0 && branch.behind == 0 && !branch.is_head {
        lines.push(DiffLine::Context("  ✓ Up to date with origin".into()));
    }
    lines
}

fn commit_detail(commit: &Commit) -> Vec<DiffLine> {
    vec![
        DiffLine::Header(format!("commit {}", commit.hash)),
        DiffLine::Context(format!("Author: {}", commit.author)),
        DiffLine::Context(format!("Date:   {} ago", commit.age)),
        DiffLine::Context(String::new()),
        DiffLine::Context(format!("    {}", commit.message)),
    ]
}

fn stash_detail(entry: &StashEntry) -> Vec<DiffLine> {
    vec![
        DiffLine::Header(format!("stash@{{{}}}", entry.index)),
        DiffLine::Context(String::new()),
        DiffLine::Context(format!("  {}", entry.message)),
    ]
}

// ── App state ───────────────────────────────────────────────────────────────

struct App {
    active_panel: Panel,

    // Data
    files: Vec<FileEntry>,
    branches: Vec<Branch>,
    commits: Vec<Commit>,
    stash: Vec<StashEntry>,

    // List states
    file_state: ListState,
    branch_state: ListState,
    commit_state: ListState,
    stash_state: ListState,

    // Diff scroll
    diff_scroll: u16,
}

// ── Messages ────────────────────────────────────────────────────────────────

enum Msg {
    // Navigation
    MoveDown,
    MoveUp,
    MoveTop,
    MoveBottom,
    NextPanel,
    PrevPanel,
    SelectPanel(Panel),

    // Actions
    ToggleStage, // space — stage/unstage file
    ScrollDiffDown,
    ScrollDiffUp,

    Quit,
}

impl App {
    fn new() -> Self {
        Self {
            active_panel: Panel::Files,
            files: sample_files(),
            branches: sample_branches(),
            commits: sample_commits(),
            stash: sample_stash(),
            file_state: ListState::new().with_selected(Some(0)),
            branch_state: ListState::new().with_selected(Some(0)),
            commit_state: ListState::new().with_selected(Some(0)),
            stash_state: ListState::new().with_selected(Some(0)),
            diff_scroll: 0,
        }
    }

    fn active_list_state_mut(&mut self) -> Option<&mut ListState> {
        match self.active_panel {
            Panel::Status => None,
            Panel::Files => Some(&mut self.file_state),
            Panel::Branches => Some(&mut self.branch_state),
            Panel::Commits => Some(&mut self.commit_state),
            Panel::Stash => Some(&mut self.stash_state),
        }
    }

    fn active_item_count(&self) -> usize {
        match self.active_panel {
            Panel::Status => 0,
            Panel::Files => self.files.len(),
            Panel::Branches => self.branches.len(),
            Panel::Commits => self.commits.len(),
            Panel::Stash => self.stash.len(),
        }
    }

    /// Get the diff/detail lines for the current panel and selection.
    fn current_detail(&self) -> Vec<DiffLine> {
        match self.active_panel {
            Panel::Status => {
                let staged = self.files.iter().filter(|f| f.staged).count();
                let unstaged = self.files.iter().filter(|f| !f.staged).count();
                vec![
                    DiffLine::Header("Repository: louie".into()),
                    DiffLine::Context(format!(
                        "Branch:     {}",
                        self.branches
                            .iter()
                            .find(|b| b.is_head)
                            .map(|b| b.name.as_str())
                            .unwrap_or("???")
                    )),
                    DiffLine::Context(String::new()),
                    DiffLine::Added(format!("  {} file(s) staged", staged)),
                    DiffLine::Removed(format!("  {} file(s) unstaged", unstaged)),
                    DiffLine::Context(String::new()),
                    DiffLine::Context(format!("  {} commit(s) total", self.commits.len())),
                    DiffLine::Context(format!("  {} stash entries", self.stash.len())),
                    DiffLine::Context(format!("  {} local branches", self.branches.len())),
                ]
            }
            Panel::Files => {
                if let Some(sel) = self.file_state.selected {
                    if sel < self.files.len() {
                        return diff_for_file(&self.files[sel]);
                    }
                }
                vec![DiffLine::Context("(no file selected)".into())]
            }
            Panel::Branches => {
                if let Some(sel) = self.branch_state.selected {
                    if sel < self.branches.len() {
                        return branch_detail(&self.branches[sel]);
                    }
                }
                vec![DiffLine::Context("(no branch selected)".into())]
            }
            Panel::Commits => {
                if let Some(sel) = self.commit_state.selected {
                    if sel < self.commits.len() {
                        return commit_detail(&self.commits[sel]);
                    }
                }
                vec![DiffLine::Context("(no commit selected)".into())]
            }
            Panel::Stash => {
                if let Some(sel) = self.stash_state.selected {
                    if sel < self.stash.len() {
                        return stash_detail(&self.stash[sel]);
                    }
                }
                vec![DiffLine::Context("(no stash selected)".into())]
            }
        }
    }
}

// ── Model impl ──────────────────────────────────────────────────────────────

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::MoveDown => {
                let count = self.active_item_count();
                if let Some(state) = self.active_list_state_mut() {
                    state.select_next(count);
                }
                self.diff_scroll = 0;
                Command::None
            }
            Msg::MoveUp => {
                if let Some(state) = self.active_list_state_mut() {
                    state.select_previous();
                }
                self.diff_scroll = 0;
                Command::None
            }
            Msg::MoveTop => {
                if let Some(state) = self.active_list_state_mut() {
                    state.select_first();
                }
                self.diff_scroll = 0;
                Command::None
            }
            Msg::MoveBottom => {
                let count = self.active_item_count();
                if let Some(state) = self.active_list_state_mut() {
                    state.select_last(count);
                }
                self.diff_scroll = 0;
                Command::None
            }
            Msg::NextPanel => {
                let next = (self.active_panel.index() + 1) % Panel::ALL.len();
                self.active_panel = Panel::from_index(next);
                self.diff_scroll = 0;
                Command::None
            }
            Msg::PrevPanel => {
                let prev = if self.active_panel.index() == 0 {
                    Panel::ALL.len() - 1
                } else {
                    self.active_panel.index() - 1
                };
                self.active_panel = Panel::from_index(prev);
                self.diff_scroll = 0;
                Command::None
            }
            Msg::SelectPanel(p) => {
                self.active_panel = p;
                self.diff_scroll = 0;
                Command::None
            }
            Msg::ToggleStage => {
                if self.active_panel == Panel::Files {
                    if let Some(sel) = self.file_state.selected {
                        if sel < self.files.len() {
                            self.files[sel].staged = !self.files[sel].staged;
                        }
                    }
                }
                Command::None
            }
            Msg::ScrollDiffDown => {
                self.diff_scroll = self.diff_scroll.saturating_add(3);
                Command::None
            }
            Msg::ScrollDiffUp => {
                self.diff_scroll = self.diff_scroll.saturating_sub(3);
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        // ┌─sidebar─┬─────────main──────────┐
        // │         │                        │
        // │         │                        │
        // └─────────┴────────────────────────┘
        // │ status bar                       │

        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(1)])
            .split(area);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(outer[0]);

        self.render_sidebar(frame, body[0]);
        self.render_diff(frame, body[1]);
        self.render_status_bar(frame, outer[1]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        if let Event::Key(key) = event {
            if key.is_ctrl(KeyCode::Char('c')) {
                return Some(Msg::Quit);
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),

                // Panel item navigation
                KeyCode::Char('j') | KeyCode::Down => Some(Msg::MoveDown),
                KeyCode::Char('k') | KeyCode::Up => Some(Msg::MoveUp),
                KeyCode::Char('g') => Some(Msg::MoveTop),
                KeyCode::Char('G') => Some(Msg::MoveBottom),

                // Panel switching
                KeyCode::Tab => Some(Msg::NextPanel),
                KeyCode::BackTab => Some(Msg::PrevPanel),
                KeyCode::Char('1') => Some(Msg::SelectPanel(Panel::Status)),
                KeyCode::Char('2') => Some(Msg::SelectPanel(Panel::Files)),
                KeyCode::Char('3') => Some(Msg::SelectPanel(Panel::Branches)),
                KeyCode::Char('4') => Some(Msg::SelectPanel(Panel::Commits)),
                KeyCode::Char('5') => Some(Msg::SelectPanel(Panel::Stash)),

                // Actions
                KeyCode::Char(' ') => Some(Msg::ToggleStage),

                // Diff scroll
                KeyCode::Char('J') | KeyCode::PageDown => Some(Msg::ScrollDiffDown),
                KeyCode::Char('K') | KeyCode::PageUp => Some(Msg::ScrollDiffUp),

                _ => None,
            }
        } else {
            None
        }
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Block>();
        registry.register::<Paragraph>();
        registry.register::<louie::widget::list::List>();
        registry.register::<louie::widget::scrollbar::Scrollbar>();
    }
}

// ── View helpers ────────────────────────────────────────────────────────────

impl App {
    fn panel_block(&self, panel: Panel) -> Block {
        let active = self.active_panel == panel;
        let border_color = if active { BORDER_ACTIVE } else { BORDER };
        let number = panel.number();
        let title_text = format!("{}{}", number, panel.title());

        Block::default()
            .title(Line::styled(
                title_text,
                if active {
                    Style::default().fg(BORDER_ACTIVE).bold()
                } else {
                    Style::default().fg(BORDER)
                },
            ))
            .borders(Borders::ALL)
            .border_type(if active {
                BorderType::Rounded
            } else {
                BorderType::Plain
            })
            .border_style(Style::default().fg(border_color))
    }

    fn render_sidebar(&self, frame: &mut Frame<'_>, area: Rect) {
        // 5 panels stacked vertically — Status gets 3 rows, others share rest
        let panels = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Status
                Constraint::Min(3),    // Files
                Constraint::Min(3),    // Branches
                Constraint::Min(3),    // Commits
                Constraint::Length(4), // Stash
            ])
            .split(area);

        self.render_status_panel(frame, panels[0]);
        self.render_file_panel(frame, panels[1]);
        self.render_branch_panel(frame, panels[2]);
        self.render_commit_panel(frame, panels[3]);
        self.render_stash_panel(frame, panels[4]);
    }

    fn render_status_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = self.panel_block(Panel::Status);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        let head = self
            .branches
            .iter()
            .find(|b| b.is_head)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "???".into());

        let staged = self.files.iter().filter(|f| f.staged).count();
        let unstaged = self.files.iter().filter(|f| !f.staged).count();

        let lines = vec![
            Line::from(vec![
                Span::styled("↑ ", Style::default().fg(ADDED)),
                Span::styled(head, Style::default().fg(BRANCH_HEAD).bold()),
                Span::raw("  "),
                Span::styled(
                    format!(
                        "↑{}",
                        self.branches
                            .iter()
                            .find(|b| b.is_head)
                            .map_or(0, |b| b.ahead)
                    ),
                    Style::default().fg(ADDED),
                ),
                Span::raw(" "),
                Span::styled(
                    format!(
                        "↓{}",
                        self.branches
                            .iter()
                            .find(|b| b.is_head)
                            .map_or(0, |b| b.behind)
                    ),
                    Style::default().fg(REMOVED),
                ),
            ]),
            Line::from(vec![
                Span::styled(format!("{}●", staged), Style::default().fg(ADDED)),
                Span::raw(" staged  "),
                Span::styled(format!("{}●", unstaged), Style::default().fg(FILE_MOD)),
                Span::raw(" modified"),
            ]),
            Line::from(vec![
                Span::styled("louie", Style::default().fg(Color::White).bold()),
                Span::styled(" — agentic TUI framework", Style::default().fg(DIM)),
            ]),
        ];

        let para = Paragraph::new(lines);
        frame.render_widget(para, inner);
    }

    fn render_file_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let items: Vec<ListItem> = self
            .files
            .iter()
            .map(|f| {
                let status_style = match f.status {
                    'M' => Style::default().fg(FILE_MOD),
                    'A' => Style::default().fg(FILE_ADD),
                    'D' => Style::default().fg(FILE_DEL),
                    _ => Style::default().fg(DIM),
                };
                let staged_indicator = if f.staged { "● " } else { "  " };
                let staged_style = if f.staged {
                    Style::default().fg(ADDED)
                } else {
                    Style::default().fg(DIM)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(staged_indicator, staged_style),
                    Span::styled(format!("{} ", f.status), status_style),
                    Span::styled(f.path.clone(), Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(self.panel_block(Panel::Files))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
            .highlight_symbol("▸ ");

        let mut state = self.file_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_branch_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let items: Vec<ListItem> = self
            .branches
            .iter()
            .map(|b| {
                let mut spans = Vec::new();
                if b.is_head {
                    spans.push(Span::styled("* ", Style::default().fg(BRANCH_HEAD).bold()));
                } else {
                    spans.push(Span::raw("  "));
                }
                let name_style = if b.is_head {
                    Style::default().fg(BRANCH_HEAD).bold()
                } else {
                    Style::default().fg(BRANCH_OTHER)
                };
                spans.push(Span::styled(b.name.clone(), name_style));

                if b.ahead > 0 || b.behind > 0 {
                    spans.push(Span::raw(" "));
                    if b.ahead > 0 {
                        spans.push(Span::styled(
                            format!("↑{}", b.ahead),
                            Style::default().fg(ADDED),
                        ));
                    }
                    if b.behind > 0 {
                        if b.ahead > 0 {
                            spans.push(Span::raw(" "));
                        }
                        spans.push(Span::styled(
                            format!("↓{}", b.behind),
                            Style::default().fg(REMOVED),
                        ));
                    }
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .block(self.panel_block(Panel::Branches))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan))
            .highlight_symbol("▸ ");

        let mut state = self.branch_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_commit_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let items: Vec<ListItem> = self
            .commits
            .iter()
            .map(|c| {
                ListItem::new(Line::from(vec![
                    Span::styled(c.hash.clone(), Style::default().fg(HASH)),
                    Span::raw(" "),
                    Span::styled(c.message.clone(), Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(c.age.clone(), Style::default().fg(DIM)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(self.panel_block(Panel::Commits))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .highlight_symbol("▸ ");

        let mut state = self.commit_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_stash_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let items: Vec<ListItem> = self
            .stash
            .iter()
            .map(|s| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("stash@{{{}}}", s.index), Style::default().fg(STASH)),
                    Span::raw(" "),
                    Span::styled(s.message.clone(), Style::default().fg(Color::White)),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(self.panel_block(Panel::Stash))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Magenta))
            .highlight_symbol("▸ ");

        let mut state = self.stash_state.clone();
        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_diff(&self, frame: &mut Frame<'_>, area: Rect) {
        let detail = self.current_detail();

        let title = match self.active_panel {
            Panel::Status => " Overview ".to_string(),
            Panel::Files => {
                if let Some(sel) = self.file_state.selected {
                    if sel < self.files.len() {
                        format!(" {} ", self.files[sel].path)
                    } else {
                        " Diff ".into()
                    }
                } else {
                    " Diff ".into()
                }
            }
            Panel::Branches => " Branch Detail ".into(),
            Panel::Commits => " Commit ".into(),
            Panel::Stash => " Stash ".into(),
        };

        let block = Block::default()
            .title(Line::styled(
                title,
                Style::default().fg(BORDER_ACTIVE).bold(),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.is_empty() {
            return;
        }

        // Build styled lines from diff
        let lines: Vec<Line> = detail
            .iter()
            .map(|dl| match dl {
                DiffLine::Header(s) => {
                    Line::styled(s.clone(), Style::default().fg(Color::White).bold())
                }
                DiffLine::Hunk(s) => Line::styled(s.clone(), Style::default().fg(HUNK)),
                DiffLine::Added(s) => Line::styled(s.clone(), Style::default().fg(ADDED)),
                DiffLine::Removed(s) => Line::styled(s.clone(), Style::default().fg(REMOVED)),
                DiffLine::Context(s) => Line::styled(s.clone(), Style::default().fg(DIM)),
            })
            .collect();

        let para = Paragraph::new(lines)
            .wrap(Wrap::None)
            .scroll((self.diff_scroll, 0));
        frame.render_widget(para, inner);

        // Scrollbar if content overflows
        let total_lines = detail.len();
        let visible = inner.height as usize;
        if total_lines > visible {
            let mut scroll_state =
                ScrollbarState::new(total_lines, visible).position(self.diff_scroll as usize);
            let scrollbar =
                Scrollbar::new(ScrollbarOrientation::Vertical).style(Style::default().fg(DIM));
            let scroll_area = Rect::new(
                area.right().saturating_sub(1),
                area.y + 1,
                1,
                area.height.saturating_sub(2),
            );
            frame.render_stateful_widget(scrollbar, scroll_area, &mut scroll_state);
        }
    }

    fn render_status_bar(&self, frame: &mut Frame<'_>, area: Rect) {
        let spans = vec![
            Span::styled(" 1-5", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" panel  ", Style::default().fg(DIM)),
            Span::styled("↑↓/jk", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" navigate  ", Style::default().fg(DIM)),
            Span::styled("space", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" stage  ", Style::default().fg(DIM)),
            Span::styled("J/K", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" scroll diff  ", Style::default().fg(DIM)),
            Span::styled("Tab", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" next panel  ", Style::default().fg(DIM)),
            Span::styled("q", Style::default().fg(BORDER_ACTIVE).bold()),
            Span::styled(" quit", Style::default().fg(DIM)),
        ];
        let bar = Paragraph::new(Line::from(spans));
        frame.render_widget(bar, area);
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let app = App::new();
    let backend = CrosstermBackend::new(std::io::stdout());
    let options = ProgramOptions {
        tick_rate: None,
        ..Default::default()
    };
    Program::new(app, backend)?.with_options(options).run()?;
    Ok(())
}
