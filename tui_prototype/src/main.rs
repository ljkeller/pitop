use std::time::Duration;

use crossterm::event::{read, Event, KeyCode, self};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, queue, style::Print, ExecutableCommand, Result};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style, Modifier};
use tui::text::Span;
use tui::widgets::{Block, Borders, Chart, Dataset, Gauge, Axis};
use tui::symbols::Marker;
use tui::Terminal;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    // Create a TUI terminal
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create your data for the graphs (replace with your own data)
    let chart_data = [(0.0, 0.0), (1.0, 1.0), (2.0, 0.5), (3.0, 0.7), (4.0, 0.2)];
    let mut gauge_value = 80;

    // Main event loop
    loop {
        gauge_value += 1;
        if gauge_value > 100 {
            gauge_value = 0;
            println!("Resetting gauge value");
        }
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            // Render the charts
            let datasets = [Dataset::default()
                .name("Chart 1")
                .marker(Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .data(&chart_data)];
            let chart = Chart::new(datasets.to_vec())
                .block(Block::default().title("Chart 1").borders(Borders::ALL))
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
            f.render_widget(chart, chunks[0]);

            // Render the gauges
            let gauge = Gauge::default()
                .block(Block::default().title("Memory Util").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Magenta))
                .percent(gauge_value as u16);
            f.render_widget(gauge, chunks[1]);
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
    }

    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}