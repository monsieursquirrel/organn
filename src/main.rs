//! A basic output stream example, using an Output AudioUnit to generate a sine wave.

extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate itertools;
extern crate pitch_calc;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use pitch_calc::{Step, Hz};
use num::Float;
use num::traits::Num;
use std::f32::consts::PI;
use itertools::Zip;
use std::sync::mpsc;

mod oscillator;

fn main() {

    let freq = Step(57.0).to_hz().hz();
    // generate harmonics
    let h0 = oscillator::Oscillator::new(freq * 1.0, 44_100);
    let h1 = oscillator::Oscillator::new(freq * 2.0, 44_100);
    let h2 = oscillator::Oscillator::new(freq * 3.0, 44_100);
    let h3 = oscillator::Oscillator::new(freq * 4.0, 44_100);
    let h4 = oscillator::Oscillator::new(freq * 5.0, 44_100);
    let h5 = oscillator::Oscillator::new(freq * 6.0, 44_100);
    let h6 = oscillator::Oscillator::new(freq * 7.0, 44_100);
    let h7 = oscillator::Oscillator::new(freq * 8.0, 44_100);

    // mix them
    let mut mixed = Zip::new((h0, h1, h2, h3, h4, h5, h6, h7))
        .map(|(s0, s1, s2, s3, s4, s5, s6, s7)|
            (s0 * 0.5) +
            (s1 * 0.25) +
            (s2 * 0.125) +
            (s3 * 0.0625) +
            (s4 * 0.0625) +
            (s5 * 0.0625) +
            (s6 * 0.0625) +
            (s7 * 0.0625)
            );

    // create channel for updates
    let (send, recv) = mpsc::channel();

    // Construct an Output audio unit.
    let audio_unit = AudioUnit::new(Type::Output, SubType::HalOutput)
        .render_callback(Box::new(move |buffer, num_frames| {
            // process messages for this thread
            loop {
                let message = recv.try_recv();
                match message {
                    Ok(pitch) => {
                        // update pitch

                    }
                    Err(mpsc::TryRecvError::Empty) => {
                        break;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        // get out of here
                        return Err("EOF".to_owned());
                    }
                }
            }

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
