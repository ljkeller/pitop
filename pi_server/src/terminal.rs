use crate::ui::ColorGenerator;
use crate::{UtilBundle, ui::draw_ui, app::App, POLLING_PERIOD_MILLIS};

use std::sync::mpsc::{Receiver};
use std::time;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, Result};
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    datastream_in: Receiver<UtilBundle>,
) -> Result<()> {
    let mut color_gen: ColorGenerator = ColorGenerator::new();
    Ok(loop {
        terminal.draw(|f| {
            draw_ui(f, app, &mut color_gen);
        })?;

        // TODO: Update tick-rate logic to be more accurate
        // TODO: Create relationship between client and server data rates
        let tick_rate = time::Duration::from_millis(POLLING_PERIOD_MILLIS);
        let timeout = tick_rate;
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if let Ok(datapoint) = datastream_in.recv_timeout(tick_rate) {
            app.on_tick(datapoint);
        } else {
            // println!("Generate 0 datapoint");
            app.on_tick(UtilBundle::new());
        }

        terminal.clear()?;
    })
}

pub fn tui(datastream_in: Receiver<UtilBundle>) -> Result<()> {
    // println!("tui");

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    run_app(&mut terminal, &mut app, datastream_in)?;

    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    disable_raw_mode()?;
    Ok(())
}
