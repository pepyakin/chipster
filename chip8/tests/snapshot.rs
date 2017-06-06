extern crate chip8;
extern crate rand;

use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use chip8::{Vm, Env};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct RenderBufDisplay {
    mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>,
}

impl RenderBufDisplay {
    fn new(mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>) -> RenderBufDisplay {
        RenderBufDisplay {
            mem
        }
    }
}

impl chip8::display::Display for RenderBufDisplay {
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

#[test]
fn it_works() {
    // TODO: random should not work.

    let p = std::env::current_dir().unwrap();
    println!("The current directory is {}", p.display());

    let mem = Rc::new(RefCell::new([false; DISPLAY_WIDTH * DISPLAY_HEIGHT]));

    let mut rom_file = File::open("tests/BRIX").unwrap();
    let mut buf = Vec::new();
    rom_file.read_to_end(&mut buf).unwrap();
    let mut vm = chip8::Vm::with_rom(&buf);

    use rand::{Rng, SeedableRng, StdRng};

    let seed: &[_] = &[2, 2, 8, 1];
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    for tick in 0..100 {
        for cycle in 0..4 {
            let mut display = RenderBufDisplay::new(mem.clone());
            let keyboard = [0u8; 16];
            vm.cycle(&mut Env {
                keyboard: keyboard,
                display: display,
                rng: rng
            });
        }
        vm.update_timers(1);
    }

    let final_mem = vm.memory.to_vec();
    let expected_mem: Vec<u8> = {
        match File::open("tests/expected.mem") {
            Ok(mut file) => {
                let mut expected_mem = Vec::new();
                file.read_to_end(&mut expected_mem);
                expected_mem
            }
            Err(e) => {
                // Assume file is not found.
                println!("Can't open file {}, creating new", e);
                let mut new_expected_file = File::create("tests/expected.mem").unwrap();
                new_expected_file.write_all(&final_mem).unwrap();
                return;
            }
        }
    };

    assert_eq!(final_mem, expected_mem);
}
