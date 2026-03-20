//! btop-style system monitor TUI — built with Louie.
//!
//! Replicates the layout of btop (~22k GitHub stars), the most popular
//! terminal system monitor:
//!
//! - CPU usage per-core gauges + sparkline history
//! - Memory/Swap gauges + composition bar
//! - Network I/O sparklines (upload/download)
//! - Process table (sorted, selectable)
//! - Animated data with tick-driven updates

use louie::ontology::registry::OntologyRegistry;
use louie::prelude::*;
use louie::runtime::{Command, Model, Program, ProgramOptions};
use louie::widget::barchart::BarChart;
use louie::widget::gauge::Gauge;
use louie::widget::line_gauge::LineGauge;
use louie::widget::sparkline::Sparkline;
use louie::widget::table::{Table, TableColumn, TableColumnWidth, TableRow, TableState};
use std::time::Duration;

// ── Palette ─────────────────────────────────────────────────────────────────

const BORDER: Color = Color::DarkGray;
const TITLE: Color = Color::White;
const CPU_COLORS: [Color; 8] = [
    Color::Green,
    Color::Cyan,
    Color::Blue,
    Color::Yellow,
    Color::Magenta,
    Color::Red,
    Color::LightGreen,
    Color::LightCyan,
];
const MEM_USED: Color = Color::Green;
const MEM_CACHE: Color = Color::Blue;
const SWAP: Color = Color::Yellow;
const NET_DOWN: Color = Color::Green;
const NET_UP: Color = Color::Magenta;
const DIM: Color = Color::DarkGray;
const PROC_HEADER: Color = Color::Cyan;

// ── Simulated system data ───────────────────────────────────────────────────

const NUM_CORES: usize = 8;
const HISTORY_LEN: usize = 60;

/// Simple LCG for deterministic "random" values without external deps.
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state >> 33
    }
    fn next_range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }
}

struct ProcessInfo {
    pid: u32,
    name: String,
    cpu: f64,
    mem_mb: u32,
    threads: u16,
    state: &'static str,
}

fn sample_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 1,
            name: "systemd".into(),
            cpu: 0.1,
            mem_mb: 12,
            threads: 1,
            state: "S",
        },
        ProcessInfo {
            pid: 842,
            name: "Xorg".into(),
            cpu: 3.2,
            mem_mb: 156,
            threads: 4,
            state: "S",
        },
        ProcessInfo {
            pid: 1024,
            name: "rust-analyzer".into(),
            cpu: 18.4,
            mem_mb: 1820,
            threads: 12,
            state: "R",
        },
        ProcessInfo {
            pid: 1337,
            name: "cargo build".into(),
            cpu: 94.2,
            mem_mb: 2340,
            threads: 16,
            state: "R",
        },
        ProcessInfo {
            pid: 2001,
            name: "firefox".into(),
            cpu: 12.7,
            mem_mb: 3200,
            threads: 48,
            state: "S",
        },
        ProcessInfo {
            pid: 2222,
            name: "code".into(),
            cpu: 8.1,
            mem_mb: 890,
            threads: 22,
            state: "S",
        },
        ProcessInfo {
            pid: 3001,
            name: "pulseaudio".into(),
            cpu: 0.8,
            mem_mb: 24,
            threads: 3,
            state: "S",
        },
        ProcessInfo {
            pid: 3500,
            name: "docker".into(),
            cpu: 2.1,
            mem_mb: 340,
            threads: 8,
            state: "S",
        },
        ProcessInfo {
            pid: 4000,
            name: "postgres".into(),
            cpu: 1.5,
            mem_mb: 128,
            threads: 6,
            state: "S",
        },
        ProcessInfo {
            pid: 4200,
            name: "node".into(),
            cpu: 5.3,
            mem_mb: 420,
            threads: 10,
            state: "S",
        },
        ProcessInfo {
            pid: 4500,
            name: "btop".into(),
            cpu: 0.4,
            mem_mb: 18,
            threads: 2,
            state: "R",
        },
        ProcessInfo {
            pid: 5000,
            name: "sshd".into(),
            cpu: 0.0,
            mem_mb: 8,
            threads: 1,
            state: "S",
        },
        ProcessInfo {
            pid: 5500,
            name: "tmux".into(),
            cpu: 0.1,
            mem_mb: 6,
            threads: 1,
            state: "S",
        },
        ProcessInfo {
            pid: 6000,
            name: "alacritty".into(),
            cpu: 1.2,
            mem_mb: 92,
            threads: 4,
            state: "S",
        },
    ]
}

