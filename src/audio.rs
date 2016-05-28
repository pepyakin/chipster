use portaudio;
use portaudio::stream;

const CHANNELS: i32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;

pub struct PortAudioHolder {
    portaudio: portaudio::PortAudio,
}

impl PortAudioHolder {
    pub fn new() -> PortAudioHolder {
        let p = portaudio::PortAudio::new().unwrap();
        PortAudioHolder { portaudio: p }
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
        let mut left_saw = 0.0;
        let mut right_saw = 0.0;

        let mut settings =
            p.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER).unwrap();
        settings.flags = portaudio::stream_flags::CLIP_OFF;

        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, frames, .. }| {
            let mut idx = 0;
            for _ in 0..frames {
                buffer[idx] = left_saw;
                buffer[idx] = right_saw;
                left_saw += 0.01;
                if left_saw >= 1.0 {
                    left_saw -= 2.0;
                }
                right_saw += 0.03;
                if right_saw >= 1.0 {
                    right_saw -= 2.0;
                }
                idx += 2;
            }
            portaudio::Continue
        };

        let mut stream = p.open_non_blocking_stream(settings, callback).unwrap();
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
