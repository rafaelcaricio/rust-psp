/// Executes a `FnOnce` `iterations` times and returns average duration per iteration
pub fn benchmark<F: FnOnce()+Copy>(f: F, iterations: i32) -> core::time::Duration {
    let mut loop_start: u64 = 0;
    let mut loop_end: u64 = 0;
    let avg_micros: u64;
    unsafe {
        crate::sys::rtc::sce_rtc_get_current_tick(&mut loop_start as *mut u64);
        
        for _ in 0..iterations {
            f();
        }
        crate::sys::rtc::sce_rtc_get_current_tick(&mut loop_end as *mut u64);
        let avg_iter_ticks = (loop_end - loop_start) / iterations as u64;
        let ticks_per_sec = crate::sys::rtc::sce_rtc_get_tick_resolution();
        avg_micros = ((avg_iter_ticks as f64 / ticks_per_sec as f64) * 1_000_000.0) as u64;
    }
    core::time::Duration::from_micros(avg_micros)
}

