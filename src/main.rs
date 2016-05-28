extern crate rand;
extern crate portaudio;

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
    chip8.execute(|chip| {

    });
}
