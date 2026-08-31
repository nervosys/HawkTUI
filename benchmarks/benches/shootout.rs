//! Head-to-head benchmarks: Hawk TUI vs other Rust TUI frameworks.
//!
//! Every group runs the *same* workload through each framework's public API so
//! the numbers are directly comparable. Groups are named `<workload>/<framework>`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

const W: u16 = 200;
const H: u16 = 50;

const LOREM: &str = "The quick brown fox jumps over the lazy dog while the terminal renderer \
diffs two frame buffers and emits the minimal escape sequence stream required to bring the \
screen up to date without flicker or tearing on any modern emulator.";

// ---------------------------------------------------------------- hawk helpers

mod hawk {
    use hawktui::core::buffer::Buffer;
    use hawktui::core::rect::Rect;
    use hawktui::core::style::{Color, Style};
    use hawktui::layout::{Constraint, Direction, Layout};
    use hawktui::widget::block::{Block, Borders};
    use hawktui::widget::gauge::Gauge;
    use hawktui::widget::list::{List, ListItem, ListState};
    use hawktui::widget::paragraph::{Paragraph, Wrap};
    use hawktui::widget::{StatefulWidget, Widget};
    pub use hawktui::core::text::{Line, Span};

    pub fn area() -> Rect {
        Rect::new(0, 0, super::W, super::H)
    }

    pub fn styled() -> Style {
        Style::default().fg(Color::Green).bg(Color::Black).bold()
    }

    pub fn dirty_buffers(pct: usize) -> (Buffer, Buffer) {
        let a = Buffer::empty(area());
        let mut b = Buffer::empty(area());
        let total = (super::W as usize) * (super::H as usize);
        let step = 100 / pct;
        for i in (0..total).step_by(step) {
            let x = (i % super::W as usize) as u16;
            let y = (i / super::W as usize) as u16;
            b.set_string(x, y, "X", styled());
        }
        (a, b)
    }

    pub fn layout_solve(area: Rect) -> usize {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Percentage(30),
                Constraint::Length(1),
            ])
            .split(area);
        let mut n = 0;
        for row in rows.iter() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Ratio(1, 3),
                    Constraint::Min(10),
                    Constraint::Length(20),
                ])
                .split(*row);
            n += cols.len();
        }
        n
    }

    pub fn dashboard(buf: &mut Buffer) {
        let area = buf.area;
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        Paragraph::new(super::LOREM)
            .block(Block::default().title("Header").borders(Borders::ALL))
            .render(rows[0], buf);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let items: Vec<ListItem> = (0..40)
            .map(|i| ListItem::new(format!("item {i:>3} - payload")))
            .collect();
        let mut state = ListState::default();
        StatefulWidget::render(
            List::new(items).block(Block::default().title("List").borders(Borders::ALL)),
            cols[0],
            buf,
            &mut state,
        );

        Paragraph::new(super::LOREM)
            .wrap(Wrap::Word)
            .style(styled())
            .block(Block::default().title("Body").borders(Borders::ALL))
            .render(cols[1], buf);

        Gauge::default()
            .ratio(0.42)
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .render(rows[2], buf);
    }

    pub fn wrap_paragraph(buf: &mut Buffer, text: &str) {
        let area = buf.area;
        Paragraph::new(text).wrap(Wrap::Word).render(area, buf);
    }

    pub fn table(buf: &mut Buffer, rows: &[Vec<String>], selected: usize) {
        use hawktui::widget::table::{Table, TableColumn, TableColumnWidth, TableRow, TableState};
        let columns = vec![
            TableColumn::new("name", TableColumnWidth::Fill),
            TableColumn::new("status", TableColumnWidth::Fixed(12)),
            TableColumn::new("size", TableColumnWidth::Fixed(10)),
            TableColumn::new("modified", TableColumnWidth::Fill),
        ];
        let table_rows: Vec<TableRow> = rows
            .iter()
            .map(|cells| TableRow::new(cells.iter().map(|c| c.as_str())))
            .collect();
        let mut state = TableState::default();
        state.select(Some(selected));
        let area = buf.area;
        StatefulWidget::render(
            Table::new(columns, table_rows)
                .block(Block::default().title("Files").borders(Borders::ALL)),
            area,
            buf,
            &mut state,
        );
    }

    pub fn scrolling_list(buf: &mut Buffer, items: &[String], offset: usize) {
        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.as_str())).collect();
        let mut state = ListState::default();
        state.offset = offset;
        state.selected = Some(offset + 3);
        let area = buf.area;
        StatefulWidget::render(
            List::new(list_items).block(Block::default().borders(Borders::ALL)),
            area,
            buf,
            &mut state,
        );
    }

    pub fn styled_spans(buf: &mut Buffer, line: &Line) {
        let (w, h) = (buf.area.width, buf.area.height);
        for y in 0..h {
            buf.set_line(0, y, line, w);
        }
    }
}


