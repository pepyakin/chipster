extern crate rand;
extern crate portaudio;
extern crate piston_window;
extern crate clap;

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

fn map_keycode(k: Button) -> Option<usize> {
    // Classical layout, see http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.3
    // +---+---+---+---+
    // | 1 | 2 | 3 | C |
    // +---+---+---+---+
    // | 4 | 5 | 6 | D |
    // +---+---+---+---+
    // | 7 | 8 | 9 | E |
    // +---+---+---+---+
    // | A | 0 | B | F |
    // +---+---+---+---+

    if let Button::Keyboard(k) = k {
        return match k {
            Key::D1 => Some(0x1),
            Key::D2 => Some(0x2),
            Key::D3 => Some(0x3),
            Key::D4 => Some(0xC),

            Key::Q => Some(0x4),
            Key::W => Some(0x5),
            Key::E => Some(0x6),
            Key::R => Some(0xD),

            Key::A => Some(0x7),
            Key::S => Some(0x8),
            Key::D => Some(0x9),
            Key::F => Some(0xE),

            Key::Z => Some(0xA),
            Key::X => Some(0x0),
            Key::C => Some(0xB),
            Key::V => Some(0xF),
            _ => None,
        };
    }
    return None;
}

struct CommandArgs {
    bin_file_name: String,
    cycles_per_second: u32, // default: 500
}

impl CommandArgs {
    fn parse() -> CommandArgs {
        use clap::{Arg, App};

        let matches = App::new("chip8 emulator")
            .arg(Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true))
            .arg(Arg::with_name("cycles per second")
                .short("c")
                .long("cycles-per-sec")
                .value_name("cycles_per_second")
                .help("Something between 500-1000")
                .takes_value(true))
            .get_matches();
            
            
        println!("{:#?}", matches);
        let cps = matches.value_of("cycles per second")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap();

        CommandArgs {
            bin_file_name: matches.value_of("INPUT").unwrap().to_string(),
            cycles_per_second: cps,
        }
    }
}

fn main() {
    let args = CommandArgs::parse();
    run(args);
}

fn run(command_args: CommandArgs) {
    let bin_file_name = command_args.bin_file_name;
    let bin_data = read_bin(bin_file_name);

    let mut portaudio_holder = audio::PortAudioHolder::new();
    let mut beeper = portaudio_holder.create_beeper();

    let mut chip8 = chip8::Chip8::with_bin(bin_data);

    let title = "Chip8";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 320])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_swap_buffers(false);
    window.set_max_fps(60);
    
    let mut paused = false;
    let mut left_from_last_update: f64 = 0.0;
    while let Some(e) = window.next() {
        if let Some(button) = e.press_args() {
            if let Some(pressed_key) = map_keycode(button) {
                chip8.keyboard[pressed_key] = 1;

                println!("key pressed {:?}", pressed_key);
                // println!("{:?}", chip8.keyboard);
            }
        }
        if let Some(button) = e.release_args() {
            if button == Button::Keyboard(Key::Space) {
                paused = !paused;
            }

            if let Some(released_key) = map_keycode(button) {
                chip8.keyboard[released_key] = 0;
                println!("key released {:?}", released_key);
                // println!("{:?}", chip8.keyboard);
            }
        }

        if paused {
            continue;
        }

        if let Some(args) = e.update_args() {
            // See "Secrets of emulation" chapter
            // in https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets

            // TODO: Test for low values.
            let dt = args.dt + left_from_last_update;
            let cycles_to_perform = (dt * command_args.cycles_per_second as f64).floor() as usize;
            let dt_per_cycle = dt / cycles_to_perform as f64;
            println!("left_from_last_update={}, dt={}, dt_per_cycle={}, cycles_to_perform={}",
                     left_from_last_update,
                     dt,
                     dt_per_cycle,
                     cycles_to_perform);

            for cycle_number in 0..cycles_to_perform {
                // println!("{}/{}", cycle_number, cycles_to_perform);

                chip8.cycle();
                chip8.update_timers(dt_per_cycle);
            }

            beeper.set_started(chip8.is_beeping());
        }

        if let Some(args) = e.render_args() {
            window.draw_2d(&e, |c, g| {
                clear([0.0, 0.0, 0.0, 1.0], g);

                let w = args.width as f64 / 64.0;
                let h = args.height as f64 / 32.0;

                for y in 0..32 {
                    for x in 0..64 {
                        let dx = x as f64 * w;
                        let dy = y as f64 * h;

                        if chip8.display.get(x, y) != 0 {
                            let rect = [dx + w * 0.1, dy + h * 0.1, w * 0.9, h * 0.9];

                            rectangle([1.0, 1.0, 1.0, 1.0], rect, c.transform, g);
                        }
                    }
                }
            });
            Window::swap_buffers(&mut window);
        }
    }
}
