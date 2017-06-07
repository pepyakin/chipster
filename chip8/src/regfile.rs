use core::ops::{Index, IndexMut};
use core::fmt;
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

impl fmt::Debug for RegFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut dbg = f.debug_struct("RegFile");
        for i in 0..16 {
            let reg_name = format!("V{:0X}", i);
            let reg_value = format!("{:02x}", self.read_at_index(i));
            dbg.field(&reg_name, &reg_value);
        }
        dbg.finish()
    }
}
