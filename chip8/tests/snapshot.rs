 #![feature(test)]

extern crate chip8;
extern crate rand;
extern crate test;

use test::{TestDesc, TestDescAndFn, DynTestName, TestFn, Options};

use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::env;

use chip8::{Vm, Env};
use rand::{SeedableRng, StdRng};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct RenderBufDisplay {
    mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>,
}

impl RenderBufDisplay {
    fn new(mem: Rc<RefCell<[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]>>) -> RenderBufDisplay {
        RenderBufDisplay { mem }
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
fn main() {
    let args: Vec<_> = env::args().collect();
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("{:?}", src_dir);

    let mut tests: Vec<TestDescAndFn> = Vec::new();

    let path = Path::new("tests/roms");
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            if entry.metadata().unwrap().is_dir() {
                continue;
            }

            let filename = entry.file_name().into_string().unwrap();

            let test_desc_and_fn = TestDescAndFn {
                desc: TestDesc {
                    name: DynTestName(filename.clone()),
                    ignore: false,
                    should_panic: test::ShouldPanic::No,
                },
                testfn: TestFn::DynTestFn(Box::new(move |_| { test_rom_snapshot(&filename); })),
            };
            tests.push(test_desc_and_fn);
        }
    }

    test::test_main(&args, tests, Options::new());
}

fn test_rom_snapshot(rom_name: &str) {
    let mem = Rc::new(RefCell::new([false; DISPLAY_WIDTH * DISPLAY_HEIGHT]));

    let rom_filename = format!("tests/roms/{}", rom_name);
    println!("{}", rom_filename);
    let mut rom_file = File::open(rom_filename).expect("rom file should exists");
    let mut buf = Vec::new();
    rom_file.read_to_end(&mut buf).unwrap();
    let mut vm = Vm::with_rom(&buf);

    let seed: &[_] = &[2, 2, 8, 1];
    let rng: StdRng = SeedableRng::from_seed(seed);

    for _ in 0..10000 {
        for _ in 0..4 {
            let display = RenderBufDisplay::new(mem.clone());
            let keyboard = [0u8; 16];
            vm.cycle(&mut Env {
                keyboard: keyboard,
                display: display,
                rng: rng,
            }).unwrap();
        }
        vm.update_timers(1);
    }

    let final_mem = vm.memory.to_vec();

    let expected_mem_filename = format!("tests/expected/{}.mem", rom_name);
    let expected_mem: Vec<u8> = {
        match File::open(&expected_mem_filename) {
            Ok(mut file) => {
                let mut expected_mem = Vec::new();
                file.read_to_end(&mut expected_mem).unwrap();
                expected_mem
            }
            Err(e) => {
                // Assume file is not found.
                println!("Can't open file {}, creating new", e);
                let mut new_expected_file = File::create(&expected_mem_filename).unwrap();
                new_expected_file.write_all(&final_mem).unwrap();
                return;
            }
        }
    };

    assert_eq!(final_mem, expected_mem);
}
