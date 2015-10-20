extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate pitch_calc;
extern crate midi;
extern crate midi_wrap;

mod basic_types;
mod oscillator;
mod mixer;
mod env;
mod voice;
mod multi;

use midi_wrap::MidiWrap;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use std::sync::mpsc;
use midi::Message;
use midi::Channel;
use std::io;

use basic_types::{BUFFER_SIZE, AudioBuffer, Input};
use multi::Multi;

// TODO: figure out how to retrieve this from the system
const SAMPLE_RATE: u32 = 44_100;

fn main() {
    let (mut multi, audio_connection) = Multi::new(32, SAMPLE_RATE);

    // create channel for updates
    let (send, recv) = mpsc::channel();

    // accept midi input
    let midi_in = MidiWrap::new("organn", "input", |midi| { send.send(midi); });

    // audio buffer and position
    let mut buf: AudioBuffer = [0.0; BUFFER_SIZE];
    let mut pos = BUFFER_SIZE;      // start at the end to trigger fetching audio

    // Construct an Output audio unit.
    let audio_unit = AudioUnit::new(Type::Output, SubType::HalOutput)
        .render_callback(Box::new(move |buffer, num_frames| {
            // process messages for this thread
            loop {
                let message = recv.try_recv();
                match message {
                    Ok(midi_message) => {
                        multi.midi_message(&midi_message);
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
                if pos >= buf.len() {
                    multi.run();
                    buf = audio_connection.get_audio();
                    pos = 0;
                }
                let sample = buf[pos];
                pos += 1;
                for channel in buffer.iter_mut() {
                    channel[frame] = sample;
                }
            }
            Ok(())
        }))
        .start()
        .unwrap();

    let mut wait_str = String::new();
    io::stdin().read_line(&mut wait_str);

    audio_unit.close();
    drop(midi_in);
}
