use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time;
use std::vec;

use ui::draw_ui;
use util_bundle::UtilBundle;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, Result};
use tui::backend::{CrosstermBackend};
use tui::{Terminal};

const N_CPU_CORES: usize = 8;

const MAX_MESSAGE_LEN: usize = 65536;
const MAX_BUFFER_SIZE: usize = 1024;

const POLLING_PERIOD_S: u64 = 1;

fn handle_sender(mut stream: TcpStream) -> io::Result<()> {
    let mut received_data: Vec<u8> = Vec::new();
    let mut buf = [0; MAX_BUFFER_SIZE];
    // TODO: use BufReader to read from stream & avoid dyanmic sizing optimization issues?
    loop {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(());
        } else if bytes_read > MAX_MESSAGE_LEN {
            return Err(io::Error::new(io::ErrorKind::Other, "Message too long"));
        }

        received_data.extend_from_slice(&buf[..bytes_read]);
        // TODO: optimize search here to only search once for '\n' (other case is when we call .position())
        if !buf.contains(&b'\n') {
            // we haven't received the full message yet
            println!("haven't received full message yet");
            continue;
        } else {
            println!("received full message");
        }

        stream.write(&received_data)?;
        println!(
            "from the sender: {}",
            String::from_utf8_lossy(&received_data)
        );
        let util_datapoint: UtilBundle = serde_json::from_slice(
            &received_data[..received_data.iter().position(|&x| x == b'\n').unwrap_or(0)],
        )?;
        println!("util_datapoint: {:?}", util_datapoint);
        // TODO: draw TUI here

        // reduce overhead of looking for more client data
        thread::sleep(time::Duration::from_secs(POLLING_PERIOD_S));
        received_data.clear();
    }

    // TODO: clearly define when we are done with a sender?
    println!("exiting handle_sender");
    Ok(())
}

pub struct App {
    cpu_util: Vec<Vec<(f64, f64)>>,
    network_tx: Vec<(f64, f64)>,
    network_rx: Vec<(f64, f64)>,
    gpu_util: Vec<(f64, f64)>,
    mem_util: Vec<(f64, f64)>,
}

impl App {
    pub fn new() -> App {
        App {
            cpu_util: vec![vec![(0.0, 0.0); 60]; N_CPU_CORES],
            network_tx: vec![(0.0, 0.0); 60],
            network_rx: vec![(0.0, 0.0); 60],
            gpu_util: vec![(0.0, 0.0); 60],
            mem_util: vec![(0.0, 0.0); 60],
        }
    }

    pub fn on_tick(&mut self) {
        println!("on_tick");
        // for _ in 0..5 {
        //     for cpu in self.cpu_util.iter_mut() {
        //         cpu.remove(0);
        //     }

        //     self.network_tx.remove(0);
        //     self.network_rx.remove(0);
        //     self.gpu_util.remove(0);
        //     self.mem_util.remove(0);
        // }
        // let new_data = self
        //     .sig_gen
        //     .by_ref()
        //     .take(5)
        //     .collect::<Vec<(f64, f64)>>()
        //     .clone();

        // for cpu in self.cpu_util.iter_mut() {
        //     cpu.extend(new_data.clone());
        // }
        // self.network_tx.extend(new_data.clone());
        // self.network_rx.extend(new_data.clone());
        // self.gpu_util.extend(new_data.clone());
        // self.mem_util.extend(new_data.clone());
    }
}

fn tui() -> Result<()> {
    println!("tui");

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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    Ok(loop {
        terminal.draw(|f| {
            draw_ui(f, app);
        })?;

        let tick_rate = time::Duration::from_millis(250);
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

fn main() -> io::Result<()> {
    println!("Pi Server is running...");

    let tui_handler = thread::spawn(tui);

    let receiver_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed bind with sender");

    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
    for stream in receiver_listener.incoming() {
        let stream = stream.expect("Failed to get stream from receiver_listener");

        // let receiver connect with sender
        let handle = thread::spawn(move || {
            handle_sender(stream).unwrap_or_else(|error| eprintln!("{:?}", error))
        });

        thread_vec.push(handle);
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }
    tui_handler.join().unwrap();

    Ok(())
}
