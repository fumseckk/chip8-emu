mod display;
mod opcodes;
mod chip8;
use display::*;
use chip8::*;
use std::time::Duration;
use std::thread::sleep;

const OP_PER_SECOND: u64 = 700;

fn main() {
    // Chip8
    let mut chip8 = Chip8::init();
    chip8.load_data("/home/phoenix/Downloads/particle_demo.ch8").unwrap();
    let mut dec = (OP_PER_SECOND / 60) as u32; 
    
    // Graphics
    let mut display = Display::new();
    display.update(&chip8.display_buf);
    
    // Main loop
    while !display.should_quit() {

        // Process opcode
        let opcode = chip8.fetch_opcode().unwrap(); // fetch_opcode().unwrap() panics if invalid operation is read in memory (i.e if None is returned)
        chip8.execute_opcode(opcode, &display).unwrap_or_else(|err| {
            eprintln!("Error executing opcode: {}", err);
            std::process::exit(1);
        });

        // Draw if necessary
        if chip8.draw_flag {
            display.update(&chip8.display_buf);
            chip8.draw_flag = false;
        }

        // Timers
        dec -= 1;
        if dec == 0 { // should decrease the timers this loop
            dec = (OP_PER_SECOND / 60) as u32;
            if chip8.delay_timer > 0 {
                chip8.delay_timer -= 1;
            }
            if chip8.sound_timer > 0 {
                chip8.sound_timer -= 1;
            }
            else if chip8.sound_timer == 0 {
                chip8.beep_sound();
            }
        }

        // println!("{:?}", opcode);
        // End loop
        sleep(Duration::from_micros((1/OP_PER_SECOND) * 1e6 as u64));
        // sleep(Duration::from_millis(500));
    }

}