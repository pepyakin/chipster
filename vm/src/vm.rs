use std::fmt;
use rand::Rng;
use rand;

use super::stack::Stack;
use super::timer;
use super::display;
use super::instruction::*;

pub struct Chip8 {
    memory: [u8; 4096],
    gpr: [u8; 16],
    stack: Stack,
    pc: u16,
    i: u16,
    dt: timer::Timer,
    st: timer::Timer,
    pub display: display::Display,
    pub keyboard: [u8; 16],
}


impl Chip8 {
    pub fn new() -> Chip8 {
        let mut chip8 = Chip8 {
            memory: [0; 4096], // TODO: beware this stuff is going to be allocated on the stack
            gpr: [0; 16],
            stack: Stack::new(),
            pc: 0x200,
            i: 0, // TODO: Initial value?
            dt: timer::Timer::new(),
            st: timer::Timer::new(),
            display: display::Display::new(),
            keyboard: [0; 16],
        };

        for i in 0..80 {
            chip8.memory[i] = FONT_SPRITES[i];
        }
        chip8
    }

    pub fn with_bin(bin_data: Box<[u8]>) -> Chip8 {
        let mut chip8 = Chip8::new();

        for (i, octet) in bin_data.iter().enumerate() {
            chip8.memory[0x200 + i] = *octet;
        }

        chip8
    }

    pub fn cycle(&mut self) {
        let instruction_word = {
            use byteorder::{ByteOrder, BigEndian};
            let actual_pc = self.pc as usize;
            InstructionWord(BigEndian::read_u16(&self.memory[actual_pc..]))
        };
        let instruction = Instruction::decode(instruction_word);
        let next_pc = self.execute_instruction(instruction);
        self.pc = next_pc;
    }

