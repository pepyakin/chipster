
#[derive(Copy, Clone)]
pub struct InstructionWord(pub u16);

impl InstructionWord {
    pub fn nnn(self) -> u16 {
        self.0 & 0x0FFF
    }

    pub fn kk(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn n(self) -> u8 {
        (self.0 & 0xF) as u8
    }

    pub fn x_reg(self) -> usize {
        ((self.0 & 0x0F00) >> 8) as usize
    }

    pub fn y_reg(self) -> usize {
        ((self.0 & 0x00F0) >> 4) as usize
    }
}
