use crate::Chip8;
use termkan::input::{Input, InputEvent, KeyEvent};


pub struct InputHandler {
    pub last_input: Option<InputEvent>
}

impl InputHandler {
    pub fn new() -> InputHandler {
        InputHandler { last_input: None }
    }

    pub fn update(&mut self) {
        let input = Input::get();
        self.last_input = input.get_event();
    }

    // Detects if Ctrl+C is pressed
    pub fn should_quit(&self) -> bool {
        eprintln!("{:?}", self.last_input);
        Some(InputEvent::Key(KeyEvent::Ctrl('c'))) == self.last_input
    }

    pub fn is_key_down(&self, key: KeyEvent) -> bool {
        Some(InputEvent::Key(key)) == self.last_input
    }

    pub fn is_key_up(&self, key: KeyEvent) -> bool {
        Some(InputEvent::Key(key)) != self.last_input
    }

    pub fn any_key_pressed(&self) -> Option<u8> {
        if let Some(InputEvent::Key(key)) = self.last_input {
        eprintln!("{:?}", self.last_input);
            return Chip8::code_from_kkey(key);
        }
        return None;
    }
}