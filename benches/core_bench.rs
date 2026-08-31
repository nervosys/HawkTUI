use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hawktui::core::buffer::Buffer;
use hawktui::core::rect::Rect;
use hawktui::core::style::{Color, Modifier, Style};
use hawktui::core::text::{Line, Text};
use hawktui::layout::{Constraint, Direction, Layout};
use hawktui::widget::paragraph::Paragraph;
use hawktui::widget::Widget;

fn bench_buffer_empty(c: &mut Criterion) {
    c.bench_function("buffer_empty_80x24", |b| {
        b.iter(|| Buffer::empty(black_box(Rect::new(0, 0, 80, 24))))
    });

    c.bench_function("buffer_empty_200x50", |b| {
        b.iter(|| Buffer::empty(black_box(Rect::new(0, 0, 200, 50))))
    });
}

fn bench_buffer_reset(c: &mut Criterion) {
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));
    c.bench_function("buffer_reset_120x40", |b| {
        b.iter(|| {
            buf.reset();
            black_box(&buf);
        })
    });
}

fn bench_buffer_set_string(c: &mut Criterion) {
    let mut style = Style::default().fg(Color::Green);
    style.add_modifier = Modifier::BOLD;
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 40));

    c.bench_function("buffer_set_string_short", |b| {
        b.iter(|| {
            buf.set_string(0, 0, black_box("Hello, world!"), style);
        })
    });

    c.bench_function("buffer_set_string_long", |b| {
        let long_str = "A".repeat(100);
        b.iter(|| {
            buf.set_string(0, 0, black_box(&long_str), style);
        })
    });
}

fn bench_buffer_diff(c: &mut Criterion) {
    let area = Rect::new(0, 0, 120, 40);
    let front = Buffer::empty(area);
    let mut back = Buffer::empty(area);

    // Identical buffers — best case (no diff)
    c.bench_function("buffer_diff_identical_120x40", |b| {
        b.iter(|| {
            let updates = front.diff(&back);
            black_box(updates);
        })
    });

    // Mutate some cells so there are differences
    let style = Style::default().fg(Color::Red);
    for y in 0..40 {
        back.set_string(0, y, "changed content here!", style);
    }

    c.bench_function("buffer_diff_partial_120x40", |b| {
        b.iter(|| {
            let updates = front.diff(&back);
            black_box(updates);
        })
    });
}

fn bench_layout_split(c: &mut Criterion) {
    let area = Rect::new(0, 0, 120, 40);

    c.bench_function("layout_3_fixed", |b| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ]);
        b.iter(|| {
            let chunks = layout.split(black_box(area));
            black_box(chunks);
        })
    });

    c.bench_function("layout_mixed_10", |b| {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Min(5),
                Constraint::Max(10),
                Constraint::Ratio(1, 3),
                Constraint::Length(2),
                Constraint::Percentage(10),
                Constraint::Min(3),
                Constraint::Length(1),
                Constraint::Percentage(15),
            ]);
        b.iter(|| {
            let chunks = layout.split(black_box(area));
            black_box(chunks);
        })
    });
}

fn bench_style_patch(c: &mut Criterion) {
    let mut base = Style::default().fg(Color::White).bg(Color::Black);
    base.add_modifier = Modifier::BOLD;
    let mut overlay = Style::default().fg(Color::Red);
    overlay.add_modifier = Modifier::ITALIC;

    c.bench_function("style_patch", |b| {
        b.iter(|| {
            let result = base.patch(black_box(overlay));
            black_box(result);
        })
    });
}

fn bench_paragraph_render(c: &mut Criterion) {
    let area = Rect::new(0, 0, 80, 24);

    c.bench_function("paragraph_short", |b| {
        b.iter(|| {
            let mut buf = Buffer::empty(area);
            let p = Paragraph::new(Text::from("Hello, world!"));
            p.render(area, &mut buf);
            black_box(&buf);
        })
    });

    c.bench_function("paragraph_multiline_100", |b| {
        let lines: Vec<Line> = (0..100)
            .map(|i| {
                Line::from(format!(
                    "Line {i}: the quick brown fox jumps over the lazy dog"
                ))
            })
            .collect();
        let text = Text::from_iter(lines);
        b.iter(|| {
            let mut buf = Buffer::empty(area);
            let p = Paragraph::new(text.clone());
            p.render(area, &mut buf);
            black_box(&buf);
        })
    });
}

fn bench_agent_protocol_serde(c: &mut Criterion) {
    use hawktui::agent::protocol::{AgentRequest, AgentResponse, RequestEnvelope};

    c.bench_function("request_serialize", |b| {
        let envelope = RequestEnvelope {
            id: Some("req-42".into()),
            request: AgentRequest::ExecuteAction {
                agent_id: "editor-1".into(),
                action: "insert_text".into(),
                params: serde_json::json!({"text": "hello world", "position": 0}),
            },
        };
        b.iter(|| {
            let json = serde_json::to_string(black_box(&envelope)).unwrap();
            black_box(json);
        })
    });

    c.bench_function("request_deserialize", |b| {
        let json = r#"{"id":"req-42","type":"execute_action","agent_id":"editor-1","action":"insert_text","params":{"text":"hello world","position":0}}"#;
        b.iter(|| {
            let envelope: RequestEnvelope = serde_json::from_str(black_box(json)).unwrap();
            black_box(envelope);
        })
    });

    c.bench_function("response_serialize", |b| {
        let resp = AgentResponse::ok(serde_json::json!({
            "agent_id": "editor-1",
            "widget_type": "Editor",
            "state": {"lines": ["hello", "world"], "cursor_row": 0, "cursor_col": 5}
        }))
        .with_id("req-42");
        b.iter(|| {
            let json = serde_json::to_string(black_box(&resp)).unwrap();
            black_box(json);
        })
    });
}

criterion_group!(
    benches,
    bench_buffer_empty,
    bench_buffer_reset,
    bench_buffer_set_string,
    bench_buffer_diff,
    bench_layout_split,
    bench_style_patch,
    bench_paragraph_render,
    bench_agent_protocol_serde,
);
criterion_main!(benches);
