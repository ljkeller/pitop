use crate::App;

use rand::seq::SliceRandom;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::symbols::{self};
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Gauge};
use tui::{Frame};

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
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
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
                .style(Style::default().fg(rand_color())) 
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
    draw_gpu_and_mem_util(app.gpu_util.last().unwrap_or(&(0.0, 0.0)).1, app.mem_util.last().unwrap_or(&(0.0, 0.0)).1, f, chunks[2]);
}

fn draw_gpu_and_mem_util<B: Backend>(gpu_util: f64, mem_util: f64, f: &mut Frame<B>, area: Rect) {
    let sublayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    draw_gpu_util(gpu_util, f, sublayout[0]);
    draw_mem_util(mem_util, f, sublayout[1]);
}

fn draw_mem_util<B: Backend>(gauge_ratio: f64, f: &mut Frame<B>, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .ratio(gauge_ratio);
    f.render_widget(gauge, area);
}

fn draw_gpu_util<B: Backend>(gauge_ratio: f64, f: &mut Frame<B>, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().title("GPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .ratio(gauge_ratio);
    f.render_widget(gauge, area);
}

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

fn draw_cpu_util<B: Backend>(datasets: Vec<Dataset>, f: &mut Frame<B>, area: Rect) {
    let chart = Chart::new(datasets.to_vec())
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