
use chip8::display as c8_display;
use std::default::Default;

/// How long it takes in seconds for pixel to go from lit to non-lit.
const PIXEL_DECAY_TIME: f32 = 0.9;

pub struct Display {
    pixels: [f32; 64 * 32], // todo: rename to intensity?
}

impl Display {
    pub fn new() -> Display {
        Display {
            pixels: [0.0; 64 * 32],
        }
    }

    pub fn update(&mut self, new_frame: &c8_display::Display, dt: f32) {
        for y in 0..32 {
            for x in 0..64 {
                if new_frame.get(x, y) != 0 {
                    self.pixels[y * 64 + x] = 1.0;
                } else {



                    let current_intensity = self.pixels[y * 64 + x];
                    let new_intensity = f32::max(0.0, current_intensity - (dt / PIXEL_DECAY_TIME));
                    self.pixels[y * 64 + x] = new_intensity;
                }
            }
        }
    }

    pub fn get_intensity(&self, x: usize, y: usize) -> f32 {
        self.pixels[y * 64 + x]
    }
}
