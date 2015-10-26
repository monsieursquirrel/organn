
use pitch_calc::Step;

use basic_types::threaded_connection;
use basic_types::unthreaded_connection;
use oscillator::Oscillator;
use mixer::Mixer;
use env::Env;
use midi::{self, Message};

use std::sync::mpsc;

static MIX_MAX: f32 = 0.25;

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

        for _ in (0..9) {
            let (output, input) = unthreaded_connection::new();
            let osc = Oscillator::new(sample_rate, output);

            oscillators.push(osc);
            osc_connections.push(input);
        }

        let num_oscs = osc_connections.len();
        let (mix_output, env_input) = unthreaded_connection::new();
        let mut mixer = Mixer::new(osc_connections, vec![0.0; num_oscs], mix_output);

        mixer.set_level(0, 1.0 * MIX_MAX);
        mixer.set_level(1, 0.6 * MIX_MAX);
        mixer.set_level(2, 0.1 * MIX_MAX);
        mixer.set_level(3, 0.4 * MIX_MAX);
        mixer.set_level(4, 0.1 * MIX_MAX);
        mixer.set_level(5, 0.4 * MIX_MAX);
        mixer.set_level(6, 0.1 * MIX_MAX);
        mixer.set_level(7, 0.1 * MIX_MAX);
        mixer.set_level(8, 0.1 * MIX_MAX);

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
        self.oscillators[0].set_freq((freq * 1.0 )  / 2.0);
        self.oscillators[1].set_freq((freq * 16.0)  / 11.0);
        self.oscillators[2].set_freq(freq * 1.0);
        self.oscillators[3].set_freq(freq * 2.0);
        self.oscillators[4].set_freq(freq * 3.0);
        self.oscillators[5].set_freq(freq * 4.0);
        self.oscillators[6].set_freq(freq * 5.0);
        self.oscillators[7].set_freq(freq * 6.0);
        self.oscillators[8].set_freq(freq * 8.0);
    }

    // convert a midi value to a mix level
    // reversed to resemble drawbars
    fn midi_to_mix_level(value: midi::U7) -> f32 {
        ((127 - value) as f32 * MIX_MAX) / 127.0
    }

    fn midi_control(&mut self, control: midi::U7, value: midi::U7) {
        // map some midi controls to the mix
        match control {
            2  => { self.mixer.set_level(0, Self::midi_to_mix_level(value)); }
            3  => { self.mixer.set_level(1, Self::midi_to_mix_level(value)); }
            4  => { self.mixer.set_level(2, Self::midi_to_mix_level(value)); }
            5  => { self.mixer.set_level(3, Self::midi_to_mix_level(value)); }
            6  => { self.mixer.set_level(4, Self::midi_to_mix_level(value)); }
            8  => { self.mixer.set_level(5, Self::midi_to_mix_level(value)); }
            9  => { self.mixer.set_level(6, Self::midi_to_mix_level(value)); }
            12 => { self.mixer.set_level(7, Self::midi_to_mix_level(value)); }
            13 => { self.mixer.set_level(8, Self::midi_to_mix_level(value)); }
            _ => {}
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

            Message::ControlChange(_, control, value) => {
                self.midi_control(control, value);
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
