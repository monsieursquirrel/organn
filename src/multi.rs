// a combined set of voices
use std::cell::RefCell;
use std::rc::Rc;

use produce_audio::{ProduceAudio, ProduceAudioMut};
use voice::Voice;
use mixer::Mixer;
use midi::{self, Message};

// voices with note assignments
struct VoiceAssign {
    voice: Rc<RefCell<Voice>>,
    note: Option<midi::U7>,
}

pub struct Multi {
    voices: Vec<Rc<RefCell<Voice>>>,
    mixer: Mixer<Rc<RefCell<Voice>>>,
    last_voice: usize
}

impl Multi {
    pub fn new(sample_rate: u32) -> Self {
        let voices: Vec<_> = (0..128)
            .map(|_| Rc::new(RefCell::new(Voice::new(sample_rate))) )
            .collect();

        let voice_refs = voices.iter().map(|ref_voc| ref_voc.clone()).collect();
        let mixer = Mixer::new(voice_refs, vec![0.25; voices.len()]);

        Multi {
            voices: voices,
            mixer: mixer,
            last_voice: 0
        }
    }

    pub fn midi_message(&mut self, message: &Message) {
        match *message {
            Message::NoteOn(_, pitch, _) => {
                // pick a voice to use
                if let Some(mut voice) = self.voices.get(pitch as usize) {
                    voice.borrow_mut().midi_message(message);
                }
            }

            Message::NoteOff(_, pitch, _) => {
                if let Some(mut voice) = self.voices.get(pitch as usize) {
                    voice.borrow_mut().midi_message(message);
                }
            }

            _ => {
                // send to everything!
            }
        }
    }
}

impl ProduceAudioMut for Multi {
    fn next_sample(&mut self) -> f32 {
        self.mixer.next_sample()
    }
}
