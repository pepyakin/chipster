
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    mem: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Display {
    pub fn new() -> Display {
        Display { mem: [0; 64 * 32] }
    }

    pub fn clear(&mut self) {
        for i in self.mem.iter_mut() {
            *i = 0;
        }
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision_bit = false;

        for (sy, byte) in sprite.iter().enumerate() {
            let dy = (y + sy) % DISPLAY_HEIGHT;
            for sx in 0..8 {
                let bit_mask = 0b1000_0000 >> sx;
                if (byte & bit_mask) != 0 {
                    let dx = (x + sx) % DISPLAY_WIDTH;
                    let index = dy * DISPLAY_WIDTH + dx;

                    if self.mem[index] == 1 {
                        collision_bit = true;
                    }
                    self.mem[index] ^= 1;
                }
            }
        }

        collision_bit
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        let index = y * DISPLAY_WIDTH + x;
        self.mem[index]
    }
}
