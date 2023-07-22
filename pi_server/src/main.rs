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
mod app;
use app::App;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, Result};
use tui::backend::CrosstermBackend;
use tui::Terminal;

const MAX_MESSAGE_LEN: usize = 65536;
const MAX_BUFFER_SIZE: usize = 1024;

const POLLING_PERIOD_mS: u64 = 250;

// const MAX_UTIL_WINDOW_N: usize = 60;

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
    let tcp_listener = TcpListener::bind("127.0.0.1:7878").expect("Failed bind with sender");

    let (utilbundle_producer, utilbundle_consumer) = channel();
    let tui_handler = thread::spawn(move || tui(utilbundle_consumer));

    process_incoming_threaded(tcp_listener, utilbundle_producer);
    tui_handler.join().unwrap();

    Ok(())
}

fn process_incoming_threaded(receiver_listener: TcpListener, utilbundle_producer: Sender<UtilBundle>) {
    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();
    for stream in receiver_listener.incoming() {
        let stream = stream.expect("Failed to get stream from receiver_listener");
        let producer = utilbundle_producer.clone();
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
}
