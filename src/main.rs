extern crate coreaudio_rs as coreaudio;
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
use std::io;

use basic_types::{BLANK_BUFFER, BUFFER_SIZE, AudioBuffer, Input};
use multi::Multi;

// TODO: figure out how to retrieve this from the system
const SAMPLE_RATE: u32 = 44_100;

fn main() {
    let (mut multi, mut midi_conn, audio_connection) = Multi::new(32, 4, SAMPLE_RATE);

    // accept midi input
    let midi_in = MidiWrap::new("organn", "input", move |midi| { midi_conn.midi_message(&midi); });

    // audio buffer and position
    let mut buf: AudioBuffer = BLANK_BUFFER;
    let mut pos = BUFFER_SIZE;      // start at the end to trigger fetching audio

    // Construct an Output audio unit.
    let audio_unit = AudioUnit::new(Type::Output, SubType::HalOutput)
        .render_callback(Box::new(move |buffer, num_frames| {
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
    io::stdin().read_line(&mut wait_str).unwrap();

    audio_unit.close();
    drop(midi_in);
}
