extern crate portaudio;
extern crate piston_window;
extern crate clap;

extern crate vm;

use piston_window::*;

mod audio;

use std::path::Path;
use std::io;
use std::fs::File;
use vm::Chip8;

struct CommandArgs {
    bin_file_name: String,
    cycles_per_second: u32, // default: 500
}

impl CommandArgs {
    fn parse() -> CommandArgs {
        use clap::{Arg, App};

        let matches = App::new("chip8 emulator")
            .arg(Arg::with_name("ROM_FILE")
                .help("rom file to load")
                .required(true))
            .arg(Arg::with_name("cycles per second")
                .short("c")
                .long("cycles-per-sec")
                .value_name("cycles_per_second")
                .help("How many Chip8 cycles should be executed per second. Values between \
                       500-1000 should be fine.")
                .takes_value(true))
            .get_matches();

        let cps = matches.value_of("cycles per second")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap();

        CommandArgs {
            bin_file_name: matches.value_of("ROM_FILE").unwrap().to_string(),
            cycles_per_second: cps,
        }
    }
}

fn build_window() -> PistonWindow {
    let title = "Chip8";
    let mut window: PistonWindow = WindowSettings::new(title, [640, 320])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_swap_buffers(false);
    window.set_max_fps(60);

    window
}

fn main() {
    let args = CommandArgs::parse();
    let mut beeper_factory = audio::BeeperFactory::new();
    let app = App::new(args, beeper_factory.create_beeper());
    let piston_window = build_window();

    app.run(piston_window);
}

struct App<'a> {
    command_args: CommandArgs,
    chip8: Chip8,
    paused: bool,
    beeper: audio::Beeper<'a>,
}

fn read_rom<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, io::Error> {
    use std::io::Read;

    let mut bin_file = File::open(path)?;
    let mut bin_buffer = Vec::new();
    bin_file.read_to_end(&mut bin_buffer)?;
    Ok(bin_buffer.into_boxed_slice())
}

fn prepare_chip8_vm(rom_file_name: &str) -> Chip8 {
    let rom_data = read_rom(rom_file_name).expect("failed to read rom");
    Chip8::with_bin(rom_data)
}

impl<'a> App<'a> {
    fn new(command_args: CommandArgs, beeper: audio::Beeper<'a>) -> App<'a> {
        let chip8 = prepare_chip8_vm(&command_args.bin_file_name);

        App {
            command_args: command_args,
            chip8: chip8,
            paused: false,
            beeper: beeper,
        }
    }

    fn run(mut self, mut window: PistonWindow) {
        while let Some(e) = window.next() {
            if Some(Button::Keyboard(Key::Space)) == e.release_args() {
                self.paused = !self.paused;
            }

            if self.paused {
                continue;
            }

            self.handle_input(&e);
            self.update(&e);
            self.render(&e, &mut window);
        }
    }

    fn handle_input(&mut self, e: &Event) {
        if let Some(button) = e.press_args() {
            if let Some(pressed_key) = map_keycode(button) {
                self.chip8.keyboard[pressed_key] = 1;
            }
        }
        if let Some(button) = e.release_args() {
            if let Some(released_key) = map_keycode(button) {
                self.chip8.keyboard[released_key] = 0;
            }
        }
    }

    fn update(&mut self, e: &Event) {
        if let Some(args) = e.update_args() {
            // See "Secrets of emulation" chapter
            // in https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets

            // TODO: Test for low values.
            let dt = args.dt;
            let cycles_to_perform =
                (dt * self.command_args.cycles_per_second as f64).floor() as usize;
            let dt_per_cycle = dt / cycles_to_perform as f64;
            println!("dt={}, dt_per_cycle={}, cycles_to_perform={}",
                     dt,
                     dt_per_cycle,
                     cycles_to_perform);

            for _cycle_number in 0..cycles_to_perform {
                // println!("{}/{}", cycle_number, cycles_to_perform);

                self.chip8.cycle();
                self.chip8.update_timers(dt_per_cycle);
            }

            self.beeper.set_started(self.chip8.is_beeping());
        }
    }

    fn render(&mut self, e: &Event, window: &mut PistonWindow) {
        if let Some(args) = e.render_args() {
            window.draw_2d(e, |c, g| {
                let clear_color = [0.98, 0.95, 0.86, 1.0];
                let solid_color = [0.02, 0.12, 0.15, 1.0];

                clear(clear_color, g);

                let w = args.width as f64 / 64.0;
                let h = args.height as f64 / 32.0;

                for y in 0..32 {
                    for x in 0..64 {
                        let dx = x as f64 * w;
                        let dy = y as f64 * h;

                        if self.chip8.display.get(x, y) != 0 {
                            // let rect = [dx + w * 0.1, dy + h * 0.1, w * 0.9, h * 0.9];
                            let rect = [dx, dy, w, h];

                            rectangle(solid_color, rect, c.transform, g);
                        }
                    }
                }
            });
            Window::swap_buffers(window);
        }
    }
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
        match k {
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
        }
    } else {
        None
    }
}