// ------------------------------------------------------------- ratatui helpers

mod rat {
    use ratatui::buffer::Buffer;
    use ratatui::layout::{Constraint, Direction, Layout, Rect};
    use ratatui::style::{Color, Modifier, Style};
    pub use ratatui::text::{Line, Span};
    use ratatui::widgets::{
        Block, Borders, Gauge, List, ListItem, ListState, Paragraph, StatefulWidget, Widget, Wrap,
    };

    pub fn area() -> Rect {
        Rect::new(0, 0, super::W, super::H)
    }

    pub fn styled() -> Style {
        Style::default()
            .fg(Color::Green)
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD)
    }

    pub fn dirty_buffers(pct: usize) -> (Buffer, Buffer) {
        let a = Buffer::empty(area());
        let mut b = Buffer::empty(area());
        let total = (super::W as usize) * (super::H as usize);
        let step = 100 / pct;
        for i in (0..total).step_by(step) {
            let x = (i % super::W as usize) as u16;
            let y = (i / super::W as usize) as u16;
            b.set_string(x, y, "X", styled());
        }
        (a, b)
    }

    pub fn layout_solve(area: Rect) -> usize {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Percentage(30),
                Constraint::Length(1),
            ])
            .split(area);
        let mut n = 0;
        for row in rows.iter() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Ratio(1, 3),
                    Constraint::Min(10),
                    Constraint::Length(20),
                ])
                .split(*row);
            n += cols.len();
        }
        n
    }

    pub fn dashboard(buf: &mut Buffer) {
        let area = buf.area;
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        Paragraph::new(super::LOREM)
            .block(Block::default().title("Header").borders(Borders::ALL))
            .render(rows[0], buf);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        let items: Vec<ListItem> = (0..40)
            .map(|i| ListItem::new(format!("item {i:>3} - payload")))
            .collect();
        Widget::render(
            List::new(items).block(Block::default().title("List").borders(Borders::ALL)),
            cols[0],
            buf,
        );

        Paragraph::new(super::LOREM)
            .wrap(Wrap { trim: false })
            .style(styled())
            .block(Block::default().title("Body").borders(Borders::ALL))
            .render(cols[1], buf);

        Gauge::default()
            .ratio(0.42)
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .render(rows[2], buf);
    }

    pub fn wrap_paragraph(buf: &mut Buffer, text: &str) {
        let area = buf.area;
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    pub fn table(buf: &mut Buffer, rows: &[Vec<String>], selected: usize) {
        use ratatui::widgets::{Row, Table, TableState};
        let widths = [
            Constraint::Fill(1),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];
        let table_rows: Vec<Row> = rows
            .iter()
            .map(|cells| Row::new(cells.iter().map(|c| c.as_str())))
            .collect();
        let mut state = TableState::default();
        state.select(Some(selected));
        let area = buf.area;
        StatefulWidget::render(
            Table::new(table_rows, widths)
                .header(Row::new(vec!["name", "status", "size", "modified"]))
                .column_spacing(1)
                .block(Block::default().title("Files").borders(Borders::ALL)),
            area,
            buf,
            &mut state,
        );
    }

    pub fn scrolling_list(buf: &mut Buffer, items: &[String], offset: usize) {
        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.as_str())).collect();
        let mut state = ListState::default();
        *state.offset_mut() = offset;
        state.select(Some(offset + 3));
        let area = buf.area;
        StatefulWidget::render(
            List::new(list_items).block(Block::default().borders(Borders::ALL)),
            area,
            buf,
            &mut state,
        );
    }

    pub fn styled_spans(buf: &mut Buffer, line: &Line<'_>) {
        let (w, h) = (buf.area.width, buf.area.height);
        for y in 0..h {
            buf.set_line(0, y, line, w);
        }
    }
}



