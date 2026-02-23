mod chip8;

use std::time::{Duration, Instant};
use std::thread;

use chip8::cpu::Chip8;

const CPU_HZ: u64 = 700;
const TIMER_HZ: u64 = 60;

fn main() {
    let mut chip8: Chip8 = Chip8::new();

    // TODO: Load ROM here
    // chip8.load_rom(&rom_bytes);

    let cpu_interval: Duration = Duration::from_secs_f64(1.0 / CPU_HZ as f64);
    let timer_interval: Duration = Duration::from_secs_f64(1.0 / TIMER_HZ as f64);

    let mut last_cpu_tick: Instant = Instant::now();
    let mut last_timer_tick: Instant = Instant::now();

    loop {
        let now: Instant = Instant::now();

        // CPU execution
        if now.duration_since(last_cpu_tick) >= cpu_interval {
            chip8.cycle();
            last_cpu_tick = now;
        }

        // Timer ticking
        if now.duration_since(last_timer_tick) >= timer_interval {
            chip8.tick_timers();
            last_timer_tick = now;
        }

        // Prevent 100% CPU usage
        thread::sleep(Duration::from_micros(500));
    }
}