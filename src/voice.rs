use std::cell::RefCell;
use std::rc::Rc;
use pitch_calc::Step;

use produce_audio::{ProduceAudioMut, ProduceAudio};
use oscillator::Oscillator;
use mixer::Mixer;

pub struct Voice {
    oscillators: Vec<Rc<RefCell<Oscillator>>>,
    mixer: Mixer<Rc<RefCell<Oscillator>>>
}

impl Voice {
    pub fn new() -> Self {
        // create the parts of the signal chain
        let oscillators: Vec<_> = (0..8)
            .map(|mult| Rc::new(RefCell::new(Oscillator::new(440.0 * ((mult + 1) as f32), 44_100))))
            .collect();


        let borrowed_oscs: Vec<_> = oscillators.iter().map(|ref_osc| ref_osc.clone()).collect();
        let mixer = Mixer::new(borrowed_oscs, vec![0.0; oscillators.len()]);

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

    pub fn set_pitch(&mut self, pitch: f32) {
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
