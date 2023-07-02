use std::str;
use std::net::TcpStream;
use std::io::{self, prelude::*, BufReader, Write};

const MAX_STREAM_WRITES: usize = 60;

fn main() -> io::Result<()> {
    println!("Win Client is running...");

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    for _ in 0..MAX_STREAM_WRITES {
        let mut input = String::new();
        
        println!("Enter a message:");
        io::stdin().read_line(&mut input).expect("Failed to read");

        stream.write(input.as_bytes()).expect("Failed to write");

        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        reader.read_until(b'\n', &mut buffer)?;

        println!("read from server: {} \n", str::from_utf8(&buffer).unwrap());
    }

    Ok(())
}