// ── App state ───────────────────────────────────────────────────────────────

struct App {
    rng: Rng,
    tick: u64,

    // CPU
    cpu_usage: [f64; NUM_CORES], // current usage per core (0.0–100.0)
    cpu_total: f64,              // aggregate
    cpu_history: Vec<u64>,       // total CPU sparkline history (0–100)
    core_histories: [Vec<u64>; NUM_CORES],

    // Memory (MB)
    mem_total: u64,
    mem_used: u64,
    mem_cached: u64,
    swap_total: u64,
    swap_used: u64,

    // Network (KB/s)
    net_down_history: Vec<u64>,
    net_up_history: Vec<u64>,
    net_down_current: u64,
    net_up_current: u64,
    net_total_down: u64,
    net_total_up: u64,

    // Processes
    processes: Vec<ProcessInfo>,
    proc_state: TableState,

    // Temperatures
    cpu_temp: u32,
    gpu_temp: u32,
}

enum Msg {
    Tick,
    ProcDown,
    ProcUp,
    ProcTop,
    ProcBottom,
    Quit,
}

impl App {
    fn new() -> Self {
        let mut rng = Rng::new(42);
        let mut cpu_history = Vec::with_capacity(HISTORY_LEN);
        let mut core_histories: [Vec<u64>; NUM_CORES] = Default::default();
        let mut net_down_history = Vec::with_capacity(HISTORY_LEN);
        let mut net_up_history = Vec::with_capacity(HISTORY_LEN);

        // Seed history with plausible data
        for _ in 0..HISTORY_LEN {
            cpu_history.push(rng.next_range(20, 60));
            net_down_history.push(rng.next_range(100, 2000));
            net_up_history.push(rng.next_range(10, 500));
            for core in core_histories.iter_mut() {
                core.push(rng.next_range(5, 95));
            }
        }

        let mut cpu_usage = [0.0; NUM_CORES];
        for (_i, usage) in cpu_usage.iter_mut().enumerate() {
            *usage = rng.next_range(5, 95) as f64;
        }

        let mut processes = sample_processes();
        processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());

        Self {
            rng,
            tick: 0,
            cpu_usage,
            cpu_total: cpu_usage.iter().sum::<f64>() / NUM_CORES as f64,
            cpu_history,
            core_histories,
            mem_total: 32768,
            mem_used: 12400,
            mem_cached: 4200,
            swap_total: 8192,
            swap_used: 512,
            net_down_history,
            net_up_history,
            net_down_current: 1200,
            net_up_current: 340,
            net_total_down: 0,
            net_total_up: 0,
            processes,
            proc_state: TableState::new(),
            cpu_temp: 62,
            gpu_temp: 55,
        }
    }

    fn simulate_tick(&mut self) {
        self.tick += 1;

        // Evolve CPU per-core with some random walk
        for i in 0..NUM_CORES {
            let delta = self.rng.next_range(0, 15) as f64 - 7.0;
            self.cpu_usage[i] = (self.cpu_usage[i] + delta).clamp(2.0, 99.0);
            if self.core_histories[i].len() >= HISTORY_LEN {
                self.core_histories[i].remove(0);
            }
            self.core_histories[i].push(self.cpu_usage[i] as u64);
        }
        self.cpu_total = self.cpu_usage.iter().sum::<f64>() / NUM_CORES as f64;

        if self.cpu_history.len() >= HISTORY_LEN {
            self.cpu_history.remove(0);
        }
        self.cpu_history.push(self.cpu_total as u64);

        // Memory fluctuation
        let mem_delta = self.rng.next_range(0, 200) as i64 - 100;
        self.mem_used = (self.mem_used as i64 + mem_delta).clamp(4000, 28000) as u64;
        let cache_delta = self.rng.next_range(0, 100) as i64 - 50;
        self.mem_cached = (self.mem_cached as i64 + cache_delta).clamp(1000, 8000) as u64;
        let swap_delta = self.rng.next_range(0, 50) as i64 - 25;
        self.swap_used = (self.swap_used as i64 + swap_delta).clamp(0, 4000) as u64;

        // Network
        self.net_down_current = self.rng.next_range(200, 5000);
        self.net_up_current = self.rng.next_range(20, 1500);
        self.net_total_down += self.net_down_current;
        self.net_total_up += self.net_up_current;

        if self.net_down_history.len() >= HISTORY_LEN {
            self.net_down_history.remove(0);
        }
        self.net_down_history.push(self.net_down_current);

        if self.net_up_history.len() >= HISTORY_LEN {
            self.net_up_history.remove(0);
        }
        self.net_up_history.push(self.net_up_current);

        // Temperatures
        self.cpu_temp =
            (self.cpu_temp as i32 + self.rng.next_range(0, 5) as i32 - 2).clamp(40, 95) as u32;
        self.gpu_temp =
            (self.gpu_temp as i32 + self.rng.next_range(0, 3) as i32 - 1).clamp(35, 85) as u32;

        // Jiggle process CPU values
        for proc in &mut self.processes {
            let delta = (self.rng.next_range(0, 10) as f64 - 5.0) * 0.3;
            proc.cpu = (proc.cpu + delta).clamp(0.0, 100.0);
        }
        self.processes
            .sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap());
    }
}