// -------------------------------------------------------- superlighttui helpers

mod light {
    use slt::buffer::Buffer;
    use slt::rect::Rect;
    use slt::style::{Color, Style};

    pub fn area() -> Rect {
        Rect::new(0, 0, super::W as u32, super::H as u32)
    }

    pub fn styled() -> Style {
        Style::new().fg(Color::Green).bg(Color::Black).bold()
    }

    pub fn dirty_buffers(pct: usize) -> (Buffer, Buffer) {
        let a = Buffer::empty(area());
        let mut b = Buffer::empty(area());
        let total = (super::W as usize) * (super::H as usize);
        let step = 100 / pct;
        for i in (0..total).step_by(step) {
            let x = (i % super::W as usize) as u32;
            let y = (i / super::W as usize) as u32;
            b.set_string(x, y, "X", styled());
        }
        (a, b)
    }
}

// ------------------------------------------------------------------ benchmarks


fn bench_buffer_alloc(c: &mut Criterion) {
    let mut g = c.benchmark_group("buffer_alloc_200x50");
    g.bench_function("hawk", |b| {
        b.iter(|| black_box(hawktui::core::buffer::Buffer::empty(hawk::area())))
    });
    g.bench_function("ratatui", |b| {
        b.iter(|| black_box(ratatui::buffer::Buffer::empty(rat::area())))
    });
    g.bench_function("superlighttui", |b| {
        b.iter(|| black_box(slt::buffer::Buffer::empty(light::area())))
    });
    g.finish();
}

fn bench_buffer_reset(c: &mut Criterion) {
    let mut g = c.benchmark_group("buffer_reset_200x50");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hb.reset();
            black_box(&hb);
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rb.reset();
            black_box(&rb);
        })
    });
    let mut sb = slt::buffer::Buffer::empty(light::area());
    g.bench_function("superlighttui", |b| {
        b.iter(|| {
            sb.reset();
            black_box(&sb);
        })
    });
    g.finish();
}

fn bench_diff(c: &mut Criterion) {
    for pct in [1usize, 5, 50] {
        let mut g = c.benchmark_group(format!("buffer_diff_{pct}pct_200x50"));
        let (ha, hbuf) = hawk::dirty_buffers(pct);
        g.bench_function("hawk", |b| b.iter(|| black_box(ha.diff(&hbuf)).len()));
        let (ra, rbuf) = rat::dirty_buffers(pct);
        g.bench_function("ratatui", |b| b.iter(|| black_box(ra.diff(&rbuf)).len()));
        let (sa, sbuf) = light::dirty_buffers(pct);
        g.bench_function("superlighttui", |b| {
            b.iter(|| black_box(sa.diff(&sbuf)).len())
        });
        g.finish();
    }
}

fn bench_set_string(c: &mut Criterion) {
    let line = "x".repeat(W as usize);
    let mut g = c.benchmark_group("set_string_full_screen");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    let hs = hawk::styled();
    g.bench_function("hawk", |b| {
        b.iter(|| {
            for y in 0..H {
                hb.set_string(0, y, black_box(&line), hs);
            }
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    let rs = rat::styled();
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            for y in 0..H {
                rb.set_string(0, y, black_box(&line), rs);
            }
        })
    });
    let mut sb = slt::buffer::Buffer::empty(light::area());
    let ss = light::styled();
    g.bench_function("superlighttui", |b| {
        b.iter(|| {
            for y in 0..H as u32 {
                sb.set_string(0, y, black_box(&line), ss);
            }
        })
    });
    g.finish();
}

fn bench_layout(c: &mut Criterion) {
    let mut g = c.benchmark_group("layout_solve_nested");
    g.bench_function("hawk", |b| {
        b.iter(|| black_box(hawk::layout_solve(hawk::area())))
    });
    g.bench_function("ratatui", |b| {
        b.iter(|| black_box(rat::layout_solve(rat::area())))
    });
    g.finish();
}

fn bench_dashboard(c: &mut Criterion) {
    let mut g = c.benchmark_group("render_dashboard_200x50");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hb.reset();
            hawk::dashboard(&mut hb);
            black_box(&hb);
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rb.reset();
            rat::dashboard(&mut rb);
            black_box(&rb);
        })
    });
    g.finish();
}

