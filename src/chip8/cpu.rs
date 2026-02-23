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

    // FX0A key-wait state: Some(x) means waiting for a key, storing into VX
    waiting_for_key: Option<u8>,
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
// Chip8 Implementation
// ===============================================================

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {

    // Initialize a new Chip8 instance with default state
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
            waiting_for_key: None,
        };

        for (index, &byte) in FONT_SET.iter().enumerate() {
            chip8.memory[FONT_START as usize + index] = byte;
        }

        chip8
    }

    // Load a ROM into memory starting at 0x200
    pub fn load_rom(&mut self, data: &[u8]) {
        let start: usize = PROGRAM_START as usize;
        let end: usize = start + data.len();

        if end > MEMORY_SIZE {
            panic!("ROM too large to fit in memory");
        }

        self.memory[start..end].copy_from_slice(data);
    }

    // Decrement timers (should be called at 60Hz externally)
    pub fn tick_timers(&mut self) {
    if self.delay_timer > 0 {
        self.delay_timer -= 1;
    }

    if self.sound_timer > 0 {
        self.sound_timer -= 1;
    }
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
        // FX0A — block until any key is pressed, then store it in VX
        if let Some(vx) = self.waiting_for_key {
            for (key_index, &pressed) in self.keys.iter().enumerate() {
                if pressed {
                    self.v[vx as usize] = key_index as u8;
                    self.waiting_for_key = None;
                }
            }
            return;
        }

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
                    // 0x0NNN (call RCA 1802 program) — not used by modern ROMs, intentionally ignored
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
                if self.v[decoded.x as usize] == decoded.nn {
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

            // Arithmetic and bitwise operations between VX and VY
            0x8 => {
                match decoded.n {

                    // VX is set to the value of VY
                    0x0 => {
                        self.v[decoded.x as usize] =
                            self.v[decoded.y as usize];
                    }

                    // VX is set to VX OR VY
                    0x1 => {
                        self.v[decoded.x as usize] |=
                            self.v[decoded.y as usize];
                    }

                    // VX is set to VX AND VY
                    0x2 => {
                        
                        self.v[decoded.x as usize] &=
                            self.v[decoded.y as usize];
                    }

                    // VX is set to VX XOR VY
                    0x3 => {
                        
                        self.v[decoded.x as usize] ^=
                            self.v[decoded.y as usize];
                    }

                    // VX += VY, VF = carry
                    0x4 => {
                        let (result, carry) =
                            self.v[decoded.x as usize]
                                .overflowing_add(self.v[decoded.y as usize]);

                        self.v[decoded.x as usize] = result;
                        self.v[0xF] = if carry { 1 } else { 0 };
                    }

                    // VX -= VY, VF = NOT borrow
                    0x5 => {
                        let (result, borrow) =
                            self.v[decoded.x as usize]
                                .overflowing_sub(self.v[decoded.y as usize]);

                        self.v[decoded.x as usize] = result;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                    }

                    // VX >>= 1, VF = least significant bit before shift
                    0x6 => {
                        let lsb = self.v[decoded.x as usize] & 0x1;
                        self.v[0xF] = lsb;
                        self.v[decoded.x as usize] >>= 1;
                    }

                    // VX = VY - VX, VF = NOT borrow
                    0x7 => {
                        let (result, borrow) =
                            self.v[decoded.y as usize]
                                .overflowing_sub(self.v[decoded.x as usize]);

                        self.v[decoded.x as usize] = result;
                        self.v[0xF] = if borrow { 0 } else { 1 };
                    }

                    // VX <<= 1, VF = most significant bit before shift
                    0xE => {
                        let msb: u8 = (self.v[decoded.x as usize] & 0x80) >> 7;
                        self.v[0xF] = msb;
                        self.v[decoded.x as usize] <<= 1;
                    }

                    _ => {
                        eprintln!("Invalid 8XYN opcode: {:#06X}", opcode);
                    }
                }
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

            // Jump to address NNN + V0
            0xB => {
                self.pc = decoded.nnn + self.v[0] as u16;
            }

            // VX = random byte AND NN
            0xC => {
                let random: u8 = rand::random();
                self.v[decoded.x as usize] = random & decoded.nn;
            }

            // Display/draw sprite at (VX, VY) with height N
            0xD => {
                let x_pos: usize = self.v[decoded.x as usize] as usize;
                let y_pos: usize = self.v[decoded.y as usize] as usize;
                let height: usize = decoded.n as usize;

                self.v[0xF] = 0;

                for row in 0..height {
                    let sprite_byte: u8 =
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

            // Key input instructions
            0xE => {
                let key: usize = self.v[decoded.x as usize] as usize;

                match decoded.nn {
                    0x9E => {
                        if key < NUM_KEYS && self.keys[key] {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        if key < NUM_KEYS && !self.keys[key] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        eprintln!("Invalid EX opcode: {:#06X}", opcode);
                    }
                }
            }

            // Miscellaneous instructions
            0xF => {
                match decoded.nn {

                    // FX07 — VX = delay_timer
                    0x07 => {
                        self.v[decoded.x as usize] = self.delay_timer;
                    }

                    // FX0A — Wait for key press, store key index in VX (blocking)
                    0x0A => {
                        if let Some(vx) = self.waiting_for_key {
                            for (key_index, &pressed) in self.keys.iter().enumerate() {
                                if pressed {
                                    self.v[vx as usize] = key_index as u8;
                                    self.waiting_for_key = None;
                                    break;
                                }
                            }
                            return;
                        }
                    }

                    // FX15 — delay_timer = VX
                    0x15 => {
                        self.delay_timer = self.v[decoded.x as usize];
                    }

                    // FX18 — sound_timer = VX
                    0x18 => {
                        self.sound_timer = self.v[decoded.x as usize];
                    }

                    // FX1E — I += VX
                    0x1E => {
                        self.i = self.i.wrapping_add(self.v[decoded.x as usize] as u16);
                    }

                    // FX29 — Set I to font character location
                    0x29 => {
                        let digit: u16 = (self.v[decoded.x as usize] & 0x0F) as u16;
                        self.i = FONT_START + digit * 5;
                    }

                    // FX33 — Store BCD representation of VX at I, I+1, I+2
                    0x33 => {
                        let value: u8 = self.v[decoded.x as usize];

                        self.memory[self.i as usize]     = value / 100;
                        self.memory[self.i as usize + 1] = (value % 100) / 10;
                        self.memory[self.i as usize + 2] = value % 10;
                    }

                    // FX55 — Store V0..VX in memory starting at I
                    0x55 => {
                        for idx in 0..=decoded.x as usize {
                            self.memory[self.i as usize + idx] = self.v[idx];
                        }
                    }

                    // FX65 — Load V0..VX from memory starting at I
                    0x65 => {
                        for idx in 0..=decoded.x as usize {
                            self.v[idx] = self.memory[self.i as usize + idx];
                        }
                    }

                    _ => {
                        eprintln!("Invalid FX opcode: {:#06X}", opcode);
                    }
                }
            }

            _ => {
                eprintln!("Unknown opcode: {:#06X}", opcode);
            }
        }
    }
}