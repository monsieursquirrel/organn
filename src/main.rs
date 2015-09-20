//! A basic output stream example, using an Output AudioUnit to generate a sine wave.

extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate pitch_calc;
extern crate midi;

mod produce_audio;
mod oscillator;
mod mixer;
mod env;
mod voice;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use std::sync::mpsc;
use midi::Message;
use midi::Channel;

use produce_audio::{ProduceAudioMut, ProduceAudio};
use voice::Voice;

// TODO: figure out how to retrieve this from the system
const SAMPLE_RATE: u32 = 44_100;

fn main() {
    let mut voice = Voice::new(SAMPLE_RATE);

    // create channel for updates
    let (send, recv) = mpsc::channel();

    // Construct an Output audio unit.
    let audio_unit = AudioUnit::new(Type::Output, SubType::HalOutput)
        .render_callback(Box::new(move |buffer, num_frames| {
            // process messages for this thread
            loop {
                let message = recv.try_recv();
                match message {
                    Ok(midi_message) => {
                        voice.midi_message(&midi_message);
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
        send.send(Message::NoteOn(Channel::Ch1, note, 100)).unwrap();
        ::std::thread::sleep_ms(3000);
    }

    audio_unit.close();
}
