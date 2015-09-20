//! A basic output stream example, using an Output AudioUnit to generate a sine wave.

extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate pitch_calc;

mod oscillator;
mod mixer;
mod produce_audio;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use pitch_calc::Step;
use std::sync::mpsc;
use std::cell::RefCell;
use std::rc::Rc;
use produce_audio::{ProduceAudioMut, ProduceAudio};

struct Voice {
    oscillators: Vec<Rc<RefCell<oscillator::Oscillator>>>,
    mixer: mixer::Mixer<Rc<RefCell<oscillator::Oscillator>>>
}

impl Voice {
    fn new() -> Self {
        // create the parts of the signal chain
        let oscillators: Vec<_> = (0..8)
            .map(|mult| Rc::new(RefCell::new(oscillator::Oscillator::new(440.0 * ((mult + 1) as f32), 44_100))))
            .collect();


        let borrowed_oscs: Vec<_> = oscillators.iter().map(|ref_osc| ref_osc.clone()).collect();
        let mixer = mixer::Mixer::new(borrowed_oscs, vec![0.0; oscillators.len()]);

        // get all the components into a struct so they share lifetime
        let mut voice = Voice {
            oscillators: oscillators,
            mixer: mixer
        };

        // link the components together
        voice.mixer.set_level(0, 0.5);
        voice.mixer.set_level(1, 0.3);
        voice.mixer.set_level(2, 0.05);
        voice.mixer.set_level(3, 0.2);
        voice.mixer.set_level(4, 0.05);
        voice.mixer.set_level(5, 0.2);
        voice.mixer.set_level(6, 0.05);
        voice.mixer.set_level(7, 0.05);

        voice
    }

    fn set_pitch(&mut self, pitch: f32) {
        let freq = Step(pitch).to_hz().hz();
        for (num, ref_osc) in self.oscillators.iter_mut().enumerate() {
            let mut oscillator = (**ref_osc).borrow_mut();
            oscillator.set_freq(freq * ((num + 1) as f32), 44_100);
        }
    }
}

impl ProduceAudioMut for Voice {
    fn next_sample(&mut self) -> f32 {
        self.mixer.next_sample()
    }
}

fn main() {
    let mut voice = Voice::new();

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
                        voice.set_pitch(pitch);
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
                let sample = voice.next_sample();
                for channel in buffer.iter_mut() {
                    channel[frame] = sample;
                }
            }
            Ok(())
        }))
        .start()
        .unwrap();

    let start_note = 33;
    for i in (0..6) {
        let note = start_note + (i * 12);
        send.send(note as f32).unwrap();
        ::std::thread::sleep_ms(3000);
    }

    audio_unit.close();
}
