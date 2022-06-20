use crate::{Chip8, InputHandler};
use termkan::{input::KeyEvent};
use rand::Rng;


type N = u8;
type NN = u8;
type NNN = u16;
type Vx = usize;
type Vy = usize;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode {
    ClearScreen(),
    Return(),
    Jump(NNN),
    CallSubroutine(NNN),
    CondEq(Vx, NN),
    CondNEq(Vx, NN),
    CondEqReg(Vx, Vy),
    SetReg(Vx, NN),
    AddToReg(Vx, NN), // no change to carry
    AssignRegToReg(Vx, Vy),
    BitwiseOr(Vx, Vy),
    BitwiseAnd(Vx, Vy),
    BitwiseXor(Vx, Vy),
    AddRegToReg(Vx, Vy), // with carry
    SubRegToReg(Vx, Vy), // with carry
    StoreLSBWithShift(Vx),
    SubRegFromReg(Vx, Vy), // with carry
    StoreMSBWithShift(Vx),
    CondNEqReg(Vx, Vy),
    SetI(NNN),
    JumpToV0Plus(NNN),
    RegRandBitwiseAnd(Vx, NN), // rand: from 0 to 255
    DrawSprite(Vx, Vy, N), // with carry
    IsKeyPressed(Vx),
    IsKeyNPressed(Vx),
    SetRegToTimer(Vx),
    AwaitKey(Vx), // blocking op
    SetDelayTimer(Vx),
    SetSoundTimer(Vx),
    AddRegToI(Vx),
    SetIToSprite(Vx),
    ToDecimal(Vx),
    DumpRegs(Vx),
    LoadRegs(Vx),
}


impl Chip8 {
    
    fn nth_nibble(n: u8, opcode: u16) -> u16 {
        match n {
            0 => opcode >> 12,
            1 => (opcode & 0x_0F00) >> 8,
            2 => (opcode & 0x_00F0) >> 4,
            3 => (opcode & 0x_000F),
            _ => 0x0 // should not happen
        }
    }


    fn range_nibble(start: u8, end: u8, opcode: u16) -> u16{
        let mut mask = 0;
        for i in start..end {
            mask |= 0xF << 4*(4-i-1);
        }
       (opcode & mask) >> (4-end)*4
    }


    pub fn kkey_from_code(code: u8) -> Option<KeyEvent> {
        match code {
            0x1 => Some(KeyEvent::Char('&')),
            0x2 => Some(KeyEvent::Char('é')),
            0x3 => Some(KeyEvent::Char('"')),
            0xC => Some(KeyEvent::Char('\'')),
            0x4 => Some(KeyEvent::Char('a')),
            0x5 => Some(KeyEvent::Char('z')),
            0x6 => Some(KeyEvent::Char('e')),
            0xD => Some(KeyEvent::Char('r')),
            0x7 => Some(KeyEvent::Char('q')),
            0x8 => Some(KeyEvent::Char('s')),
            0x9 => Some(KeyEvent::Char('d')),
            0xE => Some(KeyEvent::Char('f')),
            0xA => Some(KeyEvent::Char('w')),
            0x0 => Some(KeyEvent::Char('x')),
            0xB => Some(KeyEvent::Char('c')),
            0xF => Some(KeyEvent::Char('v')),
            _ => None
        }
    }


    pub fn code_from_kkey(key: KeyEvent) -> Option<u8> {
        match key {
            KeyEvent::Char('&') =>  Some(0x1),
            KeyEvent::Char('é') =>  Some(0x2),
            KeyEvent::Char('"') =>  Some(0x3),
            KeyEvent::Char('\'') => Some(0xC),
            KeyEvent::Char('a') =>  Some(0x4),
            KeyEvent::Char('z') =>  Some(0x5),
            KeyEvent::Char('e') =>  Some(0x6),
            KeyEvent::Char('r') =>  Some(0xD),
            KeyEvent::Char('q') =>  Some(0x7),
            KeyEvent::Char('s') =>  Some(0x8),
            KeyEvent::Char('d') =>  Some(0x9),
            KeyEvent::Char('f') =>  Some(0xE),
            KeyEvent::Char('w') =>  Some(0xA),
            KeyEvent::Char('x') =>  Some(0x0),
            KeyEvent::Char('c') =>  Some(0xB),
            KeyEvent::Char('v') =>  Some(0xF),
            _ => None
        }

    }


