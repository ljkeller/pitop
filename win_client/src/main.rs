use std::str;
use std::net::TcpStream;
use std::thread;
use std::time;
use std::io::{self, prelude::*, BufReader, Write};

use sysinfo::{System, SystemExt};

use util_bundle::UtilBundle;

const POLLING_PERIOD_MILLIS: u64 = 250;

fn send_newline_delimited_json(stream: &mut TcpStream, json_bundle: String) -> io::Result<()> {
    stream.write_all(json_bundle.as_bytes()).expect("Failed to write");
    stream.write_all(b"\n").expect("Failed to write");
    stream.flush()
}

fn main() -> io::Result<()> {
    println!("Win Client is running...");

    let mut sys = System::new_all();
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    // TODO: Use cntrl-c crate for graceful exit?
    loop {
        let bundle: UtilBundle = UtilBundle::from_refreshed_sys(&mut sys);
        println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
        let json_bundle = serde_json::to_string(&bundle).unwrap();

        // Source https://www.wikiwand.com/en/Line_Delimited_JSON
        send_newline_delimited_json(&mut stream, json_bundle)?;

        let mut reader = BufReader::new(&stream);
        let mut buffer: Vec<u8> = Vec::new();
        let bytes_returned = reader.read_until( b'\n', &mut buffer)?;

        if bytes_returned > 0 { println!("read from server: {}", str::from_utf8(&buffer).unwrap()); }
        thread::sleep(time::Duration::from_millis(POLLING_PERIOD_MILLIS));
    }
    Ok(())
}
