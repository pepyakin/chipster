use portaudio;
use portaudio::stream;

const CHANNELS: i32 = 2;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 200;

pub struct BeeperFactory {
    portaudio: portaudio::PortAudio,
}

impl BeeperFactory {
    pub fn new() -> BeeperFactory {
        let p = portaudio::PortAudio::new().unwrap();
        BeeperFactory { portaudio: p }
    }

    pub fn create_beeper<'a>(&'a mut self) -> Beeper<'a> {
        Beeper::new(&mut self.portaudio)
    }
}

pub struct Beeper<'a> {
    stream: stream::Stream<'a, stream::NonBlocking, stream::Output<f32>>,
    started: bool,
}

impl<'a> Beeper<'a> {
    fn new(p: &'a mut portaudio::PortAudio) -> Beeper<'a> {
        use std::f64::consts::PI;

        let settings = p.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER)
            .unwrap();

        let mut sine = [0.0; TABLE_SIZE];
        for i in 0..TABLE_SIZE {
            sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
        }
        let mut left_phase = 0;
        let mut right_phase = 0;

        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, frames, .. }| {
            let mut idx = 0;
            for _ in 0..frames {
                buffer[idx] = sine[left_phase];
                buffer[idx + 1] = sine[right_phase];
                left_phase += 1;
                if left_phase >= TABLE_SIZE {
                    left_phase -= TABLE_SIZE;
                }
                right_phase += 3;
                if right_phase >= TABLE_SIZE {
                    right_phase -= TABLE_SIZE;
                }
                idx += 2;
            }
            portaudio::Continue
        };

        let stream = p.open_non_blocking_stream(settings, callback).unwrap();
        Beeper {
            stream: stream,
            started: false,
        }
    }

    pub fn set_started(&mut self, started: bool) {
        if self.started != started {
            self.started = started;
            if started {
                println!("starting stream");
                self.stream.start();
            } else {
                println!("stoping stream");
                self.stream.stop();
            }
        }
    }
}

impl<'a> Drop for Beeper<'a> {
    fn drop(&mut self) {
        if self.started {
            self.stream.stop();
        }
        self.stream.close();
    }
}
