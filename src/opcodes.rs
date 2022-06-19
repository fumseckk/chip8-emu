use crate::{Chip8, Display};
use termkan::{input::KeyEvent, rds::Renderer};
use rand::Rng;

type N = u8;
type NN = u8;
type NNN = u16;
type VX = usize;
type VY = usize;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpCode {
    ClearScreen(),
    Return(),
    Jump(NNN),
    CallSubroutine(NNN),
    CondEq(VX, NN),
    CondNEq(VX, NN),
    CondEqReg(VX, VY),
    SetReg(VX, NN),
    AddToReg(VX, NN), // no change to carry
    AssignRegToReg(VX, VY),
    BitwiseOr(VX, VY),
    BitwiseAnd(VX, VY),
    BitwiseXor(VX, VY),
    AddRegToReg(VX, VY), // with carry
    SubRegToReg(VX, VY), // with carry
    StoreLSBWithShift(VX),
    SubRegFromReg(VX, VY), // with carry
    StoreMSBWithShift(VX),
    CondNEqReg(VX, VY),
    SetI(NNN),
    JumpToV0Plus(NNN),
    RegRandBitwiseAnd(VX, NN), // rand: from 0 to 255
    DrawSprite(VX, VY, N), // with carry
    IsKeyPressed(VX),
    IsKeyNPressed(VX),
    SetRegToTimer(VX),
    AwaitKey(VX), // blocking op
    SetDelayTimer(VX),
    SetSoundTimer(VX),
    AddRegToI(VX),
    SetIToSprite(VX),
    ToDecimal(VX),
    DumpRegs(VX),
    LoadRegs(VX),
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

        let X = Self::nth_nibble(1, opcode) as usize;
        let Y = Self::nth_nibble(2, opcode) as usize;
        let N = Self::nth_nibble(3, opcode) as u8;
        let NN = Self::range_nibble(2, 4, opcode) as u8;
        let NNN = Self::range_nibble(1, 4, opcode);

        match Self::nth_nibble(0, opcode) {
            0x0 => {
                match opcode {
                    0x00E0 => Some(OpCode::ClearScreen()), // 00E0
                    0x00EE => Some(OpCode::Return()), // 00EE
                    _      => None
                }
            }
            0x1 => Some(OpCode::Jump(NNN)), // 1NNN
            0x2 => Some(OpCode::CallSubroutine(NNN)), // 2NNN
            0x3 => Some(OpCode::CondEq(X, NN)), // 3XNN
            0x4 => Some(OpCode::CondNEq(X, NN)), // 4XNN
            0x5 => {
                match N {
                    0x0 => Some(OpCode::CondEqReg(X, Y)), // 5XY0
                    _   => None
                }
            }
            0x6 => Some(OpCode::SetReg(X, NN)), // 6XNN
            0x7 => Some(OpCode::AddToReg(X, NN)), // 7XNN
            0x8 => {
                match N {
                    0 => Some(OpCode::AssignRegToReg(X, Y)), // 8XY0
                    1 => Some(OpCode::BitwiseOr(X, Y)), // 8XY1
                    2 => Some(OpCode::BitwiseAnd(X, Y)), // 8XY2
                    3 => Some(OpCode::BitwiseXor(X, Y)), // 8XY3
                    4 => Some(OpCode::AddRegToReg(X, Y)), // 8XY4
                    5 => Some(OpCode::SubRegToReg(X, Y)), // 8XY5
                    6 => Some(OpCode::StoreLSBWithShift(X)), // 8XY6
                    7 => Some(OpCode::SubRegFromReg(X, Y)), // 8XY7
                    E => Some(OpCode::StoreMSBWithShift(X)), // 8XYE
                }
            }
            0x9 => {
                match N {
                    0x0 => Some(OpCode::CondNEqReg(X, Y)), // 9XY0
                    _   => None
                }
            }
            0xA => Some(OpCode::SetI(NNN)), // ANNN
            0xB => Some(OpCode::JumpToV0Plus(NNN)), // BNNN
            0xC => Some(OpCode::RegRandBitwiseAnd(X, NN)), // CXNN
            0xD => Some(OpCode::DrawSprite(X, Y, N)), // DXYN
            0xE => {
                match NN {
                    0x9E => Some(OpCode::IsKeyPressed(X)), // EX9E
                    0xA1 => Some(OpCode::IsKeyNPressed(X)), // EXA1
                    _    => None
                }
            }
            0xF => {
                match NN {
                    0x07 => Some(OpCode::SetRegToTimer(X)), // FX07
                    0x0A => Some(OpCode::AwaitKey(X)), // FX0A
                    0x15 => Some(OpCode::SetDelayTimer(X)), // FX15
                    0x18 => Some(OpCode::SetSoundTimer(X)), // FX18
                    0x1E => Some(OpCode::AddRegToI(X)), // FX1E
                    0x29 => Some(OpCode::SetIToSprite(X)), // FX29
                    0x33 => Some(OpCode::ToDecimal(X)), // FX33
                    0x55 => Some(OpCode::DumpRegs(X)), // FX55
                    0x65 => Some(OpCode::LoadRegs(X)), // FX6E
                    _    => None
                }
            }
            _ => None
        }
    }


    pub fn execute_opcode(&mut self, opcode: OpCode, display: &Display) -> Result<(), &'static str>{
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
            OpCode::Jump(NNN) => {
                self.pc = NNN
            }
            OpCode::CallSubroutine(NNN) => {
                self.stack.push(self.pc);
                self.pc = NNN;
            }
            OpCode::CondEq(X, NN) => {
                if self.V[X] == NN {
                    self.pc += 2
                }
            }
            OpCode::CondNEq(X, NN) => {
                if self.V[X] != NN {
                    self.pc += 2
                }
            }
            OpCode::CondEqReg(X, Y) => {
                if self.V[X] == self.V[Y] {
                    self.pc += 2
                }
            }
            OpCode::SetReg(X, NN) => {
                self.V[X] = NN;
            }
            OpCode::AddToReg(X, NN) => {
                self.V[X] = u8::wrapping_add(self.V[X], NN);
            }
            OpCode::AssignRegToReg(X, Y) => {
                self.V[X] = self.V[Y];
            }
            OpCode::BitwiseOr(X, Y) => {
                self.V[X] |= self.V[Y]
            }
            OpCode::BitwiseAnd(X, Y) => {
                self.V[X] &= self.V[Y];
            }
            OpCode::BitwiseXor(X, Y) => {
                self.V[X] ^= self.V[Y];
            }
            OpCode::AddRegToReg(X, Y) => {
                let (res, carry) = u8::overflowing_add(self.V[X], self.V[Y]);
                self.V[X] = res;
                self.V[0xF] = carry as u8; // carry should be 1 when overflow, 0 otherwise
            }
            OpCode::SubRegToReg(X, Y) => {
                let (res, carry) = u8::overflowing_sub(self.V[X], self.V[Y]);
                self.V[X] = res;
                self.V[0xF] = (!carry) as u8; // carry should be 0 when overflow, 1 otherwise
            }
            OpCode::StoreLSBWithShift(X) => {
                self.V[0xF] = self.V[X] & 1;
                self.V[X] >>= 1;
            }
            OpCode::SubRegFromReg(X, Y) => {
                let (res, carry) = u8::overflowing_sub(self.V[Y], self.V[X]);
                self.V[X] = res;
                self.V[0xF] = (!carry) as u8; // carry should be 0 when overflow, 1 otherwise
            }
            OpCode::StoreMSBWithShift(X) => {
                self.V[0xF] = self.V[X] >> 7;
                self.V[X] <<= 1;
            }
            OpCode::CondNEqReg(X, Y) => {
                if self.V[X] != self.V[Y] {
                    self.pc += 2;
                }
            }
            OpCode::SetI(NNN) => {
                self.i = NNN;
            }
            OpCode::JumpToV0Plus(NNN) => {
                self.pc = self.V[0] as u16 + NNN;
            }
            OpCode::RegRandBitwiseAnd(X, NN) => {
                let num: u8 = rand::thread_rng().gen_range(0..255);
                self.V[X] = num & NN;
            }
            OpCode::DrawSprite(X, Y, N) => {
                self.buf_draw_sprite(self.V[X], self.V[Y], N);
                self.draw_flag = true;
            }
            OpCode::IsKeyPressed(X) => {
                if display.is_key_down(Chip8::kkey_from_code(self.V[X]).unwrap()) {
                    self.pc += 2
                }
            }
            OpCode::IsKeyNPressed(X)=> {
                if display.is_key_up(Chip8::kkey_from_code(self.V[X]).unwrap()) {
                    self.pc += 2
                }
            }
            OpCode::SetRegToTimer(X) => {
                self.V[X] = self.delay_timer as u8;
            }
            OpCode::AwaitKey(X) => {
                match display.any_key_pressed() {
                    Some(key_as_hex) => self.V[X] = key_as_hex,
                    None => self.pc -= 2,
                }
            }
            OpCode::SetDelayTimer(X) => {
                self.delay_timer = self.V[X] as u32;
            }
            OpCode::SetSoundTimer(X) => {
                self.sound_timer = self.V[X] as u32;

            }
            OpCode::AddRegToI(X) => {
                self.i += self.V[X] as u16;
            }
            OpCode::SetIToSprite(X) => {
                self.i = self.font_location + 5 * ((self.V[X] % 15) as u16); 
            }
            OpCode::ToDecimal(X) => {
                self.memory[self.i as usize] = self.V[X] / 100;
                self.memory[self.i as usize + 1] = (self.V[X] % 100) / 10;
                self.memory[self.i as usize + 2] = self.V[X] % 10;
            }
            OpCode::DumpRegs(X) => {
                for i in 0x0..=X {
                    self.memory[self.i as usize + i] = self.V[i];
                }
            }
            OpCode::LoadRegs(X) => {
                for i in 0x0..=X {
                    self.V[i] = self.memory[self.i as usize + i];
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