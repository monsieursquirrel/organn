use std::cell::RefCell;
use std::rc::Rc;
use pitch_calc::Step;

use basic_types::{ProduceAudioMut, ProduceAudio, UnthreadedConnection};
use oscillator::Oscillator;
use mixer::Mixer;
use env::Env;
use midi::{self, Message};

pub struct Voice {
    oscillators: Vec<Oscillator<UnthreadedConnection::UnthreadedOutput>>,
    mixer: Mixer<UnthreadedConnection::UnthreadedInput, UnthreadedConnection::UnthreadedOutput>,
    env: Env<UnthreadedConnection::UnthreadedInput, UnthreadedConnection::UnthreadedOutput>,
    pitch: midi::U7
}

impl Voice {
    pub fn new(sample_rate: u32) -> (Self, UnthreadedConnection::UnthreadedInput) {
        // create the parts of the signal chain
        let oscillators = Vec::new();
        let osc_connections = Vec::new();

        for _ in (0..8) {
            let (output, input) = UnthreadedConnection::new();
            let osc = Oscillator::new(sample_rate, output);

            oscillators.push(osc);
            osc_connections.push(input);
        }

        let num_oscs = osc_connections.len();
        let (mix_output, env_input) = UnthreadedConnection::new();
        let mixer = Mixer::new(osc_connections, vec![0.0; num_oscs], mix_output);

        mixer.set_level(0, 0.5);
        mixer.set_level(1, 0.3);
        mixer.set_level(2, 0.05);
        mixer.set_level(3, 0.2);
        mixer.set_level(4, 0.05);
        mixer.set_level(5, 0.2);
        mixer.set_level(6, 0.05);
        mixer.set_level(7, 0.05);

        let (env_output, voice_connection) = UnthreadedConnection::new();

        let env = Env::new(env_input, env_output, 20, sample_rate);

        (
            Voice {
                oscillators: oscillators,
                mixer: mixer,
                env: env,
                pitch: 0
            },
            voice_connection
        )
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        let freq = Step(pitch).to_hz().hz();
        for (num, oscillator) in self.oscillators.iter_mut().enumerate() {
            oscillator.set_freq(freq * ((num + 1) as f32));
        }
    }

    pub fn midi_message(&mut self, message: &Message) {
        match *message {
            Message::NoteOn(_, pitch, _) => {
                self.set_pitch(pitch as f32);
                self.env.note_on();

                self.pitch = pitch;
            }

            Message::NoteOff(_, pitch, _) if (pitch == self.pitch) => {
                self.env.note_off();
            }

            Message::AllNotesOff(_) => {
                self.env.note_off();
            }

            _ => { }
        }
    }

    pub fn run(&mut self) {
        // just run every part of the audio chain in order
        for osc in self.oscillators.iter() {
            osc.run();
        }
        self.mixer.run();
        self.env.run();
    }
}
