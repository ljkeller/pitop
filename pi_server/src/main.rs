mod ui;
mod app;
mod terminal;
use crate::app::App;
use crate::terminal::tui;
use util_bundle::UtilBundle;use std::io;

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time;

const MAX_MESSAGE_LEN: usize = 65536;
const MAX_BUFFER_SIZE: usize = 1024;

const POLLING_PERIOD_mS: u64 = 250;

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
