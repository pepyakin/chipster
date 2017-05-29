
pub struct Timer {
    left: u8,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { left: 0 }
    }

    pub fn step(&mut self, dt: u8) {
        self.left = self.left.checked_sub(dt).unwrap_or(0);
    }

    pub fn get(&self) -> u8 {
        self.left
    }

    pub fn set(&mut self, ticks: u8) {
        self.left = ticks
    }
}
