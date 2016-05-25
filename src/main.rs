extern crate rand;

mod stack;

use stack::Stack;

use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::env;
use std::fmt;

use rand::Rng;

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

pub struct Chip8 {
    memory: [u8; 4096],
    gpr: [u8; 16],
    stack: Stack,
    pc: u16,
    i: u16,
    dt: u8,
    st: u8
}

#[derive(Copy, Clone)]
struct Instruction(u16);

impl Instruction {
    fn nnn(self) -> u16 {
        self.0 & 0x0FFF
    }
    fn kk(self) -> u8 {
        (self.0 & 0xFF) as u8
    }
    
    fn x_reg(self) -> usize {
        ((self.0 & 0x0F00) >> 8) as usize
    }
    
    fn y_reg(self) -> usize {
        ((self.0 & 0x00F0) >> 4) as usize
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; 4096], // TODO: beware this stuff is going to be allocated on the stack
            gpr: [0; 16],
            stack: Stack::new(),
            pc: 0x200,
            i: 0, // TODO: Initial value?
            dt: 0,
            st: 0
        };
        
        for i in 0..80 {
            chip8.memory[i] = FONT_SPRITES[i]; 
        }
        chip8
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
            
            // TODO: This is terribly inaccurate approximation.
            if self.dt > 0 {
                self.dt -= 1;
            }
            if self.st > 0 {
                self.st -= 1;
            }
            
            // println!("after: {:#?}", self);
        }
    }
    
    fn execute_instruction(&mut self, instruction: u16) -> u16 {
        let parsed = Instruction(instruction);
        
        let mut next_pc = self.pc + 2;       
        if instruction  == 0x00E0 {
            // 00E0 - CLS
            // TODO: Implement
        } else if instruction == 0x00EE {
            // 00EE - RET
            let retaddr = self.stack.pop();
            next_pc = retaddr;
            
            println!("RET to {:0x}@ PC:{:0x}", retaddr, self.pc);
        } else if (instruction & 0xF000) == 0x1000 {
            // 1nnn - JP addr
            let addr = parsed.nnn();
            next_pc = addr;
        } else if (instruction & 0xF000) == 0x2000 {
            // 2nnn - CALL addr
            let addr = parsed.nnn();
            let pc = self.pc;
            self.stack.push(next_pc);
            next_pc = addr;
            
            println!("CALL {:0x} @ PC:{:0x}, stack: {:#?}", addr, self.pc, self.stack);
        } else if (instruction & 0xF000) == 0x3000 {
            // 3xkk - SE Vx, byte
            let vx = parsed.x_reg();
            let imm = parsed.kk();
            if self.gpr[vx] == imm {
                next_pc += 2;
            }
        } else if (instruction & 0xF000) == 0x4000 {
            // 4xkk - SNE Vx, byte
            let vx = parsed.x_reg();
            let imm = parsed.kk();
            if self.gpr[vx] != imm {
                next_pc += 2;
            }
        } else if (instruction & 0xF000) == 0x6000 {
            // 6xkk - LD Vx, byte
            let vx = parsed.x_reg();
            let imm = parsed.kk();
            self.gpr[vx] = imm;
        } else if (instruction & 0xF000) == 0x7000 {
            // 7xkk - ADD Vx, byte
            let vx = parsed.x_reg();
            let imm = parsed.kk();
            self.gpr[vx] = self.gpr[vx].wrapping_add(imm);
        } else if (instruction & 0xF00F) == 0x8000 {
            // 8xy0 - LD Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            self.gpr[vx] = self.gpr[vy];
        } else if (instruction & 0xF00F) == 0x8001 {
            // Set Vx = Vx OR Vy.
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            self.gpr[vx] = self.gpr[vx] | self.gpr[vy];
        } else if (instruction & 0xF00F) == 0x8002 {
            // 8xy2 - AND Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            self.gpr[vx] = self.gpr[vx] & self.gpr[vy];
        } else if (instruction & 0xF00F) == 0x8003 {
            // 8xy3 - XOR Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            self.gpr[vx] = self.gpr[vx] ^ self.gpr[vy];
        } else if (instruction & 0xF00F) == 0x8004 {
            // 8xy4 - ADD Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            let (v, overflow) = self.gpr[vx].overflowing_add(self.gpr[vy]);
            self.gpr[vx] = v;
            self.gpr[VF] = if overflow { 1 } else { 0 }; 
        } else if (instruction & 0xF00F) == 0x8005 {
            // 8xy5 - SUB Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            
            let minuend = self.gpr[vx];
            let subtrahend = self.gpr[vy];
            let (v, borrow) = minuend.overflowing_sub(subtrahend);
        
            self.gpr[vx] = v;            
            self.gpr[VF] = if borrow { 0 } else { 1 } 
        } else if (instruction & 0xF00F) == 0x8006 {
            // 8xy6 - SHR Vx {, Vy}
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            
            self.gpr[VF] = self.gpr[vy] & 0x01;
            self.gpr[vx] = self.gpr[vy] >> 1;
        } else if (instruction & 0xF00F) == 0x8007 {
            // 8xy7 - SUBN Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            
            let minuend = self.gpr[vx as usize];
            let subtrahend = self.gpr[vy as usize];
            
            let (v, borrow) = subtrahend.overflowing_sub(minuend);
            
            self.gpr[vx] = v;
            self.gpr[VF] = if borrow { 0 } else { 1 };
        } else if (instruction & 0xF00F) == 0x800E {
            // 8xyE - SHL Vx {, Vy}
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            
            self.gpr[VF] = self.gpr[vy] >> 7;
            self.gpr[vx] = self.gpr[vy] << 1;
        } else if (instruction & 0xF000) == 0x9000 {
            // 9xy0 - SNE Vx, Vy
            let vx = parsed.x_reg();
            let vy = parsed.y_reg();
            if self.gpr[vx] != self.gpr[vy] {
                next_pc += 1;
            }
        } else if (instruction & 0xF000) == 0xA000 {
            // Annn - LD I, addr
            let addr = parsed.nnn();
            self.i = addr;
        } else if (instruction & 0xF000) == 0xC000 {
            // Cxkk - RND Vx, byte
            let vx = parsed.x_reg();
            let imm = parsed.kk();
            let random_byte = rand::thread_rng().gen::<u8>();
            self.gpr[vx] = random_byte & imm;
        } else if (instruction & 0xF000) == 0xD000 {
            // Dxyn - DRW Vx, Vy, nibble
            // TODO: Implement
        } else if (instruction & 0xF0FF) == 0xE09E {
            // Ex9E - SKP Vx
            // TODO: Implement
        } else if (instruction & 0xF0FF) == 0xE0A1 {
            // ExA1 - SKNP Vx
            // TODO: Implement
        } else if (instruction & 0xF0FF) == 0xF007 {
            // Fx07 - LD Vx, DT
            let vx = parsed.x_reg();
            self.gpr[vx] = self.dt;
        } else if (instruction & 0xF0FF) == 0xF00A {
            // Fx0A - LD Vx, K
            let vx = parsed.x_reg();
            self.gpr[vx] = 0; // TODO: Wait for actual keyboard input.
        } else if (instruction & 0xF0FF) == 0xF015 {
            // Fx15 - LD DT, Vx
            let vx = parsed.x_reg();
            self.dt = self.gpr[vx];
        } else if (instruction & 0xF0FF) == 0xF018 {
            // Fx18 - LD ST, Vx
            let vx = parsed.x_reg();
            self.st = self.gpr[vx];
        } else if (instruction & 0xF0FF) == 0xF01E {
            // Fx1E - ADD I, Vx
            let vx = parsed.x_reg();
            self.i = self.i.wrapping_add(self.gpr[vx] as u16);
        } else if (instruction & 0xF0FF) == 0xF029 {
            // Fx29 - LD F, Vx
            let vx = parsed.x_reg();
            let v = self.gpr[vx];
            self.i = FONT_MEMORY_OFFSET + v as u16 * 5;
        } else if (instruction & 0xF0FF) == 0xF033 {
            // Fx33 - LD B, Vx
            let vx = parsed.x_reg();
            let v = self.gpr[vx];
            let i = self.i as usize;
            
            self.memory[i] = v / 100;
            self.memory[i + 1] = (v / 10) % 10;
            self.memory[i + 2] = (v % 100) % 10; 
        } else if (instruction & 0xF0FF) == 0xF065 {
            // Fx65 - LD Vx, [I]
            let vx = parsed.x_reg();
            let i = self.i as usize;
            for offset in 0..vx {
                self.gpr[offset] = self.memory[i + offset]; 
            }
            self.i += vx as u16 + 1;
        }
        else {
            panic!("unrecognized instruction: {:#x}", instruction);
        }
        
        next_pc
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
        try!(writeln!(f, "  stack: {:#?}", self.stack));
        try!(writeln!(f, "  DT: {:02x}", self.dt));
        try!(writeln!(f, "}}"));
        
        Ok(())
    }
}

