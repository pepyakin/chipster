use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice};
use sdl2::AudioSubsystem;
use sdl2::Sdl;
use std::time::Duration;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase >= 0.0 && self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Beeper {
    beeping: bool,
    device: AudioDevice<SquareWave>,
}

impl Beeper {
    pub fn new(audio_subsystem: &AudioSubsystem) -> ::Result<Beeper> {
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1), // mono
            samples: None, // default sample size
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })?;

        Ok(Beeper {
            beeping: false,
            device,
        })
    }

    pub fn set_beeping(&mut self, beeping: bool) -> ::Result<()> {
        if self.beeping != beeping {
            self.beeping = beeping;
            if beeping {
                self.device.resume();
            } else {
                self.device.pause();
            }
        }
        Ok(())
    }
}
