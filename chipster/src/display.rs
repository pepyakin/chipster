
use chip8::display as c8_display;

pub struct Display {
    pixel_decay_time: f32,
    pixel_intensity: [f32; 64 * 32], // todo: rename to intensity?
}

impl Display {
    pub fn new(pixel_decay_time: f32) -> Display {
        Display {
            pixel_decay_time,
            pixel_intensity: [0.0; 64 * 32],
        }
    }

    pub fn update(&mut self, new_frame: &c8_display::Display, dt: f32) {
        for y in 0..32 {
            for x in 0..64 {
                if new_frame.get(x, y) != 0 {
                    self.pixel_intensity[y * 64 + x] = 1.0;
                } else {
                    let current_intensity = self.pixel_intensity[y * 64 + x];
                    let new_intensity = f32::max(0.0, current_intensity - (dt / self.pixel_decay_time));
                    self.pixel_intensity[y * 64 + x] = new_intensity;
                }
            }
        }
    }

    pub fn get_intensity(&self, x: usize, y: usize) -> f32 {
        self.pixel_intensity[y * 64 + x]
    }
}