fn bench_wrap(c: &mut Criterion) {
    let text = LOREM.repeat(8);
    let mut g = c.benchmark_group("paragraph_wrap_200x50");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hb.reset();
            hawk::wrap_paragraph(&mut hb, black_box(&text));
            black_box(&hb);
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rb.reset();
            rat::wrap_paragraph(&mut rb, black_box(&text));
            black_box(&rb);
        })
    });
    g.finish();
}

fn bench_emit(c: &mut Criterion) {
    use hawktui::backend::Backend as HawkBackend;
    use ratatui::backend::Backend as RatBackend;

    let mut g = c.benchmark_group("terminal_emit_full_frame");

    let (ha, hbuf) = hawk::dirty_buffers(50);
    let hchanges: Vec<_> = ha.diff(&hbuf);
    let mut hsink: Vec<u8> = Vec::with_capacity(1 << 20);
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hsink.clear();
            let mut back = hawktui::backend::crossterm_backend::CrosstermBackend::new(&mut hsink);
            back.draw(hchanges.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
            black_box(hsink.len());
        })
    });

    let (ra, rbuf) = rat::dirty_buffers(50);
    let rchanges: Vec<_> = ra.diff(&rbuf);
    let mut rsink: Vec<u8> = Vec::with_capacity(1 << 20);
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rsink.clear();
            let mut back = ratatui::backend::CrosstermBackend::new(&mut rsink);
            back.draw(rchanges.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
            black_box(rsink.len());
        })
    });
    g.finish();
}


/// Non-ASCII text: the path where grapheme segmentation and width tables are
/// unavoidable, so neither framework can take a shortcut.
fn bench_unicode_set_string(c: &mut Criterion) {
    // Mixed CJK (wide), accented Latin (combining), and emoji.
    let line: String = "日本語テキスト ünïcödé 🦀🚀 ".repeat(8);
    let mut g = c.benchmark_group("unicode_set_string_full_screen");

    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    let hs = hawk::styled();
    g.bench_function("hawk", |b| {
        b.iter(|| {
            for y in 0..H {
                hb.set_string(0, y, black_box(&line), hs);
            }
        })
    });

    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    let rs = rat::styled();
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            for y in 0..H {
                rb.set_string(0, y, black_box(&line), rs);
            }
        })
    });

    let mut sb = slt::buffer::Buffer::empty(light::area());
    let ss = light::styled();
    g.bench_function("superlighttui", |b| {
        b.iter(|| {
            for y in 0..H as u32 {
                sb.set_string(0, y, black_box(&line), ss);
            }
        })
    });
    g.finish();
}

/// A line built from many differently styled spans, written to every row.
fn bench_styled_spans(c: &mut Criterion) {
    let mut g = c.benchmark_group("styled_spans_full_screen");

    let hawk_line = {
        use hawk::{Line, Span};
        let spans: Vec<Span> = (0..20)
            .map(|i| {
                let style = if i % 2 == 0 {
                    hawk::styled()
                } else {
                    hawktui::core::style::Style::default()
                        .fg(hawktui::core::style::Color::Indexed(i as u8))
                };
                Span::styled(format!("span{i:02} "), style)
            })
            .collect();
        Line::from(spans)
    };
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hawk::styled_spans(&mut hb, black_box(&hawk_line));
        })
    });

    let rat_line = {
        use rat::{Line, Span};
        let spans: Vec<Span> = (0..20)
            .map(|i| {
                let style = if i % 2 == 0 {
                    rat::styled()
                } else {
                    ratatui::style::Style::default().fg(ratatui::style::Color::Indexed(i as u8))
                };
                Span::styled(format!("span{i:02} "), style)
            })
            .collect();
        Line::from(spans)
    };
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rat::styled_spans(&mut rb, black_box(&rat_line));
        })
    });
    g.finish();
}

/// A 4-column table of 200 rows with a selected row, redrawn each iteration.
fn bench_table(c: &mut Criterion) {
    let rows: Vec<Vec<String>> = (0..200)
        .map(|i| {
            vec![
                format!("component-{i:03}.rs"),
                if i % 3 == 0 { "ok" } else { "changed" }.to_string(),
                format!("{} KB", i * 7 % 900),
                format!("2026-08-{:02} 1{}:04", i % 28 + 1, i % 10),
            ]
        })
        .collect();

    let mut g = c.benchmark_group("table_render_200_rows");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hb.reset();
            hawk::table(&mut hb, black_box(&rows), 12);
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rb.reset();
            rat::table(&mut rb, black_box(&rows), 12);
        })
    });
    g.finish();
}

