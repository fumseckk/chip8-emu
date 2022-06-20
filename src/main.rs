mod display;
mod opcodes;
mod chip8;
use display::*;
use chip8::*;
use std::time::Duration;
use std::thread::sleep;
use std::env;

const OP_PER_SECOND: u64 = 700;
const OP_TRIGGER_VAL: u64 = 1_0000 / OP_PER_SECOND;
const TIMER_TRIGGER_VAL: u64 = 1_0000 / 60;


fn main() {

    // Game path: CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("Error: no game path specified");
        std::process::exit(1);
    }

    // Chip8
    let mut chip8 = Chip8::init();
    chip8.load_data(&args[1]).unwrap();
    
    // Graphics
    let mut display = Display::new();
    display.update(&chip8.display_buf);

    let mut op_trigger = OP_TRIGGER_VAL;
    let mut timer_trigger = TIMER_TRIGGER_VAL;
    
    // Main loop
    while !display.should_quit() {

        if op_trigger == 0 {
            op_trigger = OP_TRIGGER_VAL;

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
        }


        // Timers
        if timer_trigger == 0 { // should decrease the timers this loop
            timer_trigger = TIMER_TRIGGER_VAL;
            
            if chip8.delay_timer > 0 {
                chip8.delay_timer -= 1;
            }
            if chip8.sound_timer > 0 {
                chip8.sound_timer -= 1;
                if chip8.sound_timer == 0 {
                    chip8.beep_sound();
                }
            }
        }

        // println!("{:?}", opcode);
        // End loop
        timer_trigger -= 1;
        op_trigger -= 1;
        sleep(Duration::from_micros(1));
        // sleep(Duration::from_millis(500));
    }

}