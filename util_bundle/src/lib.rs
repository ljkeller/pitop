// Purpose: Library for bundling system utilization data

use sysinfo::{System, SystemExt, CpuExt, ComponentExt, DiskExt, NetworkExt};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UtilBundle {
    pub cpu_usage: Vec<f32>,
    pub cpu_temp: f32,
    pub gpu_temp: f32,
    pub gpu_usage: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub data_tx: u64,
    pub data_rx: u64,
}

impl UtilBundle {
    pub fn new() -> UtilBundle {
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
        // !ERROR: This returns null if program ran without sufficient perms (admin)
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
