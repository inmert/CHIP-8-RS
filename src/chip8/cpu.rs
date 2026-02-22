// ───────────────────────────────────────────────────────────────
// CHIP-8 Emulator — CPU Core
// Represents the complete state of the CHIP-8 virtual machine.
// ───────────────────────────────────────────────────────────────

use crate::chip8::constants::*;

// ===============================================================
// Full CHIP-8 machine state
// ===============================================================

pub struct Chip8 {
    // 4KB RAM (0x000–0xFFF)
    pub memory: [u8; MEMORY_SIZE],

    // General-purpose registers V0–VF (VF used as flag)
    pub v: [u8; NUM_REGISTERS],

    // Index register
    pub i: u16,

    // Program counter
    pub pc: u16,

    // Subroutine call stack
    pub stack: [u16; STACK_SIZE],

    // Stack pointer
    pub sp: u8,

    // 64x32 monochrome display buffer
    pub display: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],

    // 16-key hexadecimal keypad state
    pub keys: [bool; NUM_KEYS],

    // Timers (decrement at 60Hz externally)
    pub delay_timer: u8,
    pub sound_timer: u8,
}

// ===============================================================
// Decoded Opcode Representation
// ===============================================================

pub struct DecodedFields {
    pub first_nibble: u8,
    pub x: u8,
    pub y: u8,
    pub n: u8,
    pub nn: u8,
    pub nnn: u16,
}

impl DecodedFields {
    pub fn new(opcode: u16) -> Self {
        Self {
            first_nibble: ((opcode & 0xF000) >> 12) as u8,
            x:            ((opcode & 0x0F00) >> 8)  as u8,
            y:            ((opcode & 0x00F0) >> 4)  as u8,
            n:            (opcode & 0x000F)         as u8,
            nn:           (opcode & 0x00FF)         as u8,
            nnn:           opcode & 0x0FFF,
        }
    }
}

// ===============================================================
// Chip8 Construction
// ===============================================================

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {

    pub fn new() -> Self {
        let mut chip8: Chip8 = Self {
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

        for (index, &byte) in FONT_SET.iter().enumerate() {
            chip8.memory[FONT_START as usize + index] = byte;
        }

        chip8
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        let start: usize = PROGRAM_START as usize;
        let end: usize = start + data.len();

        if end > MEMORY_SIZE {
            panic!("ROM too large to fit in memory");
        }

        self.memory[start..end].copy_from_slice(data);
    }

    // ===========================================================
    // Fetch Stage
    // ===========================================================

    pub fn fetch(&mut self) -> u16 {
        let high_byte: u16 = self.memory[self.pc as usize] as u16;
        let low_byte: u16  = self.memory[(self.pc + 1) as usize] as u16;

        let opcode: u16 = (high_byte << 8) | low_byte;

        self.pc += 2;

        opcode
    }

    // ===========================================================
    // Execution Cycle
    // ===========================================================

    pub fn cycle(&mut self) {
        let opcode: u16 = self.fetch();
        let decoded: DecodedFields = DecodedFields::new(opcode);

        match decoded.first_nibble {
            
            // System instructions (0x0NNN) and special cases
            0x0 => {
                match opcode {
                    // Clear display
                    0x00E0 => {
                        self.display = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
                    }
                    // Return from subroutine
                    0x00EE => {
                        if self.sp == 0 {
                            eprintln!("Stack underflow on 0x00EE");
                            return;
                        }
                        self.sp -= 1;
                        self.pc = self.stack[self.sp as usize];
                    }
                    _ => {
                        eprintln!("Unknown opcode: {:#06X}", opcode);
                    }
                }
            }

            // Jump to address NNN
            0x1 => {
                self.pc = decoded.nnn;
            }

            // Call subroutine at NNN
            0x2 => {
                if self.sp as usize >= STACK_SIZE {
                    eprintln!("Stack overflow on 0x2NNN");
                    return;
                }

                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;

                self.pc = decoded.nnn;
            }

            // Skip next instruction if VX == NN
            0x3 => {
                if self.v[decoded.x as usize] == decoded.n {
                    self.pc += 2;
                }
            }

            // Skip next instruction if VX != NN
            0x4 => {
                if self.v[decoded.x as usize] != decoded.nn {
                    self.pc += 2;
                }
            }

            // Skip next instruction if VX == VY (only if N == 0)
            0x5 => {
                if decoded.n == 0 {
                    if self.v[decoded.x as usize] == self.v[decoded.y as usize] {
                        self.pc += 2;
                    }
                } else {
                    eprintln!("Invalid opcode: {:#06X}", opcode);
                }
            }

            // Set VX to NN
            0x6 => {

                self.v[decoded.x as usize] = decoded.nn;
            }

            // VX += NN (wrapping)
            0x7 => {
                self.v[decoded.x as usize] =
                    self.v[decoded.x as usize].wrapping_add(decoded.nn);
            }

            // Skip next instruction if VX != VY (only if N == 0)
            0x9 => {
                if decoded.n == 0 {
                    if self.v[decoded.x as usize] != self.v[decoded.y as usize] {
                        self.pc += 2;
                    }
                } else {
                    eprintln!("Invalid opcode: {:#06X}", opcode);
                }
            }

            // Set I to NNN
            0xA => {
                self.i = decoded.nnn;
            }

            // Display/draw sprite at (VX, VY) with height N
            0xD => {
                let x_pos: usize = self.v[decoded.x as usize] as usize;
                let y_pos: usize = self.v[decoded.y as usize] as usize;
                let height: usize = decoded.n as usize;

                self.v[0xF] = 0;

                for row in 0..height {
                    let sprite_byte =
                        self.memory[(self.i + row as u16) as usize];

                    for bit in 0..8 {
                        let sprite_pixel: bool =
                            (sprite_byte & (0x80 >> bit)) != 0;

                        if sprite_pixel {
                            let x: usize = (x_pos + bit) % DISPLAY_WIDTH;
                            let y: usize = (y_pos + row) % DISPLAY_HEIGHT;

                            if self.display[y][x] {
                                self.v[0xF] = 1;
                            }

                            self.display[y][x] ^= true;
                        }
                    }
                }
            }

            _ => {
                eprintln!("Unknown opcode: {:#06X}", opcode);
            }
        }
    }
}