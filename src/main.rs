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
mod mixer;

fn main() {

    let freq = Step(57.0).to_hz().hz();
    // generate harmonics
    let mut oscillators: Vec<_> = (0..8).map(|mult| oscillator::Oscillator::new(freq * ((mult + 1) as f32), 44_100)).collect();

    // mix them
    let mut mixer = mixer::Mixer::new(vec![0.0; oscillators.len()]);

    mixer.set_level(0, 0.5);
    mixer.set_level(1, 0.3);
    mixer.set_level(3, 0.2);
    mixer.set_level(4, 0.1);
    mixer.set_level(5, 0.1);
    mixer.set_level(6, 0.05);
    mixer.set_level(7, 0.05);

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
                        let freq = Step(pitch).to_hz().hz();
                        for (num, oscillator) in oscillators.iter_mut().enumerate() {
                            oscillator.set_freq(freq * ((num + 1) as f32), 44_100);
                        }
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
                let osc_outputs: Vec<_> = oscillators.iter_mut().map(|osc| osc.get_sample()).collect();
                let sample = mixer.mix(&osc_outputs);
                for channel in buffer.iter_mut() {
                    channel[frame] = sample;
                }
            }
            Ok(())
        }))
        .start()
        .unwrap();

    let mut note = 57;
    loop {
        ::std::thread::sleep_ms(3000);
        note += 1;
        send.send(note as f32);
    }

    audio_unit.close();

}
