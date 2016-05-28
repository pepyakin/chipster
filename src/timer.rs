const TIMER_TICK_DURATION: f64 = 1.0 / 60.0;

pub struct Timer {
    left: f64,
}

impl Timer {
    pub fn new() -> Timer {
        Timer { left: 0f64 }
    }

    pub fn step(&mut self, dt: f64) {
        self.left = (0f64).max(self.left - dt);
    }

    pub fn get(&self) -> u8 {
        (self.left / TIMER_TICK_DURATION) as u8
    }

    pub fn set(&mut self, ticks: u8) {
        self.left = ticks as f64 * TIMER_TICK_DURATION;
    }
}
