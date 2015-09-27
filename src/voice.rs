use std::cell::RefCell;
use std::rc::Rc;
use pitch_calc::Step;

use basic_types::{ProduceAudioMut, ProduceAudio};
use oscillator::Oscillator;
use mixer::Mixer;
use env::Env;
use midi::{self, Message};

pub struct Voice {
    oscillators: Vec<Rc<RefCell<Oscillator>>>,
    mixer: Rc<RefCell<Mixer<Rc<RefCell<Oscillator>>>>>,
    env: Env<Rc<RefCell<Mixer<Rc<RefCell<Oscillator>>>>>>,
    pitch: midi::U7
}

impl Voice {
    pub fn new(sample_rate: u32) -> Self {
        // create the parts of the signal chain
        let oscillators: Vec<_> = (0..8)
            .map(|_| Rc::new(RefCell::new(Oscillator::new(sample_rate))))
            .collect();


        let borrowed_oscs: Vec<_> = oscillators.iter().map(|ref_osc| ref_osc.clone()).collect();
        let mixer = Rc::new(RefCell::new(Mixer::new(borrowed_oscs, vec![0.0; oscillators.len()])));

        mixer.borrow_mut().set_level(0, 0.5);
        mixer.borrow_mut().set_level(1, 0.3);
        mixer.borrow_mut().set_level(2, 0.05);
        mixer.borrow_mut().set_level(3, 0.2);
        mixer.borrow_mut().set_level(4, 0.05);
        mixer.borrow_mut().set_level(5, 0.2);
        mixer.borrow_mut().set_level(6, 0.05);
        mixer.borrow_mut().set_level(7, 0.05);

        let env = Env::new(mixer.clone(), 20, sample_rate);

        Voice {
            oscillators: oscillators,
            mixer: mixer,
            env: env,
            pitch: 0
        }
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        let freq = Step(pitch).to_hz().hz();
        for (num, ref_osc) in self.oscillators.iter_mut().enumerate() {
            let mut oscillator = (**ref_osc).borrow_mut();
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
}

impl ProduceAudioMut for Voice {
    fn next_sample(&mut self) -> f32 {
        self.env.next_sample()
    }
}
