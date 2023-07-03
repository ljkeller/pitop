use std::str;
use std::net::TcpStream;
use std::thread;
use std::time;
use std::io::{self, prelude::*, BufReader, Write};

use sysinfo::{System, SystemExt, CpuExt, ComponentExt, DiskExt, NetworkExt};
use serde::{Serialize, Deserialize};

const MAX_STREAM_WRITES: usize = 5;

// todo: make a bundle lib
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

    fn from_refreshed_sys(sys: &mut System) -> UtilBundle {
        sys.refresh_all();
        UtilBundle::from_sys(&sys)
    }
}

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