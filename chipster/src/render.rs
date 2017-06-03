
use std::cell::RefCell;
use std::rc::Rc;
use chip8::display as c8_display;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

#[derive(Clone, Copy)]
enum PixelState {
    On,
    Decaying { age: f32 },
    Off,
}

impl PixelState {
    fn step(self, on: bool, delta_age: f32) -> PixelState {
        use self::PixelState::*;
        match (on, self) {
            (true, _) => On,
            (false, Off) => Off,
            (false, Decaying { age }) if age >= 1.0 => Off,
            (false, Decaying { age }) => Decaying { age: age + delta_age },
            (false, On) => Decaying { age: 0.0 },
        }
    }
}

pub struct RenderBuf {
    pixel_decay_time: f32,
    video_mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>,
    state: [PixelState; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pixel_intensity: [f32; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl RenderBuf {
    pub fn new(pixel_decay_time: f32) -> RenderBuf {
        let video_mem = Rc::new(RefCell::new([false; DISPLAY_WIDTH * DISPLAY_HEIGHT]));

        RenderBuf {
            pixel_decay_time,
            video_mem,
            state: [PixelState::Off; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            pixel_intensity: [0.0; DISPLAY_WIDTH * DISPLAY_HEIGHT],
        }
    }

    pub fn update(&mut self, dt: f32) {
        use self::PixelState::*;

        // delta time represented in pixel age.
        let delta_age = dt / self.pixel_decay_time;

        let new_frame = self.video_mem.borrow_mut();

        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let on = new_frame[y * DISPLAY_WIDTH + x];
                let prev_state = self.state[y * DISPLAY_WIDTH + x];
                let new_state = prev_state.step(on, delta_age);

                self.state[y * DISPLAY_WIDTH + x] = new_state;
                self.pixel_intensity[y * DISPLAY_WIDTH + x] = match new_state {
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
        self.pixel_intensity[y * DISPLAY_WIDTH + x]
    }

    pub fn display(&self) -> RenderBufDisplay {
        RenderBufDisplay { mem: self.video_mem.clone() }
    }
}

pub struct RenderBufDisplay {
    mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>,
}

impl c8_display::Display for RenderBufDisplay {
    fn clear(&mut self) {
        for i in self.mem.borrow_mut().iter_mut() {
            *i = false;
        }
    }

    fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision_bit = false;
        let mut mem = self.mem.borrow_mut();

        for (sy, byte) in sprite.iter().enumerate() {
            let dy = (y + sy) % DISPLAY_HEIGHT;
            for sx in 0..8 {
                let bit_mask = 0b1000_0000 >> sx;
                if (byte & bit_mask) != 0 {
                    let dx = (x + sx) % DISPLAY_WIDTH;
                    let index = dy * DISPLAY_WIDTH + dx;

                    if mem[index] {
                        collision_bit = true;
                    }
                    mem[index] ^= true;
                }
            }
        }

        collision_bit
    }
}
