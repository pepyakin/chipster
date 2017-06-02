use std::fmt;

pub struct Stack {
    sp: usize,
    frames: [u16; 16],
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            sp: 0,
            frames: [0; 16],
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
        let values = &self.frames[0..(self.sp + 1)];
        f.debug_struct("Stack")
            .field("values", &values)
            .field("sp", &self.sp)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn pop_from_empty() {
        let mut stack = Stack::new();
        stack.pop();
    }

    #[test]
    fn simple_push_pop() {
        let mut stack = Stack::new();
        stack.push(128);
        assert_eq!(128, stack.pop());
    }
}