    pub fn fetch_opcode(&mut self) -> Option<OpCode> {
        let part1 = self.memory[self.pc as usize];
        let part2 = self.memory[(self.pc + 1 ) as usize];
        let opcode: u16 = ((part1 as u16) << 8) | (part2 as u16);

        self.pc += 2;

        let x = Self::nth_nibble(1, opcode) as usize;
        let y = Self::nth_nibble(2, opcode) as usize;
        let n = Self::nth_nibble(3, opcode) as u8;
        let nn = Self::range_nibble(2, 4, opcode) as u8;
        let nnn = Self::range_nibble(1, 4, opcode);

        match Self::nth_nibble(0, opcode) {
            0x0 => {
                match opcode {
                    0x00E0 => Some(OpCode::ClearScreen()), // 00E0
                    0x00EE => Some(OpCode::Return()), // 00EE
                    _      => None
                }
            }
            0x1 => Some(OpCode::Jump(nnn)), // 1nnn
            0x2 => Some(OpCode::CallSubroutine(nnn)), // 2nnn
            0x3 => Some(OpCode::CondEq(x, nn)), // 3XNN
            0x4 => Some(OpCode::CondNEq(x, nn)), // 4XNN
            0x5 => {
                match n {
                    0x0 => Some(OpCode::CondEqReg(x, y)), // 5XY0
                    _   => None
                }
            }
            0x6 => Some(OpCode::SetReg(x, nn)), // 6XNN
            0x7 => Some(OpCode::AddToReg(x, nn)), // 7XNN
            0x8 => {
                match n {
                    0 => Some(OpCode::AssignRegToReg(x, y)), // 8XY0
                    1 => Some(OpCode::BitwiseOr(x, y)), // 8XY1
                    2 => Some(OpCode::BitwiseAnd(x, y)), // 8XY2
                    3 => Some(OpCode::BitwiseXor(x, y)), // 8XY3
                    4 => Some(OpCode::AddRegToReg(x, y)), // 8XY4
                    5 => Some(OpCode::SubRegToReg(x, y)), // 8XY5
                    6 => Some(OpCode::StoreLSBWithShift(x)), // 8XY6
                    7 => Some(OpCode::SubRegFromReg(x, y)), // 8XY7
                    0xE => Some(OpCode::StoreMSBWithShift(x)), // 8XYE
                    _ => None
                }
            }
            0x9 => {
                match n {
                    0x0 => Some(OpCode::CondNEqReg(x, y)), // 9XY0
                    _   => None
                }
            }
            0xA => Some(OpCode::SetI(nnn)), // Annn
            0xB => Some(OpCode::JumpToV0Plus(nnn)), // Bnnn
            0xC => Some(OpCode::RegRandBitwiseAnd(x, nn)), // CXNN
            0xD => Some(OpCode::DrawSprite(x, y, n)), // DXYN
            0xE => {
                match nn {
                    0x9E => Some(OpCode::IsKeyPressed(x)), // EX9E
                    0xA1 => Some(OpCode::IsKeyNPressed(x)), // EXA1
                    _    => None
                }
            }
            0xF => {
                match nn {
                    0x07 => Some(OpCode::SetRegToTimer(x)), // FX07
                    0x0A => Some(OpCode::AwaitKey(x)), // FX0A
                    0x15 => Some(OpCode::SetDelayTimer(x)), // FX15
                    0x18 => Some(OpCode::SetSoundTimer(x)), // FX18
                    0x1E => Some(OpCode::AddRegToI(x)), // FX1E
                    0x29 => Some(OpCode::SetIToSprite(x)), // FX29
                    0x33 => Some(OpCode::ToDecimal(x)), // FX33
                    0x55 => Some(OpCode::DumpRegs(x)), // FX55
                    0x65 => Some(OpCode::LoadRegs(x)), // FX6E
                    _    => None
                }
            }
            _ => None
        }
    }


