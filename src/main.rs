use procfs::{Meminfo, MemoryPressure, ProcResult};
use std::time::Instant;
use std::{fs, thread::sleep, time::Duration};
const SYSRQ_TRIGGER_FILE: &str = "/proc/sysrq-trigger";
const CHECK_INTERVAL: Duration = Duration::from_secs(1);
const RECOVERY_INTERVAL: Duration = Duration::from_secs(10);
const MEM_THRESHOLD: f32 = 15.0;
const SYSRQ_WHEN_MEMORY_THRESHOLD: bool = false;
fn get_memory_used(meminfo: &Meminfo) -> Option<u64> {
    Some(
        meminfo.mem_total
            - meminfo.mem_available?
            - meminfo.mem_free
            - meminfo.buffers
            - meminfo.shmem?,
    )
}
fn main() -> ProcResult<()> {
    let _ = simple_logger::SimpleLogger::new().init();
    log::info!("tiny-oom started.");
    log::info!("build config: MEM_THRESHOLD={MEM_THRESHOLD}, SYSRQ_WHEN_MEMORY_THRESHOLD={SYSRQ_WHEN_MEMORY_THRESHOLD}");
    loop {
        let memory_pressure_percent = MemoryPressure::new()?.full.avg10;
        #[cfg(debug_assertions)]
        log::debug!("memory pressure: {memory_pressure_percent}%");
        if memory_pressure_percent > MEM_THRESHOLD {
            if SYSRQ_WHEN_MEMORY_THRESHOLD {
                let now = Instant::now();
                fs::write(SYSRQ_TRIGGER_FILE, "f")?;
                log::info!("sysrq triggered in {:?}", now.elapsed());
            } else {
                log::warn!("memory pressure is too high, but not triggering sysrq; checking memory usage before");
                let meminfo = Meminfo::new()?;
                let used = get_memory_used(&meminfo).unwrap_or(0);
                let used_percent = (100 * used) / meminfo.mem_total;

                if used_percent > 90 {
                    log::warn!("memory usage is too high, triggering sysrq");
                    fs::write(SYSRQ_TRIGGER_FILE, "f")?;
                }
                log::info!("memory usage: {used_percent}%");
            }
            sleep(RECOVERY_INTERVAL);
        }
        sleep(CHECK_INTERVAL);
    }
}
