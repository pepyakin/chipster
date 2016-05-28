extern crate rand;
extern crate portaudio;
extern crate piston_window;

use piston_window::*;

mod audio;
mod chip8;

use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::env;

fn read_bin<P: AsRef<Path>>(path: P) -> Box<[u8]> {
    let mut bin_file = File::open(path).unwrap();
    let mut bin_buffer = Vec::new();
    bin_file.read_to_end(&mut bin_buffer);
    bin_buffer.into_boxed_slice()
}

fn main() {
    let bin_file_name = env::args().nth(1).unwrap();
    let bin_data = read_bin(bin_file_name);

    let mut portaudio_holder = audio::PortAudioHolder::new();
    let mut beeper = portaudio_holder.create_beeper();

    let mut chip8 = chip8::Chip8::with_bin(bin_data);

    let title = "Chip8";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    let mut left_from_last_update: f64 = 0.0;
    while let Some(e) = window.next() {
        if let Some(args) = e.update_args() {
            // See "Secrets of emulation" chapter
            // in https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets

            let dt = args.dt + left_from_last_update;
            let cycles_per_second = 500u32;
            let cycles_to_perform = (dt * cycles_per_second as f64).floor() as usize;
            let dt_per_cycle = dt / cycles_to_perform as f64;
            println!("left_from_last_update={}, dt={}, dt_per_cycle={}, cycles_to_perform={}",
                     left_from_last_update,
                     dt,
                     dt_per_cycle,
                     cycles_to_perform);

            for cycle_number in 0..cycles_to_perform {
                println!("{}/{}", cycle_number, cycles_to_perform);

                chip8.cycle();
                chip8.update_timers(dt_per_cycle);
            }

            beeper.set_started(chip8.is_beeping());
        }

        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g| {
                clear([0.5, 1.0, 0.5, 1.0], g);


            });
        }
    }
}
