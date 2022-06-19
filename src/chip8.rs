use crate::Display;
use std::error::Error;


pub struct Chip8 {
    pub memory: [u8; 4096],            // 4KB of memory for the program (writable).
    pub display_buf: [[bool; 32]; 64], // 2D pixel array
    pub pc: u16,                       // Program Counter: Points at current instruction in memory
    pub i: u16,                        // Index register: Points at locations in memory
    pub stack: Vec<u16>,               // Stack used to call functions/subroutines and return from them
    pub delay_timer: u32,              // decreases at 60Hz
    pub sound_timer: u32,              // beeps when value is 0, decreases at 60Hz
    pub V: [u8; 16],                   // 16 8-bit registers, from V0 to VF. VF is often used as a flag register.
    pub font_location: u16,            // Starting point of the stored fonts in memory
    pub draw_flag: bool,                // true if current opcode has changed the display buffer
    pub timers_dec_flag: bool
}


impl Chip8 {

    pub fn init() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; 4096],
            display_buf: [[false; 32]; 64],
            pc: 0x200,
            i: 0,
            stack: Vec::with_capacity(32),
            delay_timer: 0,
            sound_timer: 0,
            V: [0; 16],
            font_location: 0x200,
            draw_flag: false,
            timers_dec_flag: false,
        };

        chip8.init_font();
        chip8
    }

    fn init_font(self: &mut Self) {
        let font = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
    
        self.memory[0x50..=0x09F].clone_from_slice(&font[..]);
    }

    pub fn load_data(self: &mut Self, path: &'static str) -> Result<(), Box<dyn Error>> {
        let mem = std::fs::read(path)?;
        let mem_len = mem.len();
        self.memory[0x200..0x200+mem_len].clone_from_slice(&mem[..]);

        Ok(())
    }

    pub fn beep_sound(&self) {
        // TODO
    }
}
