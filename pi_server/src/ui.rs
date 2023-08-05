use crate::App;

use rand::seq::SliceRandom;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::{self};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge};
use tui::{Frame};

pub struct ColorGenerator {
    idx_to_color: Vec<Color>
}

impl ColorGenerator {
    pub fn new() -> ColorGenerator {
        ColorGenerator { idx_to_color: Vec::new() }
    }

    // memoized so that each cpu core always has the same color as before
    pub fn idx_to_color_persistant(&mut self, idx: usize) -> Color {
        if idx < self.idx_to_color.len() {
            self.idx_to_color[idx];
        }

        while idx >= self.idx_to_color.len() {
            self.idx_to_color.push(rand_color());
        }
        self.idx_to_color[idx]
    }
}

fn rand_color() -> Color {
    let color_wheel = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];

    color_wheel.choose(&mut rand::thread_rng()).unwrap().clone()
}

pub fn draw_ui(
    f: &mut Frame<'_, CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    color_gen: &mut ColorGenerator,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.size());

    let mut cpu_datasets: Vec<Dataset> = Vec::new();
    for (cpu_core, cpu_data) in app.cpu_util.iter().enumerate() {
        cpu_datasets.push(
            Dataset::default()
                .name(format!("{}{}", "cpu", cpu_core.to_string()))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(color_gen.idx_to_color_persistant(cpu_core))) 
                .data(cpu_data),
        );
    }

    let network_datasets = vec![
        Dataset::default()
            .name("Tx")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.network_tx),
        Dataset::default()
            .name("Rx")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Red))
            .data(&app.network_rx),
    ];

    draw_cpu_util(cpu_datasets, f, chunks[0]);
    draw_network_util(network_datasets, f, chunks[1]);
    // TODO: pass (bounded) value here from app

    draw_gpu_and_mem_util(get_gpu_ratio(app.gpu_power_draw.last(), app.gpu_power_limit), 
        app.gpu_power_limit,
        app.mem_util.last().unwrap_or(&(0.0, 0.0)).1,
        app.mem_total_bytes,
        f,
        chunks[2]);
}

fn get_gpu_ratio(gpu_power_draw: Option<&(f64, f64)>, max_gpu_power: f64) -> f64 {
    if max_gpu_power == 0.0 { return 0.0; }

    if let Some((x, active_draw)) = gpu_power_draw {
        (active_draw/max_gpu_power).min(1.0).max(0.0)
    } else {
        0.0
    }
}

fn draw_gpu_and_mem_util<B: Backend>(gpu_power_draw: f64, gpu_power_limit: f64, mem_util: f64, mem_total: u64, f: &mut Frame<B>, area: Rect) {
    let sublayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    draw_gpu_power_draw(gpu_power_draw, gpu_power_limit, f, sublayout[0]);
    draw_mem_util(mem_util, mem_total, f, sublayout[1]);
}

fn draw_mem_util<B: Backend>(gauge_ratio: f64, mem_total_bytes: u64, f: &mut Frame<B>, area: Rect) {
    let mem_title = format!("Memory ({}GB)", mem_total_bytes/1024/1024/1024);
    let gauge = Gauge::default()
        .block(Block::default().title(mem_title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(gauge_ratio);
    f.render_widget(gauge, area);
}

fn draw_gpu_power_draw<B: Backend>(gauge_ratio: f64, gpu_power_limit: f64, f: &mut Frame<B>, area: Rect) {
    let title = format!("GPU (Limit {}W)", gpu_power_limit);
    let gauge = Gauge::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(gauge_ratio);
    f.render_widget(gauge, area);
}

// TODO: Should I dynamically size the y axis label & bounds?
fn draw_network_util<B: Backend>(datasets: Vec<Dataset>, f: &mut Frame<B>, area: Rect) {
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Network",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray)) //TODO: add labels, dynamic bounds? or just hold static last X ticks
                .bounds([0.0, 60.0]),
        )
        .y_axis(
            Axis::default()
                .title("kbps")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::raw("0"),
                    Span::styled("50.0", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 50.0]),
        );
    f.render_widget(chart, area);
}

fn draw_cpu_util<B: Backend>(datasets: Vec<Dataset>, f: &mut Frame<B>, area: Rect) {
    let chart = Chart::new(datasets)
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 60.0]), // TODO: Update x axis bounds
        )
        .y_axis(
            Axis::default()
                .title("Util")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::raw("0%"),
                    Span::styled("100%", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0]),
        );
    f.render_widget(chart, area);
}