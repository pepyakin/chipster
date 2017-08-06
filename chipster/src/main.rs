 #![feature(link_args)]
 #![feature(never_type)]

extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate chip8;
extern crate rand;
extern crate sdl2;

mod beep;
mod render;
mod looper;

use looper::Step;
use render::RenderBuf;

use chip8::{Vm, Env};

use std::path::Path;
use std::io;
use std::fs::File;

use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, BlendMode};
use sdl2::video::Window;

error_chain! {
    foreign_links {
        Chip8(chip8::Error);
        Io(io::Error);
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
            .arg(
                Arg::with_name("ROM_FILE")
                    .help("rom file to load")
                    .required(true),
            )
            .arg(
                Arg::with_name("cycles per second")
                    .short("c")
                    .long("cycles-per-sec")
                    .value_name("cycles_per_second")
                    .help(
                        "How many Chip8 cycles should be executed per second. Values between \
                       500-1000 should be fine.",
                    )
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("pixel decay time")
                    .short("d")
                    .long("pixel-decay-time")
                    .value_name("pixel_decay_time")
                    .help("How many seconds takes for pixel from lit to non-lit")
                    .takes_value(true),
            )
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

fn read_rom<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    use std::io::Read;

    let mut rom_file = File::open(path)?;
    let mut rom_buffer = Vec::new();
    rom_file.read_to_end(&mut rom_buffer)?;
    Ok(rom_buffer)
}

fn main() {
    use std::process::exit;
    match do_run() {
        Ok(_) => exit(0),
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    }
}

fn do_run() -> Result<()> {
    #[cfg(not(target_os = "emscripten"))]
    let args = CommandArgs::parse();

    #[cfg(target_os = "emscripten")]
    let args = CommandArgs {
        rom_file_name: "file.rom".to_string(),
        cycles_per_second: 15000,
        pixel_decay_time: 0.1,
    };

    let app = App::new(&args)?;
    app.run()?
}

struct App<'a> {
    command_args: &'a CommandArgs,
    render_buf: RenderBuf,
    vm: Vm,
    passed_dt: f64,
    paused: bool,
    keyboard: [u8; 16],
}

impl<'a> App<'a> {
    fn new(command_args: &'a CommandArgs) -> Result<App<'a>> {
        let render_buf = RenderBuf::new(command_args.pixel_decay_time);

        #[cfg(not(target_os = "emscripten"))]
        let rom_data = read_rom(&command_args.rom_file_name)?;

        #[cfg(target_os = "emscripten")]
        let rom_data = include_bytes!("../../stuff/f8z.ch8") as &[u8];

        let vm = Vm::with_rom(&rom_data);

        Ok(App {
            command_args: command_args,
            render_buf: render_buf,
            vm: vm,
            passed_dt: 0f64,
            paused: false,
            keyboard: [0; 16],
        })
    }

    fn run(mut self) -> Result<!> {
        let ctx = sdl2::init().unwrap();
        let video_ctx = ctx.video().unwrap();
        let window = video_ctx
            .window("chipster", 640, 320)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_blend_mode(BlendMode::Blend);

        let mut events = ctx.event_pump().unwrap();
        let mut timer = ctx.timer().unwrap();

        let audio = ctx.audio().unwrap();
        let mut beeper = beep::Beeper::new(&audio)?;

        let mut last_ticks = timer.ticks();

        let main_loop = || {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        return Ok(Step::Done);
                    }

                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        if let Some(pressed_key) = map_keycode(keycode) {
                            self.keyboard[pressed_key] = 1;
                        }
                    }
                    Event::KeyUp { keycode: Some(keycode), .. } => {
                        if let Some(pressed_key) = map_keycode(keycode) {
                            self.keyboard[pressed_key] = 0;
                        }
                    }
                    _ => {}
                }
            }

            let current_ticks = timer.ticks();
            let dt = (current_ticks - last_ticks) as f64 / 1000.0;
            self.update(dt)?;
            self.render(&mut canvas);
            last_ticks = current_ticks;

            beeper.set_beeping(self.vm.is_beeping())?;

            Ok(Step::Cont)
        };

        looper::start_loop(main_loop)
    }

    fn update(&mut self, dt: f64) -> Result<()> {
        const TIMER_TICK_DURATION: f64 = 1.0 / 60.0;

        // See "Secrets of emulation" chapter
        // in https://github.com/AfBu/haxe-chip-8-emulator/wiki/(Super)CHIP-8-Secrets

        // TODO: Test for low values.
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

        self.render_buf.update(dt as f32);

        Ok(())
    }

    fn render(&mut self, canvas: &mut Canvas<Window>) {
        let clear_color = Color::RGB(250, 242, 219);
        canvas.set_draw_color(clear_color);
        canvas.clear();

        let (w, h) = match canvas.window().size() {
            (win_width, win_height) => (
                (win_width as f64 / 64.0) as u32,
                (win_height as f64 / 32.0) as u32,
            ),
        };

        for y in 0..32 {
            for x in 0..64 {
                let dx = x as i32 * w as i32;
                let dy = y as i32 * h as i32;

                match self.render_buf.get_intensity(x, y) {
                    intensity if intensity > 0.0 => {
                        let solid_color = Color::RGBA(5, 31, 38, (intensity * 255.0) as u8);
                        canvas.set_draw_color(solid_color);

                        let rect = Rect::new(dx, dy, w, h);
                        let _ = canvas.fill_rect(rect);
                    }
                    _ => {}
                }
            }
        }

        canvas.present();
    }
}

fn map_keycode(k: Keycode) -> Option<usize> {
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

    match k {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
