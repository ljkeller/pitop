use std::io;
use std::time;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

const MAX_STREAM_READS: usize = 60;
const MAX_BUFFER_SIZE: usize = 1024;

fn handle_sender(mut stream: TcpStream) -> io::Result<()> {
    let mut buf = [0; MAX_BUFFER_SIZE];
    // TODO: use BufReader to read from stream & avoid dyanmic sizing optimization issues
    // Also, make sure we are building buffer up in case we don't get all the data in one read
    for _ in 0..MAX_STREAM_READS {
        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(());
        }
        stream.write(&buf[..bytes_read])?;

        println!("from the sender: {}", String::from_utf8_lossy(&buf));
        
        // reduce overhead of looking for more client data
        thread::sleep(time::Duration::from_secs(1));
        buf = [0; MAX_BUFFER_SIZE];
    }
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
