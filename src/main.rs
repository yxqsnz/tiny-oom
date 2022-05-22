use anyhow::{anyhow, Result};
use itertools::Itertools;
use log::LevelFilter;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    thread::sleep,
    time::Duration,
};
const SYSRQ_TRIGGER_FILE: &str = "/proc/sysrq-trigger";
const PSI_MEMORY_FILE: &str = "/proc/pressure/memory";
const CHECK_INTERVAL: Duration = Duration::from_secs(1);
const RECOVERY_INTERVAL: Duration = Duration::from_secs(10);
const MEM_THRESHOLD: f32 = 15.0;
fn get_avg10_from_string(s: String) -> Option<f32> {
    s.split_ascii_whitespace()
        .skip(1)
        .map(|x| x.split_once("avg10="))
        .filter_map(|x| Some(x?.1.to_string()))
        .find_map(|x| x.parse::<f32>().ok())
}
fn main() -> Result<()> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    log::debug!("sysrq trigger file: {SYSRQ_TRIGGER_FILE}");
    log::debug!("psi memory file: {PSI_MEMORY_FILE}");
    log::info!("tiny-oom started!");
    loop {
        let psi_reader = BufReader::new(File::open(PSI_MEMORY_FILE)?);
        let avg10 = psi_reader
            .lines()
            .skip(1)
            .map_ok(get_avg10_from_string)
            .next()
            .ok_or(anyhow!("no psi data"))??;
        if avg10 > Some(MEM_THRESHOLD) {
            log::info!("triggering sysrq");
            fs::write(SYSRQ_TRIGGER_FILE, "f")?;
            log::debug!("recovering");
            sleep(RECOVERY_INTERVAL);
        } else if avg10 > Some(MEM_THRESHOLD / 2.0) {
            log::warn!("memory pressure is high");
            sleep(CHECK_INTERVAL / 2);
        } else {
            log::debug!("memory pressure is low, avg = {:?}", avg10);
            sleep(CHECK_INTERVAL);
        }
    }
}
