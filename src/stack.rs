use std::fmt;

pub struct Stack {
    sp: usize,
    frames: [u16; 16]
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            sp: 0,
            frames: [0; 16]
        }
    }
    
    pub fn pop(&mut self) -> u16 {
        let value = self.frames[self.sp];
        self.sp -= 1;
        value
    }

    pub fn push(&mut self, value: u16) {
        let new_sp = self.sp + 1;
        if new_sp > 15 {
            panic!("stackoverflow! stack: {:?}", self.frames);
        }
        
        self.sp = new_sp;
        self.frames[new_sp] = value;
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "Stack {{"));
        try!(writeln!(f, "["));
        for i in 0..(self.sp + 1) {
            try!(writeln!(f, "    {:01x}: {:04x}", i, self.frames[i as usize]));
        }
        try!(writeln!(f, "]"));
        try!(writeln!(f, "  SP: {:04x} ({})", self.sp, self.sp));
        try!(writeln!(f, "}}"));
        
        Ok(())
    }
}
