use std::string;
use std::time::Duration;

use crossterm::event::{read, Event, KeyCode, self};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, queue, style::Print, ExecutableCommand, Result};
use tui::backend::{CrosstermBackend, Backend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style, Modifier};
use tui::text::Span;
use tui::widgets::{Block, Borders, Chart, Dataset, Gauge, Axis};
use tui::symbols::{Marker, self};
use tui::{Terminal, Frame};

const N_CPU_CORES: usize = 8;

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

// TODO: add monotonically increasing time window? Might just be more noise
struct App {
    sig_gen: Signal,
    cpu_util: Vec<Vec<(f64, f64)>>,
    network_tx: Vec<(f64, f64)>,
    network_rx: Vec<(f64, f64)>,
    gpu_util: Vec<(f64, f64)>,
    mem_util: Vec<(f64, f64)>,
}

impl App {
    fn new () -> App {
        let mut sig = Signal::new(0.2, 3.0, 50.0);
        let data = sig.by_ref().take(200).collect::<Vec<(f64, f64)>>();

        // data1.clone().into_iter().map(|(x, y)| (x, y + rand::random::<f64>() * sig.scale)).collect::<Vec<(f64, f64)>>();
        App {
            sig_gen: sig,
            cpu_util: vec![data.clone(); N_CPU_CORES],
            network_tx: data.clone(),
            network_rx: data.clone(),
            gpu_util: data.clone(),
            mem_util: data.clone(),
        }
    }

    fn on_tick(&mut self) {
        for _ in 0..5 {
            for cpu in self.cpu_util.iter_mut() {
                cpu.remove(0);
            }

            self.network_tx.remove(0);
            self.network_rx.remove(0);
            self.gpu_util.remove(0);
            self.mem_util.remove(0);
        }
        let new_data = self.sig_gen.by_ref().take(5).collect::<Vec<(f64, f64)>>().clone();

        for cpu in self.cpu_util.iter_mut() {
            cpu.extend(new_data.clone());
        }
        self.network_tx.extend(new_data.clone());
        self.network_rx.extend(new_data.clone());
        self.gpu_util.extend(new_data.clone());
        self.mem_util.extend(new_data.clone());
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    run_app(&mut terminal, &mut app)?;

    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    let chart_data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 0.7), (4.0, 0.2)];
    let mut gauge_value = 80;
    Ok(loop {
        gauge_value += 1;
        if gauge_value > 100 {
            gauge_value = 0;
            println!("Resetting gauge value");
        }
        terminal.draw(|f| {
            ui(f, chart_data, app, gauge_value);
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
        app.on_tick();

        terminal.clear()?;
    })
}

fn ui(f: &mut Frame<'_, CrosstermBackend<std::io::Stdout>>, chart_data: [(f64, f64); 5], app: &mut App, gauge_value: i32) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(33)].as_ref())
        .split(f.size());

    let mut cpu_datasets: Vec<Dataset> = Vec::new();
    for (cpu_core, cpu_data) in app.cpu_util.iter().enumerate() {
        cpu_datasets.push(
            Dataset::default()
                .name(format!("{}{}", "cpu", cpu_core.to_string()))
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red)) // TODO: Randomly generate colors
                .data(cpu_data)
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
            .data(&app.network_rx)
    ];

    draw_cpu_util(cpu_datasets, f, chunks[0]);
    draw_network_util(network_datasets, f, chunks[1]);
    // TODO: pass (bounded) value here from app
    draw_gpu_and_mem_util(0.150, 
        0.8,
        f,
        chunks[2]);
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
        .block(Block::default()
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
                .bounds([0.0, 500.0])
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
                .bounds([0.0, 500.0]) // TODO: Update x axis bounds
        ).y_axis(
            Axis::default()
                .title("Util")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::raw("0%"),
                    Span::styled("100%", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([0.0, 100.0])
        );
    f.render_widget(chart, area);
}