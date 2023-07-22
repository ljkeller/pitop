use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time;
use std::vec;

use ui::draw_ui;
use util_bundle::UtilBundle;
mod ui;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, Result};
use tui::backend::CrosstermBackend;
use tui::Terminal;

const MAX_MESSAGE_LEN: usize = 65536;
const MAX_BUFFER_SIZE: usize = 1024;

const POLLING_PERIOD_mS: u64 = 250;

const MAX_UTIL_WINDOW_N: usize = 60;

fn handle_sender(mut in_stream: TcpStream, out_stream: Sender<UtilBundle>) -> io::Result<()> {
    let mut received_data: Vec<u8> = Vec::new();
    let mut buf = [0; MAX_BUFFER_SIZE];
    // TODO: Use cntrl-c crate for graceful exit?
    loop {
        let bytes_read = in_stream.read(&mut buf)?;

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

        in_stream.write(&received_data)?;
        println!(
            "from the sender: {}",
            String::from_utf8_lossy(&received_data)
        );
        let util_datapoint: UtilBundle = serde_json::from_slice(
            &received_data[..received_data.iter().position(|&x| x == b'\n').unwrap_or(0)],
        )?;
        println!("util_datapoint: {:?}", util_datapoint);
        out_stream.send(util_datapoint);

        // reduce overhead of looking for more client data
        thread::sleep(time::Duration::from_millis(POLLING_PERIOD_mS));
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
            cpu_util: vec![],
            network_tx: vec![],
            network_rx: vec![],
            gpu_util: vec![],
            mem_util: vec![],
        }
    }

    // TODO: Optimize
    pub fn on_tick(&mut self, datapoint: UtilBundle) {
        println!("on_tick");

        if self.cpu_util.len() > MAX_UTIL_WINDOW_N {
            self.cpu_util.remove(0);
        }
        if self.network_tx.len() > MAX_UTIL_WINDOW_N {
            self.network_tx.remove(0);
        }
        if self.network_rx.len() > MAX_UTIL_WINDOW_N {
            self.network_rx.remove(0);
        }
        if self.gpu_util.len() > MAX_UTIL_WINDOW_N {
            self.gpu_util.remove(0);
        }
        if self.mem_util.len() > MAX_UTIL_WINDOW_N {
            self.mem_util.remove(0);
        }

        // could just pop, push, then add 1 to all x values
        self.cpu_util.push(
            datapoint
                .cpu_usage
                .iter()
                .map(|f| (0 as f64, *f as f64))
                .collect(),
        );
        self.network_tx.push((0 as f64, datapoint.data_tx as f64));
        self.network_rx.push((0 as f64, datapoint.data_rx as f64));
        self.gpu_util.push((0 as f64, datapoint.gpu_usage as f64));
        // TODO: never divide by 0 (wont be an issue once sharing info between threads)
        if datapoint.mem_total > 0 {
            self.mem_util.push((
                0 as f64,
                datapoint.mem_used as f64 / datapoint.mem_total as f64,
            ));
        } else {
            self.mem_util.push((0.0, 0.0));
        }

        self.cpu_util
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, v)| v.iter_mut().for_each(|(t, y)| *t = i as f64));
        self.network_tx
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.network_rx
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.gpu_util
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.mem_util
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
    }
}

fn tui(datastream_in: Receiver<UtilBundle>) -> Result<()> {
    println!("tui");

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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    datastream_in: Receiver<UtilBundle>,
) -> Result<()> {
    Ok(loop {
        terminal.draw(|f| {
            draw_ui(f, app);
        })?;

        // TODO: Update tick-rate logic to be more accurate
        // TODO: Create relationship between client and server data rates
        let tick_rate = time::Duration::from_millis(POLLING_PERIOD_mS);
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
            println!("Generate 0 datapoint");
            app.on_tick(UtilBundle::new());
        }

        terminal.clear()?;
    })
}

fn main() -> io::Result<()> {
    println!("Pi Server is running...");
    let (producer, consumer) = channel();

    let tui_handler = thread::spawn(move || tui(consumer));

    let receiver_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed bind with sender");

    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
    for stream in receiver_listener.incoming() {
        let stream = stream.expect("Failed to get stream from receiver_listener");
        let producer = producer.clone();
        // let receiver connect with sender
        // Might have to use Arc/Mutex here?
        let handle = thread::spawn(move || {
            handle_sender(stream, producer).unwrap_or_else(|error| eprintln!("{:?}", error))
        });

        thread_vec.push(handle);
    }

    for handle in thread_vec {
        handle.join().unwrap();
    }
    tui_handler.join().unwrap();

    Ok(())
}
