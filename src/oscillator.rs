use num::Float;
use num::traits::Num;
use std::f32::consts::PI;

const SHIFT: f32 = (1 << 16) as f32;

struct PhaseIter {
    pos: u32,
    loop_len: u32,
    outscale: f32,
}

impl PhaseIter {
    fn calc_loop(freq: f32, sample_rate: u32) -> u32 {
        (((sample_rate as f32) * SHIFT) / (freq)) as u32
    }

    fn new(freq: f32, sample_rate: u32, outscale: f32) -> Self {
        PhaseIter {
            pos: 0,
            loop_len: Self::calc_loop(freq, sample_rate),
            outscale: outscale
        }
    }

    fn set_freq(&mut self, freq: f32, sample_rate: u32) {
        // try to avoid jumps in output by remapping the current position to the new loop len
        let new_len = Self::calc_loop(freq, sample_rate);
        let new_pos = (((self.pos as u64) * (new_len as u64)) / (self.loop_len as u64)) as u32;
        self.loop_len = new_len;
        self.pos = new_pos;
    }
}

impl Iterator for PhaseIter {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let pos = self.pos;
        self.pos = (self.pos + (1 << 16)) % self.loop_len;
        Some(((pos as f32) * self.outscale) / (self.loop_len as f32))
    }
}

pub struct Oscillator {
    phase: PhaseIter
}

impl Oscillator {
    pub fn new(freq: f32, sample_rate: u32) -> Self {
        Oscillator {
            phase: PhaseIter::new(freq, sample_rate, PI * 2.0)
        }
    }

    pub fn set_freq(&mut self, freq: f32, sample_rate: u32) {
        println!("freq: {}", freq);
        self.phase.set_freq(freq, sample_rate);
    }

    pub fn get_sample(&mut self) -> f32 {
        self.phase.next().unwrap().sin()
    }
}
