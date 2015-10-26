
use pitch_calc::Step;

use basic_types::threaded_connection;
use basic_types::unthreaded_connection;
use oscillator::Oscillator;
use mixer::Mixer;
use env::Env;
use midi::{self, Message};

use std::sync::mpsc;

pub struct Voice {
    oscillators: Vec<Oscillator<unthreaded_connection::UnthreadedOutput>>,
    mixer: Mixer<unthreaded_connection::UnthreadedInput, unthreaded_connection::UnthreadedOutput>,
    env: Env<unthreaded_connection::UnthreadedInput, threaded_connection::ThreadedOutput>,
    pitch: midi::U7,
    midi_input: mpsc::Receiver<midi::Message>
}

impl Voice {
    pub fn new(sample_rate: u32, midi_in: mpsc::Receiver<midi::Message>, output: threaded_connection::ThreadedOutput) -> Self {
        // create the parts of the signal chain
        let mut oscillators = Vec::new();
        let mut osc_connections = Vec::new();

        for _ in (0..8) {
            let (output, input) = unthreaded_connection::new();
            let osc = Oscillator::new(sample_rate, output);

            oscillators.push(osc);
            osc_connections.push(input);
        }

        let num_oscs = osc_connections.len();
        let (mix_output, env_input) = unthreaded_connection::new();
        let mut mixer = Mixer::new(osc_connections, vec![0.0; num_oscs], mix_output);

        mixer.set_level(0, 0.5);
        mixer.set_level(1, 0.3);
        mixer.set_level(2, 0.05);
        mixer.set_level(3, 0.2);
        mixer.set_level(4, 0.05);
        mixer.set_level(5, 0.2);
        mixer.set_level(6, 0.05);
        mixer.set_level(7, 0.05);

        let env = Env::new(env_input, output, 20, sample_rate);

        Voice {
            oscillators: oscillators,
            mixer: mixer,
            env: env,
            pitch: 0,
            midi_input: midi_in
        }
    }

    fn set_pitch(&mut self, pitch: f32) {
        let freq = Step(pitch).to_hz().hz();
        for (num, oscillator) in self.oscillators.iter_mut().enumerate() {
            oscillator.set_freq(freq * ((num + 1) as f32));
        }
    }

    fn midi_message(&mut self, message: &Message) {
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

    pub fn run(&mut self) -> Result<(), ()>{
        // process messages for this voice
        loop {
            let message = self.midi_input.try_recv();
            match message {
                Ok(midi_message) => {
                    self.midi_message(&midi_message);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    break;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // get out of here
                    return Err(());
                }
            }
        }

        // just run every part of the audio chain in order
        for osc in self.oscillators.iter_mut() {
            osc.run();
        }
        self.mixer.run();
        self.env.run();

        Ok(())
    }
}