    pub fn execute_opcode(&mut self, opcode: OpCode, input_handler: &InputHandler) -> Result<(), &'static str>{
        match opcode {
            OpCode::ClearScreen() => {
                self.buf_clear_screen();
                self.draw_flag = true;
            },
            OpCode::Return() => {
                match self.stack.pop() {
                    Some(elt) => self.pc = elt,
                    None => return Err("Tried to return from subroutine with empty stack")
                }
            }
            OpCode::Jump(nnn) => {
                self.pc = nnn
            }
            OpCode::CallSubroutine(nnn) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            OpCode::CondEq(x, nn) => {
                if self.v[x] == nn {
                    self.pc += 2
                }
            }
            OpCode::CondNEq(x, nn) => {
                if self.v[x] != nn {
                    self.pc += 2
                }
            }
            OpCode::CondEqReg(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2
                }
            }
            OpCode::SetReg(x, nn) => {
                self.v[x] = nn;
            }
            OpCode::AddToReg(x, nn) => {
                self.v[x] = u8::wrapping_add(self.v[x], nn);
            }
            OpCode::AssignRegToReg(x, y) => {
                self.v[x] = self.v[y];
            }
            OpCode::BitwiseOr(x, y) => {
                self.v[x] |= self.v[y]
            }
            OpCode::BitwiseAnd(x, y) => {
                self.v[x] &= self.v[y];
            }
            OpCode::BitwiseXor(x, y) => {
                self.v[x] ^= self.v[y];
            }
            OpCode::AddRegToReg(x, y) => {
                let (res, carry) = u8::overflowing_add(self.v[x], self.v[y]);
                self.v[x] = res;
                self.v[0xF] = carry as u8; // carry should be 1 when overflow, 0 otherwise
            }
            OpCode::SubRegToReg(x, y) => {
                let (res, carry) = u8::overflowing_sub(self.v[x], self.v[y]);
                self.v[x] = res;
                self.v[0xF] = (!carry) as u8; // carry should be 0 when overflow, 1 otherwise
            }
            OpCode::StoreLSBWithShift(x) => {
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            OpCode::SubRegFromReg(x, y) => {
                let (res, carry) = u8::overflowing_sub(self.v[y], self.v[x]);
                self.v[x] = res;
                self.v[0xF] = (!carry) as u8; // carry should be 0 when overflow, 1 otherwise
            }
            OpCode::StoreMSBWithShift(x) => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            }
            OpCode::CondNEqReg(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            OpCode::SetI(nnn) => {
                self.i = nnn;
            }
            OpCode::JumpToV0Plus(nnn) => {
                self.pc = self.v[0] as u16 + nnn;
            }
            OpCode::RegRandBitwiseAnd(x, nn) => {
                let num: u8 = rand::thread_rng().gen_range(0..255);
                self.v[x] = num & nn;
            }
            OpCode::DrawSprite(x, y, n) => {
                self.buf_draw_sprite(self.v[x], self.v[y], n);
                self.draw_flag = true;
            }
            OpCode::IsKeyPressed(x) => {
                if input_handler.is_key_down(Chip8::kkey_from_code(self.v[x]).unwrap()) {
                    self.pc += 2
                }
            }
            OpCode::IsKeyNPressed(x)=> {
                if input_handler.is_key_up(Chip8::kkey_from_code(self.v[x]).unwrap()) {
                    self.pc += 2
                }
            }
            OpCode::SetRegToTimer(x) => {
                self.v[x] = self.delay_timer as u8;
            }
            OpCode::AwaitKey(x) => {
                match input_handler.any_key_pressed() {
                    Some(key_as_hex) => self.v[x] = key_as_hex,
                    None => self.pc -= 2,
                }
            }
            OpCode::SetDelayTimer(x) => {
                self.delay_timer = self.v[x] as u32;
            }
            OpCode::SetSoundTimer(x) => {
                self.sound_timer = self.v[x] as u32;

            }
            OpCode::AddRegToI(x) => {
                self.i += self.v[x] as u16;
            }
            OpCode::SetIToSprite(x) => {
                self.i = self.font_location + 5 * ((self.v[x] % 15) as u16); 
            }
            OpCode::ToDecimal(x) => {
                self.memory[self.i as usize] = self.v[x] / 100;
                self.memory[self.i as usize + 1] = (self.v[x] % 100) / 10;
                self.memory[self.i as usize + 2] = self.v[x] % 10;
            }
            OpCode::DumpRegs(x) => {
                for i in 0x0..=x {
                    self.memory[self.i as usize + i] = self.v[i];
                }
            }
            OpCode::LoadRegs(x) => {
                for i in 0x0..=x {
                    self.v[i] = self.memory[self.i as usize + i];
                }
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn nth_nibble_test() {
        let res = Chip8::nth_nibble(0, 0x2FA3);
        assert_eq!(res, 0x2);

        let res = Chip8::nth_nibble(1, 0x2FA3);
        assert_eq!(res, 0xF);

        let res = Chip8::nth_nibble(2, 0x2FA3);
        assert_eq!(res, 0xA);
        
        let res = Chip8::nth_nibble(3, 0x2FA3);
        assert_eq!(res, 0x3);
    }

    #[test]
    pub fn range_nibble_test() {
        let res = Chip8::range_nibble(0, 1, 0x2FA3);
        assert_eq!(res, 0x2);

        let res = Chip8::range_nibble(2, 4, 0x2FA3);
        assert_eq!(res, 0xA3);

        let res = Chip8::range_nibble(0, 4, 0x2FA3);
        assert_eq!(res, 0x2FA3);
    }

    #[test]
    pub fn fetch_opcode_test() {
        let mut chip8 = Chip8::init();

        // Il faudrait plus de tests de cette fonction en théorie
        chip8.memory[chip8.pc as usize] = 0x00;
        chip8.memory[(chip8.pc + 1) as usize] = 0xE0;
        let res = Chip8::fetch_opcode(&mut chip8).unwrap();
        assert_eq!(res, OpCode::ClearScreen());
    }

}