// ───────────────────────────────────────────────────────────────
// CHIP-8 Emulator — CPU Core
// Represents the complete state of the CHIP-8 virtual machine.
// ───────────────────────────────────────────────────────────────

use crate::chip8::constants::*;

/// Full CHIP-8 machine state.
pub struct Chip8 {
    // 4KB RAM (0x000–0xFFF)
    pub memory: [u8; MEMORY_SIZE],

    // General-purpose registers V0–VF (VF used as flag)
    pub v: [u8; NUM_REGISTERS],

    // Index register (12-bit address stored in u16)
    pub i: u16,

    // Program counter (points to current instruction)
    pub pc: u16,

    // Subroutine call stack (stores return addresses)
    pub stack: [u16; STACK_SIZE],

    // Stack pointer (index into stack array)
    pub sp: u8,

    // 64x32 monochrome display buffer
    pub display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],

    // 16-key hexadecimal keypad state
    pub keys: [bool; NUM_KEYS],

    // Timers (decrement at 60Hz externally)
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Chip8 {

    /// Initializes a new CHIP-8 instance.
    /// - Clears memory and registers
    /// - Sets PC to 0x200
    /// - Loads built-in font set into reserved memory
    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: PROGRAM_START,
            stack: [0; STACK_SIZE],
            sp: 0,
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };

        // Load font set into memory at FONT_START
        for (index, byte) in FONT_SET.iter().enumerate() {
            chip8.memory[(FONT_START as usize) + index] = *byte;
        }

        chip8
    }

    /// Fetches the next 16-bit opcode from memory.
    /// Increments the program counter by 2.
    pub fn fetch(&mut self) -> u16 {
        let high_byte = self.memory[self.pc as usize] as u16;
        let low_byte  = self.memory[(self.pc + 1) as usize] as u16;

        let opcode = (high_byte << 8) | low_byte;

        self.pc += 2;

        opcode
    }

    /// Extracts commonly used fields from a 16-bit opcode.
    ///
    /// Returns:
    /// (first_nibble, x, y, n, nnn, nn)
    fn decode_fields(opcode: u16) -> (u8, u8, u8, u8, u16, u8) {
        let first_nibble = ((opcode & 0xF000) >> 12) as u8;
        let x            = ((opcode & 0x0F00) >> 8) as u8;
        let y            = ((opcode & 0x00F0) >> 4) as u8;
        let n            = (opcode & 0x000F) as u8;
        let nn           = (opcode & 0x00FF) as u8;
        let nnn          = opcode & 0x0FFF;

        (first_nibble, x, y, n, nnn, nn)
    }

    /// Executes one fetch-decode-execute cycle.
    /// Timing control must be handled externally.
    pub fn cycle(&mut self) {
        // Fetch
        let opcode = self.fetch();

        // Decode
        let (first, x, y, n, nnn, nn) = Self::decode_fields(opcode);

        // Execute
        match first {

            0x0 => {
                match opcode {
                    0x00E0 => {
                        // Clear display
                    }
                    0x00EE => {
                        // Return from subroutine
                    }
                    _ => {
                        // 0NNN - Ignored (legacy RCA 1802 call)
                    }
                }
            }

            0x1 => {
                // 1NNN - Jump to address NNN
            }

            0x6 => {
                // 6XNN - Set VX = NN
            }

            0x7 => {
                // 7XNN - VX += NN
            }

            0xA => {
                // ANNN - Set I = NNN
            }

            0xD => {
                // DXYN - Draw sprite
            }

            _ => {
                // Unknown or unimplemented opcode
            }
        }
    }
}