
use chip8::display as c8_display;

#[derive(Clone, Copy)]
enum PixelState {
    On,
    Decaying { age: f32 },
    Off
}

impl PixelState {
    fn step(self, on: bool, delta_age: f32) -> PixelState {
        use self::PixelState::*;
        match (on, self) {
            (true, _) => On,
            (false, Off) => Off,
            (false, Decaying { age }) if age >= 1.0 => Off,
            (false, Decaying { age }) => Decaying { age: age + delta_age },
            (false, On) => Decaying { age: 0.0 }
        }
    }
}

pub struct Display {
    pixel_decay_time: f32,
    state: [PixelState; 64 * 32],
    pixel_intensity: [f32; 64 * 32],
}

impl Display {
    pub fn new(pixel_decay_time: f32) -> Display {
        Display {
            pixel_decay_time,
            state: [PixelState::Off; 64 * 32],
            pixel_intensity: [0.0; 64 * 32],
        }
    }

    pub fn update(&mut self, new_frame: &c8_display::Display, dt: f32) {
        use self::PixelState::*;

        // delta time represented in pixel age.
        let delta_age = dt / self.pixel_decay_time;

        for y in 0..32 {
            for x in 0..64 {
                let on = new_frame.get(x, y) != 0;
                let prev_state = self.state[y * 64 + x];
                let new_state = prev_state.step(on, delta_age);
                
                self.state[y * 64 + x] = new_state;
                self.pixel_intensity[y * 64 + x] = match new_state {
                    Off => 0.0,
                    On => 1.0,
                    Decaying { age } => Self::pixel_intensity_for_age(age),
                };
            }
        }
    }

    fn pixel_intensity_for_age(age: f32) -> f32 {
        // let c = 1.0 - age;
        let c = -2f32.powf((age - 1.0) * 5.0) + 1.0;

        // clamp
        f32::min(f32::max(0.0, c), 1.0)
    }

    pub fn get_intensity(&self, x: usize, y: usize) -> f32 {
        self.pixel_intensity[y * 64 + x]
    }
}
