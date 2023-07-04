use std::io;
use std::time;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

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
        if !buf.contains(&b'\n') {
            // we haven't received the full message yet
            println!("haven't received full message yet");
            continue;
        } else {
            println!("received full message");
        }

        stream.write(&received_data)?;
        println!("from the sender: {}", String::from_utf8_lossy(&received_data));
        
        // reduce overhead of looking for more client data
        thread::sleep(time::Duration::from_secs(POLLING_PERIOD_S));
        received_data.clear();
    }

    // TODO: clearly define when we are done with a sender?
    println!("exiting handle_sender");
    Ok(())
}

fn main() -> io::Result<()> {
    println!("Pi Server is running...");

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

    Ok(())
}
