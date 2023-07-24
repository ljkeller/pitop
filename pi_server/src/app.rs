use util_bundle::UtilBundle;

const MAX_UTIL_WINDOW_N: usize = 60;

pub struct App {
    pub cpu_util: Vec<Vec<(f64, f64)>>,
    pub network_tx: Vec<(f64, f64)>,
    pub network_rx: Vec<(f64, f64)>,
    pub gpu_util: Vec<(f64, f64)>,
    pub mem_util: Vec<(f64, f64)>,
}

impl App {
    pub fn new() -> App {
        App {
            cpu_util: vec![],
            network_tx: vec![],
            network_rx: vec![],
            gpu_util: vec![],
            mem_util: vec![],
        }
    }

    // TODO: Optimize if necessary
    pub fn on_tick(&mut self, datapoint: UtilBundle) {

        if self.cpu_util.len() > MAX_UTIL_WINDOW_N {
            self.cpu_util.remove(0);
        }
        if self.network_tx.len() > MAX_UTIL_WINDOW_N {
            self.network_tx.remove(0);
        }
        if self.network_rx.len() > MAX_UTIL_WINDOW_N {
            self.network_rx.remove(0);
        }
        if self.gpu_util.len() > MAX_UTIL_WINDOW_N {
            self.gpu_util.remove(0);
        }
        if self.mem_util.len() > MAX_UTIL_WINDOW_N {
            self.mem_util.remove(0);
        }

        while self.cpu_util.len() < datapoint.cpu_usage.len() {
            self.cpu_util.push(vec![]);
        }

        datapoint.cpu_usage.iter().enumerate().for_each(|(idx, f)| {
            self.cpu_util[idx].push((0 as f64, *f as f64))
        });

        self.network_tx.push((0 as f64, datapoint.data_tx as f64));
        self.network_rx.push((0 as f64, datapoint.data_rx as f64));
        self.gpu_util.push((0 as f64, datapoint.gpu_usage as f64));
        // TODO: never divide by 0 (wont be an issue once sharing info between threads)
        if datapoint.mem_total > 0 {
            self.mem_util.push((
                0 as f64,
                datapoint.mem_used as f64 / datapoint.mem_total as f64,
            ));
        } else {
            self.mem_util.push((0.0, 0.0));
        }
        
        // There are a couple obvious ways to organize cpu_util data:
        // 1. [[core1], [core2], [core3], ...]
        // 2. [[datapoint1], [datapoint2], [datapoint3], ...]
        // Organizing as 1. allows us to easily plot each core as its own dataset (and follows how other 
        // utils are stored)
        self.cpu_util
            .iter_mut()
            .for_each(|v| v.iter_mut().rev().enumerate().for_each(|(idx, (x_t, _y_t))| *x_t = idx as f64));

        self.network_tx
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.network_rx
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.gpu_util
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
        self.mem_util
            .iter_mut()
            .rev()
            .enumerate()
            .for_each(|(i, (t, _y))| *t = i as f64);
    }
}