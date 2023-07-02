use std::str;
use std::net::TcpStream;
use std::io::{self, prelude::*, BufReader, Write};

use sysinfo::{System, SystemExt, CpuExt, ComponentExt, DiskExt, NetworkExt};
use serde::{Serialize, Deserialize};

const MAX_STREAM_WRITES: usize = 60;

#[derive(Serialize, Deserialize, Debug)]
struct UtilBundle {
    cpu_usage: Vec<f32>,
    cpu_temp: f32,
    gpu_temp: f32,
    gpu_usage: f32,
    mem_used: u64,
    mem_total: u64,
    disk_used: u64,
    disk_total: u64,
    data_tx: u64,
    data_rx: u64,
}

impl UtilBundle {
    fn new() -> UtilBundle {
        UtilBundle {
            cpu_usage: Vec::new(),
            cpu_temp: 0.0,
            gpu_temp: 0.0,
            gpu_usage: 0.0,
            mem_used: 0,
            mem_total: 0,
            disk_used: 0,
            disk_total: 0,
            data_tx: 0,
            data_rx: 0,
        }
    }

    fn from_sys(sys: &System) -> UtilBundle {
        let mut bundle = UtilBundle::new();

        bundle.cpu_usage = sys.cpus().iter().map(|x| x.cpu_usage()).collect();
        bundle.cpu_temp = sys.components().iter().map(|x| x.temperature()).sum::<f32>() / sys.components().len() as f32;
        bundle.gpu_temp = 0 as f32; // TODO: IMPLEMENT
        bundle.gpu_usage = 0 as f32; // TODO: IMPLEMENT
        bundle.mem_used = sys.used_memory();
        bundle.mem_total = sys.total_memory();
        bundle.disk_used = sys.disks().iter().map(|x| x.total_space() - x.available_space()).sum::<u64>();
        bundle.disk_total = sys.disks().iter().map(|x| x.total_space()).sum::<u64>();
        bundle.data_tx = sys.networks().into_iter().map(|(_, iface)| iface.transmitted()).sum::<u64>();
        bundle.data_rx = sys.networks().into_iter().map(|(_, iface)| iface.received()).sum::<u64>();

        bundle
    }
}

fn main() -> io::Result<()> {
    println!("Win Client is running...");

    let mut sys = System::new_all();
    sys.refresh_all();
    let mut bundle = UtilBundle::from_sys(&sys);
    println!("{}", serde_json::to_string_pretty(&bundle).unwrap());

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