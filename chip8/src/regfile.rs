use std::ops::{Index, IndexMut};
use instruction::Reg;

pub struct RegFile {
    gpr: [u8; 16],
}

impl RegFile {
    pub fn new() -> RegFile {
        RegFile { gpr: [0; 16] }
    }

    pub fn read_at_index(&self, index: usize) -> u8 {
        self.gpr[index]
    }

    pub fn write_at_index(&mut self, index: usize, value: u8) {
        self.gpr[index] = value;
    }
}

impl Index<Reg> for RegFile {
    type Output = u8;

    fn index(&self, index: Reg) -> &u8 {
        &self.gpr[index.index() as usize]
    }
}

impl IndexMut<Reg> for RegFile {
    fn index_mut(&mut self, index: Reg) -> &mut u8 {
        &mut self.gpr[index.index() as usize]
    }
}
