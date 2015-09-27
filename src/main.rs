extern crate coreaudio_rs as coreaudio;
extern crate num;
extern crate pitch_calc;
extern crate midi;

mod basic_types;
mod oscillator;
mod mixer;
mod env;
mod voice;
mod multi;

use coreaudio::audio_unit::{AudioUnit, Type, SubType};
use std::sync::mpsc;
use midi::Message;
use midi::Channel;

use basic_types::{ProduceAudioMut, ProduceAudio, BUFFER_SIZE, AudioBuffer, Input};
use multi::Multi;

// TODO: figure out how to retrieve this from the system
const SAMPLE_RATE: u32 = 44_100;

fn main() {
    let (mut multi, audio_connection) = Multi::new(32, SAMPLE_RATE);

    // create channel for updates
    let (send, recv) = mpsc::channel();

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
                if (pos >= buf.len()) {
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

    let start_note = 33;
    for i in (0..6) {
        let note = start_note + (i * 12);
        send.send(Message::NoteOn(Channel::Ch1, note, 100)).unwrap();
        ::std::thread::sleep_ms(3000);
        send.send(Message::NoteOff(Channel::Ch1, note, 100)).unwrap();
        ::std::thread::sleep_ms(1000);
    }

    let start_note = 45;
    for i in (0..7) {
        let note = start_note + (i * 2);
        send.send(Message::NoteOn(Channel::Ch1, note,      100)).unwrap();
        send.send(Message::NoteOn(Channel::Ch1, note + 4,  100)).unwrap();
        send.send(Message::NoteOn(Channel::Ch1, note + 7,  100)).unwrap();
        for _ in (0..6) {
            for j in (0..3) {
                let top_note = note + 12 + (j * 12);
                send.send(Message::NoteOn(Channel::Ch1, top_note, 100)).unwrap();
                ::std::thread::sleep_ms(150);
                send.send(Message::NoteOff(Channel::Ch1, top_note, 100)).unwrap();
            }
        }

        send.send(Message::NoteOff(Channel::Ch1, note,      100)).unwrap();
        send.send(Message::NoteOff(Channel::Ch1, note + 4,  100)).unwrap();
        send.send(Message::NoteOff(Channel::Ch1, note + 7,  100)).unwrap();
        ::std::thread::sleep_ms(1000);
    }

    audio_unit.close();
}
