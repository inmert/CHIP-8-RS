// ───────────────────────────────────────────────────────────────
//  CHIP-8 Emulator — Rust
//  The core CPU structure representing the state of the emulator.
// ───────────────────────────────────────────────────────────────

use crate ::chip8::constant::*;

pub struct Chip8 {
    // =========================
    // Core Memory
    // =========================
    pub memory: [u8; MEMORY_SIZE],      // 4KB of memory

    // =========================
    // Registers
    // =========================
    pub v: [u8; NUM_REGISTERS],         // V0–VF
    pub i: u16,                         // Index register
    pub pc: u16,                        // Program counter

    // =========================
    // Stack
    // =========================
    pub stack: [u16; STACK_SIZE],       // Call stack for subroutines
    pub sp: u8,                         // Stack pointer

    // =========================
    // Display
    // =========================
    pub display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],   // 64x32 monochrome display

    // =========================
    // Keypad
    // =========================
    pub keys: [bool; NUM_KEYS],         // 16 keys (0-9, A-F)

    // =========================
    // Timers
    // =========================
    pub delay_timer: u8,                // Delay timer
    pub sound_timer: u8,                // Sound timer
}