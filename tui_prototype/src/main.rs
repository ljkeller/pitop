use std::time::Duration;

use crossterm::event::{read, Event, KeyCode, self};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, queue, style::Print, ExecutableCommand, Result};
use tui::backend::{CrosstermBackend, Backend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style, Modifier};
use tui::text::Span;
use tui::widgets::{Block, Borders, Chart, Dataset, Gauge, Axis};
use tui::symbols::Marker;
use tui::{Terminal, Frame};

//TODO: Implement sig gen for protype
pub struct Signal {
    x: f64,
    interval: f64,
    period: f64,
    scale: f64,
}

impl Signal {
    pub fn new(interval: f64, period: f64, scale: f64) -> Signal {
        Signal {
            x: 0.0,
            interval,
            period,
            scale,
        }
    }
}

impl Iterator for Signal {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let point_mapping = (self.x, (self.x * 1.0 / self.period).sin() * self.scale);
        self.x += self.interval;
        Some(point_mapping)
    }
}

struct App {
    sig_gen: Signal,
    cpu_util: Vec<Vec<(f64, f64)>>,
    network_tx: Vec<(f64, f64)>,
    network_rx: Vec<(f64, f64)>,
    gpu_util: Vec<(f64, f64)>,
    mem_util: Vec<(f64, f64)>,
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal)?;

    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let chart_data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 0.7), (4.0, 0.2)];
    let mut gauge_value = 80;
    Ok(loop {
        gauge_value += 1;
        if gauge_value > 100 {
            gauge_value = 0;
            println!("Resetting gauge value");
        }
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref())
                .split(f.size());

            let datasets = [Dataset::default()
                .name("Chart 1")
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&chart_data)];

            draw_cpu_util(datasets, f, chunks[0]);
            draw_network_util(gauge_value, f, chunks[1]);
            // TODO: remove numeric touchups
            draw_gpu_and_mem_util(100-gauge_value, gauge_value/2, f, chunks[2]);
        })?;

        let tick_rate = Duration::from_millis(250);
        let timeout = tick_rate;
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }

        terminal.clear()?;
    })
}

fn draw_gpu_and_mem_util<B: Backend>(gpu_util: i32, mem_util: i32, f: &mut Frame<B>, area: Rect) {
    let sublayout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);
    draw_gpu_util(gpu_util, f, sublayout[0]);
    draw_mem_util(mem_util, f, sublayout[1]);
}

fn draw_mem_util<B: Backend>(gauge_value: i32, f: &mut Frame<B>, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().title("Memory").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(gauge_value as u16);
    f.render_widget(gauge, area);
}

fn draw_gpu_util<B: Backend>(gauge_value: i32, f: &mut Frame<B>, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().title("GPU").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(gauge_value as u16);
    f.render_widget(gauge, area);
}

// TODO: implement
fn draw_network_util<B: Backend>(gauge_value: i32, f: &mut Frame<B>, area: Rect) {
    let gauge = Gauge::default()
        .block(Block::default().title("Network").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Magenta))
        .percent(gauge_value as u16);
    f.render_widget(gauge, area);
}

// TODO: implement
fn draw_cpu_util<B: Backend>(datasets: [Dataset<'_>; 1], f: &mut Frame<B>, area: Rect) {
    let chart = Chart::new(datasets.to_vec())
        .block(Block::default().title("CPU").borders(Borders::ALL))
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 5.0])
                .labels(vec![
                    Span::styled("0.0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("2.5"),
                    Span::styled("5.0", Style::default().add_modifier(Modifier::ITALIC)),
                ])
        ).y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, 1.0])
                .labels(vec![
                    Span::styled("0.0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("0.5"),
                    Span::styled("1.0", Style::default().add_modifier(Modifier::ITALIC)),
                ])
        );
    f.render_widget(chart, area);
}