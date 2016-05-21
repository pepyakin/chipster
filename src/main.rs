use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::env;
use std::fmt;

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut bin_file = File::open(path).unwrap();
    let mut bin_buffer = Vec::new();
    bin_file.read_to_end(&mut bin_buffer);
    bin_buffer.into_boxed_slice()
}

fn main() {
    let bin_file_name = env::args().nth(1).unwrap();
    let bin_data = read_bin(bin_file_name); 
       
    let mut chip8 = Chip8::new();
    chip8.execute(bin_data);
}

struct Chip8 {
    memory: [u8; 4096],
    gpr: [u8; 16],
    
    pc: u16,
    i: u16
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096], // TODO: beware this stuff is going to be allocated on the stack
            gpr: [0; 16],
            pc: 0x200,
            i: 0 // TODO: Initial value?
        }
    }
    
    fn execute(&mut self, bin_data: Box<[u8]>) { 
        loop {
            let actual_pc = (self.pc - 0x200) as usize;
            
            let first_byte = (*bin_data)[actual_pc] as u16;
            let second_byte = (*bin_data)[actual_pc + 1] as u16;
            let instruction = (first_byte << 8) | second_byte;
            
            println!("0x{:0x}: 0x{:0x}", self.pc, instruction);
            let next_pc = self.execute_instruction(instruction);
            self.pc = next_pc;
            
            println!("after: {:#?}", self);
        }
    }
    
    fn execute_instruction(&mut self, instruction: u16) -> u16 {        
        if (instruction & 0xF000) == 0x1000 {
            // 1nnn - JP addr
            let addr = instruction & 0x0FFF;
            return addr;
        } else if (instruction & 0xF000) == 0x6000 {
            // 6xkk - LD Vx, byte
            let dst_r = ((instruction & 0x0F00) >> 8) as usize;
            let imm = (instruction & 0x00FF) as u8;
            self.gpr[dst_r] = imm;
        } else if (instruction & 0xF000) == 0x7000 {
            // 7xkk - ADD Vx, byte
            let dst_r = ((instruction & 0x0F00) >> 8) as usize;
            let imm = (instruction & 0x00FF) as u8;
            self.gpr[dst_r] = self.gpr[dst_r] + imm;
        } else if (instruction & 0xF000) == 0xA000 {
            // Annn - LD I, addr
            let addr = instruction & 0x0FFF;
            self.i = addr;
        } else if (instruction & 0xF000) == 0xD000 {
            // Dxyn - DRW Vx, Vy, nibble
            // TODO: Implement
        } 
        else {
            panic!("unrecognized instruction: {:#x}", instruction);
        }
        
        self.pc + 2
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Chip8 {{"));
        for i in 0..16 {
            try!(writeln!(f, "  gpr{}: {:0x}", i, self.gpr[i]));
        }
        try!(writeln!(f, "  pc: {:0x}", self.pc));
        try!(writeln!(f, "}}"));
        
        Ok(())
    }
}