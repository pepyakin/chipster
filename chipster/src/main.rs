extern crate portaudio;
extern crate piston_window;
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate chip8;
extern crate rand;

use piston_window::*;
use piston_window::Input::*;

mod beep;
mod render;

use std::path::Path;
use std::io;
use std::fs::File;
use render::{RenderBuf, RenderBufDisplay};
use chip8::{Vm, Env};

error_chain! {
    links {
        Chip8Error(chip8::Error, chip8::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        PortAudio(portaudio::Error);
    }
}

struct CommandArgs {
    rom_file_name: String,
    cycles_per_second: u32, // default: 500
    pixel_decay_time: f32,
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
            .arg(Arg::with_name("pixel decay time")
                     .short("d")
                     .long("pixel-decay-time")
                     .value_name("pixel_decay_time")
                     .help("How many seconds takes for pixel from lit to non-lit")
                     .takes_value(true))
            .get_matches();

        let cycles_per_second = matches
            .value_of("cycles per second")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(500);

        let pixel_decay_time = matches
            .value_of("pixel decay time")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.3);

        CommandArgs {
            rom_file_name: matches.value_of("ROM_FILE").unwrap().to_string(),
            cycles_per_second,
            pixel_decay_time,
        }
    }
}

fn build_window() -> PistonWindow {
    let title = "Chip8";
    let mut window: PistonWindow =
        WindowSettings::new(title, [640, 320])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_swap_buffers(false);
    window.set_max_fps(60);

    window
}

quick_main!(|| -> Result<()> {
    let args = CommandArgs::parse();

    let mut beeper_factory = beep::BeeperFactory::new()?;
    beeper_factory
        .with_beeper(|mut beeper| {
                         let app = App::new(&args, &mut beeper)?;
                         let piston_window = build_window();
                         app.run(piston_window)?;
                         Ok(())
                     })?;

    Ok(())
});

struct App<'a, 'b: 'a> {
    command_args: &'a CommandArgs,
    render_buf: RenderBuf,
    vm: Vm,
    passed_dt: f64,
    paused: bool,
    beeper: &'a mut beep::Beeper<'b>,
    keyboard: [u8; 16],
}

fn read_rom<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    use std::io::Read;

    let mut rom_file = File::open(path)?;
    let mut rom_buffer = Vec::new();
    rom_file.read_to_end(&mut rom_buffer)?;
    Ok(rom_buffer)
}

impl<'a, 'b: 'a, 'c> App<'a, 'b> {
    fn new(command_args: &'a CommandArgs, beeper: &'a mut beep::Beeper<'b>) -> Result<App<'a, 'b>> {
        let render_buf = RenderBuf::new(command_args.pixel_decay_time);

        let rom_data = read_rom(&command_args.rom_file_name)?;
        let vm = Vm::with_rom(&rom_data);

        Ok(App {
               command_args: command_args,
               render_buf: render_buf,
               vm: vm,
               passed_dt: 0f64,
               paused: false,
               beeper: beeper,
               keyboard: [0; 16],
           })
    }

    fn run(mut self, mut window: PistonWindow) -> Result<()> {
        while let Some(e) = window.next() {
            if Release(Button::Keyboard(Key::Space)) == e {
                self.paused = !self.paused;
            }

            if self.paused {
                continue;
            }

            match e {
                Press(button) => {
                    if let Some(pressed_key) = map_keycode(button) {
                        self.keyboard[pressed_key] = 1;
                    }
                }
                Release(button) => {
                    if let Some(released_key) = map_keycode(button) {
                        self.keyboard[released_key] = 0;
                    }
                }
                Update(update_args) => {
                    self.update(update_args)?;
                }
                Render(render_args) => {
                    self.render(&e, render_args, &mut window);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn update(&mut self, args: UpdateArgs) -> Result<()> {
        const TIMER_TICK_DURATION: f64 = 1.0 / 60.0;

        // See "Secrets of emulation" chapter
        // in https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets

        // TODO: Test for low values.
        let dt = args.dt;
        let cycles_to_perform = (dt * self.command_args.cycles_per_second as f64).floor() as usize;
        let dt_per_cycle = dt / cycles_to_perform as f64;
        // println!("dt={}, dt_per_cycle={}, cycles_to_perform={}",
        //          dt,
        //          dt_per_cycle,
        //          cycles_to_perform);

        for _cycle_number in 0..cycles_to_perform {
            // println!("{}/{}", _cycle_number, cycles_to_perform);

            let display = self.render_buf.display();
            self.vm.cycle(&mut Env {
                display,
                rng: rand::thread_rng(),
                keyboard: self.keyboard.clone(),
            })?;

            self.passed_dt += dt_per_cycle;
            if self.passed_dt > TIMER_TICK_DURATION {
                let ticks_passed = (self.passed_dt / TIMER_TICK_DURATION) as u8;
                self.passed_dt -= ticks_passed as f64 * TIMER_TICK_DURATION;

                // println!("updating {} ticks", ticks_passed);
                self.vm.update_timers(ticks_passed);
            }
        }

        self.beeper.set_beeping(self.vm.is_beeping())?;
        self.render_buf.update(args.dt as f32);

        Ok(())
    }

    fn render(&mut self, e: &Input, args: RenderArgs, window: &mut PistonWindow) {
        window.draw_2d(e, |c, g| {
            let clear_color = [0.98, 0.95, 0.86, 1.0];

            clear(clear_color, g);

            let w = args.width as f64 / 64.0;
            let h = args.height as f64 / 32.0;

            for y in 0..32 {
                for x in 0..64 {
                    let dx = x as f64 * w;
                    let dy = y as f64 * h;

                    match self.render_buf.get_intensity(x, y) {
                        intensity if intensity > 0.0 => {
                            let rect = [dx, dy, w, h];
                            let solid_color = [0.02, 0.12, 0.15, intensity];

                            rectangle(solid_color, rect, c.transform, g);
                        }
                        _ => {}
                    }
                }
            }
        });
        Window::swap_buffers(window);
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
