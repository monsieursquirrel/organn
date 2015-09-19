//! A basic output stream example, using an Output AudioUnit to generate a sine wave.

extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate itertools;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use num::Float;
use num::traits::Num;
use std::f64::consts::PI;
use itertools::Zip;

const SHIFT: f64 = (1 << 16) as f64;

struct PhaseIter {
    current: u32,
    loop_len: u32,
    outscale: f64,
}

impl PhaseIter {
    fn new(freq: f64, sample_rate: u32, outscale: f64) -> Self {
        PhaseIter {
            current: 0,
            loop_len: (((sample_rate as f64) * SHIFT) / (freq)) as u32,
            outscale: outscale
        }
    }
}

impl Iterator for PhaseIter {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        let pos = self.current;
        self.current = (self.current + (1 << 16)) % self.loop_len;
        Some(((pos as f64) * self.outscale) / (self.loop_len as f64))
    }
}

fn main() {

    // generate harmonics
    let h0 = PhaseIter::new(220.0 * 1.0, 44_100, PI * 2.0).map(|phase| phase.sin() as f32);
    let h1 = PhaseIter::new(220.0 * 2.0, 44_100, PI * 2.0).map(|phase| phase.sin() as f32);
    let h2 = PhaseIter::new(220.0 * 3.0, 44_100, PI * 2.0).map(|phase| phase.sin() as f32);
    let h3 = PhaseIter::new(220.0 * 4.0, 44_100, PI * 2.0).map(|phase| phase.sin() as f32);

    // mix them
    let mut mixed = Zip::new((h0, h1, h2, h3))
        .map(|(s0, s1, s2, s3)| (s0 * 0.5) + (s1 * 0.25) + (s2 * 0.125) + (s3 * 0.0625));

    // Construct an Output audio unit.
    let audio_unit = AudioUnit::new(Type::Output, SubType::HalOutput)
        .render_callback(Box::new(move |buffer, num_frames| {
            for frame in (0..num_frames) {
                let sample = mixed.next().unwrap();
                for channel in buffer.iter_mut() {
                    channel[frame] = sample;
                }
            }
            Ok(())
        }))
        .start()
        .unwrap();

    loop {
        ::std::thread::sleep_ms(30000);
    }

    audio_unit.close();

}
