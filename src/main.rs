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
    stack: [u16; 16],
    sp: usize,
    pc: u16,
    i: u16
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096], // TODO: beware this stuff is going to be allocated on the stack
            gpr: [0; 16],
            stack: [0; 16],
            sp: 0,
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
            
            println!("{:04x}: {:04x}", self.pc, instruction);
            let next_pc = self.execute_instruction(instruction);
            self.pc = next_pc;
            
            println!("after: {:#?}", self);
        }
    }
    
    fn execute_instruction(&mut self, instruction: u16) -> u16 {
        let mut next_pc = self.pc + 2;       
        if instruction  == 0x00E0 {
            // 00E0 - CLS
            // TODO: Implement
        } else if instruction == 0x00EE {
            // 00EE - RET
            // TODO: Implement
            let retaddr = self.pop();
            next_pc = retaddr;
        } else if (instruction & 0xF000) == 0x1000 {
            // 1nnn - JP addr
            let addr = instruction & 0x0FFF;
            next_pc = addr;
        } else if (instruction & 0xF000) == 0x2000 {
            // 2nnn - CALL addr
            let addr = instruction & 0x0FFF;
            let pc = self.pc;
            self.push(next_pc);
            next_pc = addr;
        } else if (instruction & 0xF000) == 0x3000 {
            // 3xkk - SE Vx, byte
            let dst_r = ((instruction & 0x0F00) >> 8) as usize;
            let imm = (instruction & 0x00FF) as u8;
            if self.gpr[dst_r] == imm {
                next_pc += 2;
            }
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
        } else if (instruction & 0xF00F) == 0x8000 {
            // 8xy0 - LD Vx, Vy
            let dst_r = ((instruction & 0x0F00) >> 8) as usize;
            let src_r = ((instruction & 0x00F0) >> 4) as usize;
            self.gpr[dst_r] = self.gpr[src_r];
        } else if (instruction & 0xF000) == 0xA000 {
            // Annn - LD I, addr
            let addr = instruction & 0x0FFF;
            self.i = addr;
        } else if (instruction & 0xF000) == 0xD000 {
            // Dxyn - DRW Vx, Vy, nibble
            // TODO: Implement
        } else if (instruction & 0xF0FF) == 0xE0A1 {
            // ExA1 - SKNP Vx
            // TODO: Implement
        } else if (instruction & 0xF0FF) == 0xF01E {
            // Fx1E - ADD I, Vx
            let dst_r = ((instruction & 0x0F00) >> 8) as usize;
            self.i = self.i + (self.gpr[dst_r] as u16);
        }
        else {
            panic!("unrecognized instruction: {:#x}", instruction);
        }
        
        next_pc
    }
        
    fn pop(&mut self) -> u16 {
        let value = self.stack[self.sp];
        self.sp -= 1;
        value
    }

    fn push(&mut self, value: u16) {
        self.sp += 1;
        self.stack[self.sp] = value;
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Chip8 {{"));
        for i in 0..16 {
            try!(writeln!(f, "  V{:0X}: {:02x}", i, self.gpr[i]));
        }
        try!(writeln!(f, "  PC: {:04x}", self.pc));
        try!(writeln!(f, "  I : {:04x}", self.i));
        try!(writeln!(f, "  SP: {:04x}", self.sp));
        try!(writeln!(f, "  stack: ["));
        for i in 0..self.sp {
            try!(writeln!(f, "  {:01x}: {:04x}", i, self.stack[i as usize]));
        }
        try!(writeln!(f, "  ]"));
        try!(writeln!(f, "}}"));
        
        Ok(())
    }
}