/// A long list scrolled by one row per iteration — the classic TUI hot loop.
fn bench_scrolling_list(c: &mut Criterion) {
    let items: Vec<String> = (0..1000)
        .map(|i| format!("{i:04}  entry with a reasonably long label"))
        .collect();

    let mut g = c.benchmark_group("list_scroll_1000_items");
    let mut hb = hawktui::core::buffer::Buffer::empty(hawk::area());
    let mut off = 0usize;
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hb.reset();
            off = (off + 1) % 900;
            hawk::scrolling_list(&mut hb, black_box(&items), off);
        })
    });
    let mut rb = ratatui::buffer::Buffer::empty(rat::area());
    let mut roff = 0usize;
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rb.reset();
            roff = (roff + 1) % 900;
            rat::scrolling_list(&mut rb, black_box(&items), roff);
        })
    });
    g.finish();
}

/// Compositing one buffer onto another — what overlays and modals cost.
fn bench_merge(c: &mut Criterion) {
    let mut g = c.benchmark_group("buffer_merge_overlay");

    let (_, hsrc) = hawk::dirty_buffers(50);
    let mut hdst = hawktui::core::buffer::Buffer::empty(hawk::area());
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hdst.merge(black_box(&hsrc));
            black_box(&hdst);
        })
    });

    let (_, rsrc) = rat::dirty_buffers(50);
    let mut rdst = ratatui::buffer::Buffer::empty(rat::area());
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rdst.merge(black_box(&rsrc));
            black_box(&rdst);
        })
    });
    g.finish();
}

/// Emitting a frame whose cells alternate style — the worst case for an
/// encoder, since almost every cell needs new escape sequences.
fn bench_emit_style_churn(c: &mut Criterion) {
    use hawktui::backend::Backend as HawkBackend;
    use ratatui::backend::Backend as RatBackend;

    let mut g = c.benchmark_group("terminal_emit_style_churn");

    let harea = hawk::area();
    let hfront = hawktui::core::buffer::Buffer::empty(harea);
    let mut hback = hawktui::core::buffer::Buffer::empty(harea);
    for y in 0..H {
        for x in 0..W {
            let style = if (x + y) % 2 == 0 {
                hawk::styled()
            } else {
                hawktui::core::style::Style::default()
                    .fg(hawktui::core::style::Color::Indexed((x % 255) as u8))
            };
            hback.set_string(x, y, "#", style);
        }
    }
    let hchanges: Vec<_> = hfront.diff(&hback);
    let mut hsink: Vec<u8> = Vec::with_capacity(1 << 22);
    g.bench_function("hawk", |b| {
        b.iter(|| {
            hsink.clear();
            let mut backend =
                hawktui::backend::crossterm_backend::CrosstermBackend::new(&mut hsink);
            backend
                .draw(hchanges.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
            black_box(hsink.len());
        })
    });

    let rarea = rat::area();
    let rfront = ratatui::buffer::Buffer::empty(rarea);
    let mut rback = ratatui::buffer::Buffer::empty(rarea);
    for y in 0..H {
        for x in 0..W {
            let style = if (x + y) % 2 == 0 {
                rat::styled()
            } else {
                ratatui::style::Style::default().fg(ratatui::style::Color::Indexed((x % 255) as u8))
            };
            rback.set_string(x, y, "#", style);
        }
    }
    let rchanges: Vec<_> = rfront.diff(&rback);
    let mut rsink: Vec<u8> = Vec::with_capacity(1 << 22);
    g.bench_function("ratatui", |b| {
        b.iter(|| {
            rsink.clear();
            let mut backend = ratatui::backend::CrosstermBackend::new(&mut rsink);
            backend
                .draw(rchanges.iter().map(|(x, y, c)| (*x, *y, *c)))
                .unwrap();
            black_box(rsink.len());
        })
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_buffer_alloc,
    bench_buffer_reset,
    bench_diff,
    bench_set_string,
    bench_layout,
    bench_dashboard,
    bench_wrap,
    bench_emit,
    bench_unicode_set_string,
    bench_styled_spans,
    bench_table,
    bench_scrolling_list,
    bench_merge,
    bench_emit_style_churn
);
criterion_main!(benches);
