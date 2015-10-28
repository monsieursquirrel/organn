use std::f32::consts::PI;
use basic_types::{Output, BLANK_BUFFER, AudioBuffer};

struct PhaseIter {
    pos: u32,
    increment: u32,
    sample_rate: u32,
    outscale: f32,
}

impl PhaseIter {
    fn new(sample_rate: u32, outscale: f32) -> Self {
        PhaseIter {
            pos: 0,
            increment: 0,
            outscale: outscale,
            sample_rate: sample_rate
        }
    }

    fn set_freq(&mut self, freq: f32) {
        // stay away from the nyquist limit!
        if freq > 0.0 && freq < (self.sample_rate as f32 / 2.1) {
            self.increment = ((freq * (2 as f32).powi(32)) / self.sample_rate as f32) as u32;
        }
        else {
            self.increment = 0;
            self.pos = 0;
        }
    }
}

impl Iterator for PhaseIter {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let pos = self.pos;
        self.pos = self.pos.wrapping_add(self.increment);
        Some(((pos as f32) * self.outscale) / (2 as f32).powi(32))
    }
}

pub struct Oscillator<T> where T: Output {
    phase: PhaseIter,
    output: T
}

impl<T> Oscillator<T> where T: Output {
    pub fn new(sample_rate: u32, output: T) -> Self {
        Oscillator {
            phase: PhaseIter::new(sample_rate, PI * 2.0),
            output: output
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.phase.set_freq(freq);
    }

    pub fn run(&mut self) {
        let mut samples: AudioBuffer = BLANK_BUFFER;
        for sample in samples.iter_mut() {
            *sample = self.phase.next().unwrap().sin();
        }
        self.output.supply_audio(samples);
    }
}