// ── Model impl ──────────────────────────────────────────────────────────────

impl Model for App {
    type Msg = Msg;

    fn update(&mut self, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Tick => {
                self.simulate_tick();
                Command::None
            }
            Msg::ProcDown => {
                self.proc_state.select_next(self.processes.len());
                Command::None
            }
            Msg::ProcUp => {
                self.proc_state.select_previous();
                Command::None
            }
            Msg::ProcTop => {
                self.proc_state.select(Some(0));
                Command::None
            }
            Msg::ProcBottom => {
                if !self.processes.is_empty() {
                    self.proc_state.select(Some(self.processes.len() - 1));
                }
                Command::None
            }
            Msg::Quit => Command::Quit,
        }
    }

    fn view(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        // ┌───────────────────┬────────────────────┐
        // │   CPU (gauges +   │   Memory / Swap     │
        // │   sparkline)      │   + Network         │
        // ├───────────────────┴────────────────────┤
        // │         Process table                   │
        // └─────────────────────────────────────────┘

        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(45), // top: CPU + Mem/Net
                Constraint::Min(8),         // bottom: processes
            ])
            .split(area);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(55), // CPU panel
                Constraint::Percentage(45), // Mem + Net
            ])
            .split(main[0]);

        self.render_cpu_panel(frame, top[0]);

        // Right side: Mem + Net stacked
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(55), // Memory
                Constraint::Percentage(45), // Network
            ])
            .split(top[1]);

        self.render_mem_panel(frame, right[0]);
        self.render_net_panel(frame, right[1]);
        self.render_proc_panel(frame, main[1]);
    }

    fn handle_event(&self, event: Event) -> Option<Msg> {
        match event {
            Event::Tick => Some(Msg::Tick),
            Event::Key(key) => {
                if key.is_ctrl(KeyCode::Char('c')) {
                    return Some(Msg::Quit);
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => Some(Msg::Quit),
                    KeyCode::Char('j') | KeyCode::Down => Some(Msg::ProcDown),
                    KeyCode::Char('k') | KeyCode::Up => Some(Msg::ProcUp),
                    KeyCode::Char('g') | KeyCode::Home => Some(Msg::ProcTop),
                    KeyCode::Char('G') | KeyCode::End => Some(Msg::ProcBottom),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn init(&self) -> Command<Msg> {
        Command::SetTickRate(Duration::from_millis(500))
    }

    fn register_ontology(&self, registry: &mut OntologyRegistry) {
        registry.register::<Block>();
        registry.register::<Paragraph>();
        registry.register::<Gauge>();
        registry.register::<LineGauge>();
        registry.register::<Sparkline>();
        registry.register::<BarChart>();
        registry.register::<Table>();
    }
}

// ── View helpers ────────────────────────────────────────────────────────────

impl App {
    fn render_cpu_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .title(Line::styled(
                format!(" CPU {:.0}%  {}°C ", self.cpu_total, self.cpu_temp),
                Style::default().fg(TITLE).bold(),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 4 || inner.width < 10 {
            return;
        }

        // Split: core gauges on left, sparkline on right
        let halves = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(inner);

        // Per-core line gauges
        let gauge_area = halves[0];
        let rows_per_core = 1u16;
        let max_cores = (gauge_area.height as usize).min(NUM_CORES);

        for i in 0..max_cores {
            let y = gauge_area.y + i as u16 * rows_per_core;
            if y >= gauge_area.bottom() {
                break;
            }
            let core_area = Rect::new(gauge_area.x, y, gauge_area.width, 1);
            let pct = self.cpu_usage[i] / 100.0;
            let color = CPU_COLORS[i % CPU_COLORS.len()];

            let label = format!("C{} {:3.0}%", i, self.cpu_usage[i]);
            let gauge = LineGauge::new()
                .ratio(pct)
                .label(Span::styled(label, Style::default().fg(color)))
                .line_style(Style::default().fg(color))
                .style(Style::default().fg(DIM));

            frame.render_widget(gauge, core_area);
        }

        // CPU total sparkline
        let spark_area = halves[1];
        if spark_area.width > 2 && spark_area.height > 1 {
            let sparkline = Sparkline::new(self.cpu_history.clone())
                .max(100)
                .bar_style(Style::default().fg(Color::Green))
                .block(
                    Block::default()
                        .title(Span::styled(" Total ", Style::default().fg(DIM)))
                        .borders(Borders::LEFT)
                        .border_style(Style::default().fg(BORDER)),
                );
            frame.render_widget(sparkline, spark_area);
        }
    }

    fn render_mem_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let mem_pct = self.mem_used as f64 / self.mem_total as f64;
        let swap_pct = if self.swap_total > 0 {
            self.swap_used as f64 / self.swap_total as f64
        } else {
            0.0
        };

        let title = format!(
            " Memory {:.1} / {:.1} GiB ({:.0}%) ",
            self.mem_used as f64 / 1024.0,
            self.mem_total as f64 / 1024.0,
            mem_pct * 100.0
        );

        let block = Block::default()
            .title(Line::styled(title, Style::default().fg(TITLE).bold()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 3 || inner.width < 10 {
            return;
        }

        // Stack: RAM gauge, cache gauge, swap gauge, disk info
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // RAM used
                Constraint::Length(1), // RAM cached
                Constraint::Length(1), // Swap
                Constraint::Length(1), // blank
                Constraint::Min(1),    // details
            ])
            .split(inner);

        // RAM used gauge
        let ram_label = format!("Used  {:>5} MiB", self.mem_used);
        let ram_gauge = LineGauge::new()
            .ratio(mem_pct)
            .label(Span::styled(ram_label, Style::default().fg(MEM_USED)))
            .line_style(Style::default().fg(MEM_USED))
            .style(Style::default().fg(DIM));
        frame.render_widget(ram_gauge, rows[0]);

        // Cache
        let cache_pct = self.mem_cached as f64 / self.mem_total as f64;
        let cache_label = format!("Cache {:>5} MiB", self.mem_cached);
        let cache_gauge = LineGauge::new()
            .ratio(cache_pct)
            .label(Span::styled(cache_label, Style::default().fg(MEM_CACHE)))
            .line_style(Style::default().fg(MEM_CACHE))
            .style(Style::default().fg(DIM));
        frame.render_widget(cache_gauge, rows[1]);

        // Swap
        let swap_label = format!("Swap  {:>5} MiB", self.swap_used);
        let swap_gauge = LineGauge::new()
            .ratio(swap_pct)
            .label(Span::styled(swap_label, Style::default().fg(SWAP)))
            .line_style(Style::default().fg(SWAP))
            .style(Style::default().fg(DIM));
        frame.render_widget(swap_gauge, rows[2]);

        // Detail text
        if rows[4].height > 0 {
            let detail = Paragraph::new(vec![Line::from(vec![
                Span::styled("  Free: ", Style::default().fg(DIM)),
                Span::styled(
                    format!(
                        "{:.1} GiB",
                        (self.mem_total - self.mem_used - self.mem_cached) as f64 / 1024.0
                    ),
                    Style::default().fg(Color::White),
                ),
                Span::styled("   GPU: ", Style::default().fg(DIM)),
                Span::styled(
                    format!("{}°C", self.gpu_temp),
                    Style::default().fg(if self.gpu_temp > 70 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ])]);
            frame.render_widget(detail, rows[4]);
        }
    }

    fn render_net_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let title = format!(
            " Network  ▼ {} KB/s  ▲ {} KB/s ",
            self.net_down_current, self.net_up_current
        );

        let block = Block::default()
            .title(Line::styled(title, Style::default().fg(TITLE).bold()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height < 2 || inner.width < 10 {
            return;
        }

        // Split top/bottom for download/upload sparklines
        let halves = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        // Download sparkline
        let down_spark = Sparkline::new(self.net_down_history.clone())
            .bar_style(Style::default().fg(NET_DOWN))
            .block(
                Block::default()
                    .title(Span::styled(
                        format!(
                            " ▼ Download  total: {:.1} MB ",
                            self.net_total_down as f64 / 1024.0
                        ),
                        Style::default().fg(NET_DOWN),
                    ))
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(BORDER)),
            );
        frame.render_widget(down_spark, halves[0]);

        // Upload sparkline
        let up_spark = Sparkline::new(self.net_up_history.clone())
            .bar_style(Style::default().fg(NET_UP))
            .block(Block::default().title(Span::styled(
                format!(
                    " ▲ Upload    total: {:.1} MB ",
                    self.net_total_up as f64 / 1024.0
                ),
                Style::default().fg(NET_UP),
            )));
        frame.render_widget(up_spark, halves[1]);
    }

    fn render_proc_panel(&self, frame: &mut Frame<'_>, area: Rect) {
        let columns = vec![
            TableColumn::new("PID", TableColumnWidth::Fixed(7)),
            TableColumn::new("Name", TableColumnWidth::Fill),
            TableColumn::new("CPU%", TableColumnWidth::Fixed(7)),
            TableColumn::new("MEM", TableColumnWidth::Fixed(8)),
            TableColumn::new("THR", TableColumnWidth::Fixed(5)),
            TableColumn::new("S", TableColumnWidth::Fixed(3)),
        ];

        let rows: Vec<TableRow> = self
            .processes
            .iter()
            .map(|p| {
                let cpu_color = if p.cpu > 50.0 {
                    Color::Red
                } else if p.cpu > 20.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };

                TableRow::new(vec![
                    Line::styled(format!("{:>6}", p.pid), Style::default().fg(DIM)),
                    Line::styled(p.name.clone(), Style::default().fg(Color::White)),
                    Line::styled(format!("{:>5.1}%", p.cpu), Style::default().fg(cpu_color)),
                    Line::styled(format!("{:>5} MB", p.mem_mb), Style::default().fg(MEM_USED)),
                    Line::styled(format!("{:>4}", p.threads), Style::default().fg(DIM)),
                    Line::styled(
                        p.state.to_string(),
                        Style::default().fg(if p.state == "R" { Color::Green } else { DIM }),
                    ),
                ])
            })
            .collect();

        let proc_count = self.processes.len();
        let cpu_total: f64 = self.processes.iter().map(|p| p.cpu).sum();

        let table = Table::new(columns, rows)
            .block(
                Block::default()
                    .title(Line::styled(
                        format!(" Processes ({})  CPU: {:.1}% ", proc_count, cpu_total),
                        Style::default().fg(TITLE).bold(),
                    ))
                    .title_bottom(Line::styled(
                        " j/k navigate  q quit ",
                        Style::default().fg(DIM),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(BORDER)),
            )
            .header_style(Style::default().fg(PROC_HEADER).bold())
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Green))
            .column_spacing(1);

        let mut state = self.proc_state.clone();
        frame.render_stateful_widget(table, area, &mut state);
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> std::io::Result<()> {
    let app = App::new();
    let backend = CrosstermBackend::new(std::io::stdout());
    let options = ProgramOptions {
        tick_rate: Some(Duration::from_millis(500)),
        ..Default::default()
    };
    Program::new(app, backend)?.with_options(options).run()?;
    Ok(())
}
