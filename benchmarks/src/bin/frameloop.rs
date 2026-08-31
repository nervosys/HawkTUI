//! Process-level frame-loop harness.
//!
//! Runs a complete, realistic redraw loop — build widgets, lay them out, render
//! into a back buffer, diff against the front buffer, encode the escape-sequence
//! stream — for a fixed number of frames, then reports throughput and peak
//! resident memory for the whole process.
//!
//! Each framework runs in its own process so the memory figure reflects only
//! that framework's allocations.
//!
//! ```text
//! frameloop <hawk|ratatui> [frames]
//! ```

use std::time::Instant;

const W: u16 = 200;
const H: u16 = 50;

const LOREM: &str = "The quick brown fox jumps over the lazy dog while the terminal renderer \
diffs two frame buffers and emits the minimal escape sequence stream required to bring the \
screen up to date without flicker or tearing on any modern emulator.";

fn main() {
    let mut args = std::env::args().skip(1);
    let framework = args.next().unwrap_or_else(|| "hawk".into());
    let frames: usize = args
        .next()
        .and_then(|f| f.parse().ok())
        .unwrap_or(10_000);

    let start = Instant::now();
    let bytes = match framework.as_str() {
        "hawk" => hawk_loop(frames),
        "ratatui" => ratatui_loop(frames),
        other => {
            eprintln!("unknown framework: {other}");
            std::process::exit(2);
        }
    };
    let elapsed = start.elapsed();

    let fps = frames as f64 / elapsed.as_secs_f64();
    let peak_kb = peak_rss_kb();
    println!(
        "{framework}\t{frames}\t{:.1}\t{:.0}\t{}\t{}",
        elapsed.as_secs_f64() * 1000.0,
        fps,
        peak_kb,
        bytes
    );
}

/// One frame of a realistic dashboard, rendered through Hawk TUI.
fn hawk_loop(frames: usize) -> usize {
    use hawktui::backend::Backend;
    use hawktui::backend::crossterm_backend::CrosstermBackend;
    use hawktui::core::buffer::Buffer;
    use hawktui::core::rect::Rect;
    use hawktui::core::style::{Color, Style, Stylize};
    use hawktui::layout::{Constraint, Direction, Layout};
    use hawktui::widget::block::{Block, Borders};
    use hawktui::widget::gauge::Gauge;
    use hawktui::widget::list::{List, ListItem, ListState};
    use hawktui::widget::paragraph::{Paragraph, Wrap};
    use hawktui::widget::{StatefulWidget, Widget};

    let area = Rect::new(0, 0, W, H);
    let mut front = Buffer::empty(area);
    let mut back = Buffer::empty(area);
    let mut sink: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut total = 0usize;
    let mut cells = 0usize;

    for frame in 0..frames {
        back.reset();
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        Paragraph::new(LOREM)
            .block(Block::default().title("Header").borders(Borders::ALL))
            .render(rows[0], &mut back);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let items: Vec<ListItem> = (0..40)
            .map(|i| ListItem::new(format!("item {:>3} - tick {}", i, frame % 1000)))
            .collect();
        let mut state = ListState::default();
        state.selected = Some(frame % 40);
        StatefulWidget::render(
            List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(Style::default().reversed()),
            cols[0],
            &mut back,
            &mut state,
        );

        Paragraph::new(LOREM)
            .wrap(Wrap::Word)
            .style(Style::default().fg(Color::Green))
            .block(Block::default().title("Body").borders(Borders::ALL))
            .render(cols[1], &mut back);

        Gauge::default()
            .ratio((frame % 100) as f64 / 100.0)
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .render(rows[2], &mut back);

        let changes = front.diff(&back);
        cells += changes.len();
        sink.clear();
        {
            let mut backend = CrosstermBackend::new(&mut sink);
            backend
                .draw(changes.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
        }
        total += sink.len();
        std::mem::swap(&mut front, &mut back);
    }
    eprintln!("changed cells: {cells}");
    total
}

/// The same frame, rendered through ratatui.
fn ratatui_loop(frames: usize) -> usize {
    use ratatui::backend::{Backend, CrosstermBackend};
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::widgets::{
        Block, Borders, Gauge, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    };

    let area = Rect::new(0, 0, W, H);
    let mut front = Buffer::empty(area);
    let mut back = Buffer::empty(area);
    let mut sink: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut total = 0usize;
    let mut cells = 0usize;

    for frame in 0..frames {
        back.reset();
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        Paragraph::new(LOREM)
            .block(Block::default().title("Header").borders(Borders::ALL))
            .render(rows[0], &mut back);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let items: Vec<ListItem> = (0..40)
            .map(|i| ListItem::new(format!("item {:>3} - tick {}", i, frame % 1000)))
            .collect();
        let mut state = ListState::default();
        state.select(Some(frame % 40));
        StatefulWidget::render(
            List::new(items)
                .block(Block::default().title("List").borders(Borders::ALL))
                .highlight_style(Style::new().add_modifier(Modifier::REVERSED)),
            cols[0],
            &mut back,
            &mut state,
        );

        Paragraph::new(LOREM)
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Green))
            .block(Block::default().title("Body").borders(Borders::ALL))
            .render(cols[1], &mut back);

        Gauge::default()
            .ratio((frame % 100) as f64 / 100.0)
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .render(rows[2], &mut back);

        let changes = front.diff(&back);
        cells += changes.len();
        sink.clear();
        {
            let mut backend = CrosstermBackend::new(&mut sink);
            backend
                .draw(changes.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
        }
        total += sink.len();
        std::mem::swap(&mut front, &mut back);
    }
    eprintln!("changed cells: {cells}");
    total
}

/// Peak resident set size of this process, in kilobytes.
#[cfg(windows)]
fn peak_rss_kb() -> u64 {
    // PROCESS_MEMORY_COUNTERS, queried through the same PSAPI entry point the
    // task manager uses. Declared inline to keep the harness dependency-free.
    #[repr(C)]
    #[derive(Default)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
    }

    let mut counters = ProcessMemoryCounters {
        cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
        ..Default::default()
    };
    // SAFETY: `counters` is a correctly sized, correctly aligned local.
    let ok = unsafe {
        K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<ProcessMemoryCounters>() as u32,
        )
    };
    if ok == 0 {
        0
    } else {
        (counters.peak_working_set_size / 1024) as u64
    }
}

/// Peak resident set size of this process, in kilobytes.
#[cfg(unix)]
fn peak_rss_kb() -> u64 {
    // `VmHWM` on Linux; falls back to 0 elsewhere.
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(not(any(windows, unix)))]
fn peak_rss_kb() -> u64 {
    0
}