const FONT_MEMORY_OFFSET: u16 = 0;
const FONT_SPRITES: [u8; 80] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const VF: usize = 0x0F;

#[cfg(test)]
mod test {
    use super::*;
    
    impl Chip8 {
        fn is_borrow_bit_set(&self) -> bool {
            self.gpr[VF] == 0x00
        }
    }
    
    #[test]
    fn op_sub_eq() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6105); // LD  V1,  5
        chip8.execute_instruction(0x6205); // LD  V2,  5
        chip8.execute_instruction(0x8125); // SUB V1, V2
        
        let result = chip8.gpr[0x01];
        let borrowed = chip8.is_borrow_bit_set();  
        
        assert_eq!(result, 0);
        assert_eq!(borrowed, false); 
    }
    
    #[test]
    fn op_sub_normal() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x610A); // LD  V1, 10
        chip8.execute_instruction(0x6205); // LD  V2,  5
        chip8.execute_instruction(0x8125); // SUB V1, V2
        
        let result = chip8.gpr[0x01];  
        
        assert_eq!(result, 5);
        assert_eq!(chip8.is_borrow_bit_set(), false);
    }
    
    #[test]
    fn op_sub_borrow() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6105); // LD  V1,  5
        chip8.execute_instruction(0x620A); // LD  V2, 10
        chip8.execute_instruction(0x8125); // SUB V1, V2
        
        let result = chip8.gpr[0x01];  
        
        assert_eq!(result, -5i8 as u8);
        assert_eq!(chip8.is_borrow_bit_set(), true);
    }
    
    #[test]
    fn op_shr_shifted_1() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6103); // LD   V1, 0b0000_0011
        chip8.execute_instruction(0x8216); // SHR  V2, V1
        
        let result = chip8.gpr[0x02];
        let shifted_bit = chip8.gpr[VF];
        
        assert_eq!(result, 1);
        assert_eq!(shifted_bit, 1);
    }
    
    #[test]
    fn op_shr_shifted_0() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6106); // LD   V1, 0b0000_0110
        chip8.execute_instruction(0x8216); // SHR  V2, V1
        
        let result = chip8.gpr[0x02];
        let shifted_bit = chip8.gpr[VF];
        
        assert_eq!(result, 3);
        assert_eq!(shifted_bit, 0);
    }
    
    #[test]
    fn op_subn_eq() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6105); // LD   V1,  5
        chip8.execute_instruction(0x6205); // LD   V2,  5
        chip8.execute_instruction(0x8127); // SUBN V1, V2
        
        let result = chip8.gpr[0x01];
        
        assert_eq!(result, 0);
        assert_eq!(chip8.is_borrow_bit_set(), false);   
    }
    
    #[test]
    fn op_subn_normal() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6105); // LD   V1,  5
        chip8.execute_instruction(0x620A); // LD   V2, 10
        chip8.execute_instruction(0x8127); // SUBN V1, V2
        
        let result = chip8.gpr[0x01];
        
        assert_eq!(result, 5);
        assert_eq!(chip8.is_borrow_bit_set(), false);
    }
    
    #[test]
    fn op_subn_borrow() {
        let mut chip8 = Chip8::new();
            
        chip8.execute_instruction(0x610A); // LD   V1, 10
        chip8.execute_instruction(0x6205); // LD   V2,  5
        chip8.execute_instruction(0x8127); // SUBN V1, V2
        
        let result = chip8.gpr[0x01];
        
        assert_eq!(result, -5i8 as u8);
        assert_eq!(chip8.is_borrow_bit_set(), true);
    }
    
    #[test]
    fn op_shl_shifted_1() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x61C0); // LD   V1, 0b1100_0000
        chip8.execute_instruction(0x821E); // SHL  V2, V1
        
        let result = chip8.gpr[0x02];
        let shifted_bit = chip8.gpr[VF];
        
        assert_eq!(result, 0x80);
        assert_eq!(shifted_bit, 1);
    }
    
    #[test]
    fn op_shl_shifted_0() {
        let mut chip8 = Chip8::new();
        
        chip8.execute_instruction(0x6160); // LD   V1, 0b0110_0000
        chip8.execute_instruction(0x821E); // SHL  V2, V1
        
        let result = chip8.gpr[0x02];
        let shifted_bit = chip8.gpr[VF];
        
        assert_eq!(result, 0xC0);
        assert_eq!(shifted_bit, 0);
    }
}
