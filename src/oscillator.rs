use num::Float;
use std::f32::consts::PI;
use basic_types::{ProduceAudioMut, Output, BUFFER_SIZE, AudioBuffer};

const SHIFT: f32 = (1 << 16) as f32;

struct PhaseIter {
    pos: u32,
    loop_len: u32,
    outscale: f32,
    sample_rate: u32
}

impl PhaseIter {
    fn calc_loop(&self, freq: f32) -> u32 {
        (((self.sample_rate as f32) * SHIFT) / (freq)) as u32
    }

    fn new(sample_rate: u32, outscale: f32) -> Self {
        PhaseIter {
            pos: 0,
            loop_len: 0,
            outscale: outscale,
            sample_rate: sample_rate
        }
    }

    fn set_freq(&mut self, freq: f32) {
        if freq > 0.0 && freq < (self.sample_rate as f32 / 2.0) {
            // try to avoid jumps in output by remapping the current position to the new loop len
            let new_len = self.calc_loop(freq);
            let new_pos = if self.loop_len > 0 {
                    (((self.pos as u64) * (new_len as u64)) / (self.loop_len as u64)) as u32
                }
                else {
                    0
                };
            self.loop_len = new_len;
            self.pos = new_pos;
        }
        else {
            self.loop_len = 0;
            self.pos = 0;
        }
    }
}

impl Iterator for PhaseIter {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.loop_len > 0 {
            let pos = self.pos;
            self.pos = (self.pos + (1 << 16)) % self.loop_len;
            Some(((pos as f32) * self.outscale) / (self.loop_len as f32))
        }
        else {
            Some(0.0)
        }
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
        let mut samples: AudioBuffer = [0.0; BUFFER_SIZE];
        for sample in samples.iter_mut() {
            *sample = self.phase.next().unwrap().sin();
        }
        self.output.supply_audio(samples);
    }
}
