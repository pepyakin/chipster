use std::fmt;

use rand::Rng;
use rand;

use stack::Stack;
use timer;
use display;
use instruction::*;
use regfile::RegFile;

pub struct Chip8 {
    memory: [u8; 4096],
    gpr: RegFile,
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
            memory: [0; 4096],
            gpr: RegFile::new(),
            stack: Stack::new(),
            pc: 0x200,
            i: 0, // TODO: Initial value?
            dt: timer::Timer::new(),
            st: timer::Timer::new(),
            display: display::Display::new(),
            keyboard: [0; 16],
        };

        {
            let font_memory = &mut chip8.memory[0..80];
            font_memory.copy_from_slice(&FONT_SPRITES);
        }

        chip8
    }

    pub fn with_rom(rom_data: &[u8]) -> Chip8 {
        use std::io::Write;

        let mut chip8 = Chip8::new();
        {
            let mut rom_start = &mut chip8.memory[0x200..];
            rom_start.write_all(rom_data).expect("write should succeed");
        }

        chip8
    }

    pub fn cycle(&mut self) -> ::Result<()> {
        let instruction_word = {
            use byteorder::{ByteOrder, BigEndian};
            let actual_pc = self.pc as usize;
            InstructionWord(BigEndian::read_u16(&self.memory[actual_pc..]))
        };
        let instruction = Instruction::decode(instruction_word)?;
        let next_pc = self.execute_instruction(instruction);
        self.pc = next_pc;

        Ok(())
    }

    pub fn update_timers(&mut self, dt: u8) {
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
            }
            Sys(_addr) => {
                unimplemented!();
            }
            Jump(addr) => {
                next_pc = addr.0;
            }
            Call(addr) => {
                self.stack.push(next_pc);
                next_pc = addr.0;
            }
            SkipEqImm { vx, imm, inv } => {
                if !inv {
                    if self.gpr[vx] == imm.0 {
                        next_pc += 2;
                    }
                } else {
                    if self.gpr[vx] != imm.0 {
                        next_pc += 2;
                    }
                }
            }
            SkipEqReg { vx, vy, inv } => {
                if !inv {
                    if self.gpr[vx] == self.gpr[vy] {
                        next_pc += 2;
                    }
                } else {
                    if self.gpr[vx] != self.gpr[vy] {
                        next_pc += 2;
                    }
                }
            }
            PutImm { vx, imm } => {
                self.gpr[vx] = imm.0;
            }
            AddImm { vx, imm } => {
                let x = self.gpr[vx];
                self.gpr[vx] = x.wrapping_add(imm.0);
            }
            Apply { vx, vy, f } => {
                let x = self.gpr[vx];
                let y = self.gpr[vy];

                match f {
                    Fun::Id => {
                        self.gpr[vx] = y;
                    }
                    Fun::Or => {
                        self.gpr[vx] = x | y;
                    }
                    Fun::And => {
                        self.gpr[vx] = x & y;
                    }
                    Fun::Xor => {
                        self.gpr[vx] = x ^ y;
                    }
                    Fun::Add => {
                        let (v, overflow) = x.overflowing_add(y);
                        self.gpr[vx] = v;
                        self.gpr[Reg::Vf] = if overflow { 1 } else { 0 };
                    }
                    Fun::Subtract => {
                        let (v, borrow) = x.overflowing_sub(y);
                        self.gpr[vx] = v;
                        self.gpr[Reg::Vf] = if borrow { 0 } else { 1 };
                    }
                    Fun::ShiftRight => {
                        self.gpr[vx] = y >> 1;
                        self.gpr[Reg::Vf] = y & 0x01;
                    }
                    Fun::SubtractInv => {
                        let (v, borrow) = y.overflowing_sub(x);
                        self.gpr[vx] = v;
                        self.gpr[Reg::Vf] = if borrow { 0 } else { 1 };
                    }
                    Fun::ShiftLeft => {
                        self.gpr[vx] = y << 1;
                        self.gpr[Reg::Vf] = y << 1;
                    }
                }
            }
            SetI(addr) => {
                self.i = addr.0;
            }
            JumpPlusV0(_addr) => {
                panic!("instruction not implemented 0xBxxx");
            }
            Randomize { vx, imm } => {
                let random_byte = rand::thread_rng().gen::<u8>();
                self.gpr[vx] = random_byte & imm.0;
            }

            Draw { vx, vy, n } => {
                let x = self.gpr[vx] as usize;
                let y = self.gpr[vy] as usize;
                let from = self.i as usize;
                let to = from + (n.0 as usize);

                let collision_bit = {
                    let sprite = &self.memory[from..to];
                    self.display.draw(x, y, sprite)
                };

                self.gpr[Reg::Vf] = if collision_bit { 1 } else { 0 };
            }
            SkipPressed { vx, inv } => {
                let x = self.gpr[vx] as usize;
                if !inv {
                    if self.keyboard[x] == 1 {
                        next_pc += 2;
                    }
                } else {
                    if self.keyboard[x] != 1 {
                        next_pc += 2;
                    }
                }
            }
            GetDT(vx) => {
                let dt = self.dt.get();
                self.gpr[vx] = dt;
            }
            WaitKey(_vx) => {
                panic!("instruction not implemented 0xFxxA");
            }
            SetDT(vx) => {
                let x = self.gpr[vx];
                self.dt.set(x);
            }
            SetST(vx) => {
                let x = self.gpr[vx];
                self.st.set(x);
            }
            AddI(vx) => {
                let x = self.gpr[vx] as u16;
                self.i = self.i.wrapping_add(x);
            }
            LoadGlyph(vx) => {
                let v = self.gpr[vx];
                self.i = FONT_MEMORY_OFFSET + v as u16 * 5;
            }
            StoreBCD(vx) => {
                let v = self.gpr[vx];
                let i = self.i as usize;

                self.memory[i] = v / 100;
                self.memory[i + 1] = (v / 10) % 10;
                self.memory[i + 2] = (v % 100) % 10;
            }
            StoreRegs(vx) => {
                let i = self.i as usize;
                for offset in 0..(vx.index() as usize + 1) {
                    self.memory[i + offset] = self.gpr.read_at_index(offset);
                }
                self.i += vx as u16 + 1;
            }
            LoadRegs(vx) => {
                let i = self.i as usize;
                for offset in 0..(vx.index() as usize + 1) {
                    self.gpr.write_at_index(offset, self.memory[i + offset]);
                }
                self.i += vx as u16 + 1;
            }
        }

        next_pc
    }

    pub fn is_beeping(&self) -> bool {
        self.st.get() != 0
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Chip8")
            .field("gpr", &self.gpr)
            .field("pc", &format!("{:04x}", self.pc))
            .field("i", &format!("{:04x}", self.i))
            .field("dt", &format!("{:02x}", self.dt.get()))
            .field("st", &format!("{:02x}", self.st.get()))
            .field("stack", &self.stack)
            .finish()
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
