use std::str;
use std::net::TcpStream;
use std::thread;
use std::time;
use std::io::{self, prelude::*, BufReader, Write};

use sysinfo::{System, SystemExt};
use serde::{Serialize, Deserialize};

use util_bundle::UtilBundle;

const MAX_STREAM_WRITES: usize = 5;
fn main() -> io::Result<()> {
    println!("Win Client is running...");

    let mut sys = System::new_all();
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    for _ in 0..MAX_STREAM_WRITES {
        let bundle = UtilBundle::from_refreshed_sys(&mut sys);
        println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
        let json_bundle = serde_json::to_string(&bundle).unwrap();

        // TODO: send a more network-friendly format over the wire
        stream.write_all(json_bundle.as_bytes()).expect("Failed to write");
        stream.flush();

        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        // TODO: send a proper termination character
        reader.read_until(b'}', &mut buffer)?;

        println!("read from server: {} \n", str::from_utf8(&buffer).unwrap());
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}