
#[cfg(target_os = "emscripten")]
mod emscripten;

use super::Result;
use std::{thread, process, time};

pub fn start_loop<F>(f: F) -> !
where
    F: FnMut() -> Result<Step>,
{
        #[cfg(target_os = "emscripten")]
    let looper = emscripten::EmscriptenLooper;

        #[cfg(not(target_os = "emscripten"))]
    let looper = BlockingLooper;

    looper.start_loop(f)
}

pub enum Step {
    Cont,
    Done,
}

trait Looper {
    fn start_loop<F>(self, f: F) -> !
    where
        F: FnMut() -> Result<Step>;
}

struct BlockingLooper;

impl Looper for BlockingLooper {
    fn start_loop<F>(self, mut f: F) -> !
    where
        F: FnMut() -> Result<Step>,
    {
        let frame_interval = time::Duration::from_millis(16);
        loop {
            let frame_start = time::Instant::now();

            match f() {
                Ok(Step::Cont) => {
                    if let Some(delay) = frame_interval.checked_sub(frame_start.elapsed()) { 
                        thread::sleep(delay) 
                    }
                }
                Ok(Step::Done) => {
                    process::exit(0);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
