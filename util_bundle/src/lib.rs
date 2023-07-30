// Purpose: Library for bundling system utilization data

use std::process::{Command, Stdio};

use sysinfo::{System, SystemExt, CpuExt, ComponentExt, DiskExt, NetworkExt};
use serde::{Serialize, Deserialize};

fn get_nvidia_smi_output() -> Result<String, std::io::Error> {
    let output = Command::new("nvidia-smi")
        .arg("--query-gpu=power.draw,power.limit")
        .arg("--format=csv,noheader,nounits")
        .output()?
        .stdout;

    String::from_utf8(output).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

fn parse_nvidia_smi_output(output: String) -> Option<(f64, f64)> {
    let mut lines = output.lines();
    if let Some(line) = lines.next() {
        let mut stats = line.split(",");
        if let (Some(power_draw_str), Some(power_max_str)) = (stats.next(), stats.next()) {
            if let (Ok(power_draw), Ok(power_max)) = (power_draw_str.trim().parse::<f64>(), power_max_str.trim().parse::<f64>()) {
                return Some((power_draw, power_max));
            }
        }
    }
    None
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UtilBundle {
    pub cpu_usage: Vec<f32>,
    pub cpu_temp: f32,
    pub gpu_power: f64,
    pub gpu_power_limit: f64,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub data_tx: u64,
    pub data_rx: u64,
}

// TODO: Should I make NvidiaBundle an optiopnal field of UtilBundle?
pub struct NvidiaBundle {
    pub gpu_power_draw: f64,
    pub gpu_power_limit: f64
}

impl UtilBundle {
    pub fn new() -> UtilBundle {
        UtilBundle {
            cpu_usage: Vec::new(),
            cpu_temp: 0.0,
            gpu_power: 0.0,
            gpu_power_limit: 0.0,
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
        // !ERROR: This returns null if program ran without sufficient perms (admin)
        bundle.cpu_temp = sys.components().iter().map(|x| x.temperature()).sum::<f32>() / sys.components().len() as f32;
        if let Ok(nvidia_smi_output) = get_nvidia_smi_output() {
            if let Some((power_draw, power_max)) = parse_nvidia_smi_output(nvidia_smi_output) {
                bundle.gpu_power = power_draw;
                bundle.gpu_power_limit = power_max;
            } else {
                bundle.gpu_power = 0.0;
                bundle.gpu_power_limit = 0.0;
            }
        } else {
            bundle.gpu_power = 0.0;
            bundle.gpu_power_limit = 0.0;
        }
        bundle.mem_used = sys.used_memory();
        bundle.mem_total = sys.total_memory();
        bundle.disk_used = sys.disks().iter().map(|x| x.total_space() - x.available_space()).sum::<u64>();
        bundle.disk_total = sys.disks().iter().map(|x| x.total_space()).sum::<u64>();
        bundle.data_tx = sys.networks().into_iter().map(|(_, iface)| iface.transmitted()).sum::<u64>();
        bundle.data_rx = sys.networks().into_iter().map(|(_, iface)| iface.received()).sum::<u64>();

        bundle
    }

    pub fn from_refreshed_sys(sys: &mut System) -> UtilBundle {
        sys.refresh_all();
        UtilBundle::from_sys(&sys)
    }
} 

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_init() {
        let result = UtilBundle::new();
        assert!(result.cpu_usage.is_empty());
    }
}
