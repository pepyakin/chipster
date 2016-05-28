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
    
    let title = "Hello Piston! (press any key to enter inner loop)";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.5, 1.0, 0.5, 1.0], g);
            rectangle([1.0, 0.0, 0.0, 1.0], [50.0, 50.0, 100.0, 100.0], c.transform, g);
        });
    }
}