    pub fn update_timers(&mut self, dt: f64) {
        self.dt.step(dt);
        self.st.step(dt);
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> u16 {
        use instruction::Instruction::*;
        
        let mut next_pc = self.pc + 2;

        match instruction {
            ClearScreen => self.display.clear(),
            Ret => {
                let retaddr = self.stack.pop();
                next_pc = retaddr;
            },
            Sys(addr) => {
                unimplemented!();
            },
            Jump(addr) => {
                next_pc = addr.0;
            },
            Call(addr) => {
                self.stack.push(next_pc);
                next_pc = addr.0;
            },
            SkipEqImm { 
                vx: vx,
                imm: imm,
                inv: inv
            } => {
                if !inv {
                    if self.read_gpr(vx) == imm.0 {
                        next_pc += 2;
                    }
                } else {
                    if self.read_gpr(vx) != imm.0 {
                        next_pc += 2;
                    }
                }
            },
            SkipEqReg {
                vx: vx,
                vy: vy,
                inv: inv
            } => {
                if !inv {
                    if self.read_gpr(vx) == self.read_gpr(vy) {
                        next_pc += 2;
                    }
                } else {
                    if self.read_gpr(vx) != self.read_gpr(vy) {
                        next_pc += 2;
                    }
                }
            },
            PutImm {
                vx: vx,
                imm: imm
            } => {
                self.write_gpr(vx, imm.0);
            },
            AddImm {
                vx: vx,
                imm: imm
            } => {
                let x = self.read_gpr(vx);
                self.write_gpr(vx, x.wrapping_add(imm.0));
            },
            Apply {
                vx: vx,
                vy: vy,
                f: f
            } => {
                let x = self.read_gpr(vx);
                let y = self.read_gpr(vy);
                
                match f {
                    Fun::Id => {
                        self.write_gpr(vx, y);
                    },
                    Fun::Or => {
                        self.write_gpr(vx, x | y);
                    },
                    Fun::And => {
                        self.write_gpr(vx, x & y);
                    },
                    Fun::Xor => {
                        self.write_gpr(vx, x ^ y);
                    },
                    Fun::Add => {
                        let (v, overflow) = x.overflowing_add(y);
                        self.write_gpr(vx, v);
                        self.write_gpr(Reg::Vf, if overflow { 1 } else { 0 });
                    },
                    Fun::Subtract => {
                        let (v, borrow) = x.overflowing_sub(y);
                        self.write_gpr(vx, v);
                        self.write_gpr(Reg::Vf, if borrow { 0 } else { 1 });
                    },
                    Fun::ShiftRight => {
                        self.write_gpr(vx, y >> 1);
                        self.write_gpr(Reg::Vf, y & 0x01);
                    },
                    Fun::SubtractInv => {
                        let (v, borrow) = y.overflowing_sub(x);
                        self.write_gpr(vx, v);
                        self.write_gpr(Reg::Vf, if borrow { 0 } else { 1 });
                    },
                    Fun::ShiftLeft => {
                        self.write_gpr(vx, y << 1);
                        self.write_gpr(Reg::Vf, y << 1);
                    }
                }
            },
            SetI(addr) => {
                self.i = addr.0;
            },
            JumpPlusV0(addr) => {
                panic!("instruction not implemented 0xBxxx");
            },
            Randomize { 
                vx: vx, 
                imm: imm 
            } => {
                let random_byte = rand::thread_rng().gen::<u8>();
                self.write_gpr(vx, random_byte & imm.0);
            },
            
            Draw {
                vx: vx,
                vy: vy,
                n: n
            } => {
                let x = self.read_gpr(vx) as usize;
                let y = self.read_gpr(vy) as usize;
                let from = self.i as usize;
                let to = from + (n.0 as usize);
                
                let collision_bit = {
                    let sprite = &self.memory[from..to];
                    self.display.draw(x, y, sprite)
                };
                
                self.write_gpr(Reg::Vf, if collision_bit { 1 } else { 0 });
            },
            SkipPressed {
                vx: vx,
                inv: inv
            } => {
                let x = self.read_gpr(vx) as usize;
                if !inv {
                    if self.keyboard[x] == 1 {
                        next_pc += 2;
                    }
                } else {
                    if self.keyboard[x] != 1 {
                        next_pc += 2;
                    }
                }
            },
            GetDT(vx) => {
                let dt = self.dt.get();
                self.write_gpr(vx, dt);
            },
            WaitKey(vx) => {
                panic!("instruction not implemented 0xFxxA");
            },
            SetDT(vx) => {
                let x = self.read_gpr(vx);
                self.dt.set(x);
            },
            SetST(vx) => {
                let x = self.read_gpr(vx);
                self.st.set(x);
            },
            AddI(vx) => {
                let x = self.read_gpr(vx) as u16;
                self.i = self.i.wrapping_add(x);
            },
            LoadGlyph(vx) => {
                let v = self.read_gpr(vx);
                self.i = FONT_MEMORY_OFFSET + v as u16 * 5;
            },
            StoreBCD(vx) => {
                let v = self.read_gpr(vx);
                let i = self.i as usize;
                
                self.memory[i] = v / 100;
                self.memory[i + 1] = (v / 10) % 10;
                self.memory[i + 2] = (v % 100) % 10;
            },
            StoreRegs(vx) => {
                let i = self.i as usize;
                for offset in 0..(vx.index() as usize + 1) {
                   self.memory[i + offset] = self.gpr[offset];
                }
                self.i += vx as u16 + 1;
            },
            LoadRegs(vx) => {
                let i = self.i as usize;
                for offset in 0..(vx.index() as usize + 1) {
                   self.gpr[offset] = self.memory[i + offset];
                }
                self.i += vx as u16 + 1;
            }
        }

        next_pc
    }
    
    fn read_gpr(&self, reg: Reg) -> u8 {
        self.gpr[reg.index() as usize]
    }
    
    fn write_gpr(&mut self, reg: Reg, value: u8) {
        self.gpr[reg.index() as usize] = value;
    }

    pub fn is_beeping(&self) -> bool {
        self.st.get() != 0
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
        try!(writeln!(f, "  DT: {:02x}", self.dt.get()));
        try!(writeln!(f, "}}"));

        Ok(())
    }
}

const FONT_MEMORY_OFFSET: u16 = 0;

#[cfg_attr(rustfmt, rustfmt_skip)]
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
