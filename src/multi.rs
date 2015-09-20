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

impl VoiceAssign {
    fn new(voice: Rc<RefCell<Voice>>) -> Self {
        VoiceAssign {
            voice: voice,
            note: None
        }
    }
}

pub struct Multi {
    voices: Vec<VoiceAssign>,
    mixer: Mixer<Rc<RefCell<Voice>>>,
    last_voice: usize
}

impl Multi {
    pub fn new(num_voices: usize, sample_rate: u32) -> Self {
        let voices: Vec<_> = (0..num_voices)
            .map(|_| Rc::new(RefCell::new(Voice::new(sample_rate))) )
            .collect();

        let voice_refs = voices.iter().map(|ref_voc| ref_voc.clone()).collect();
        let mixer = Mixer::new(voice_refs, vec![0.25; voices.len()]);

        let voice_assigns = voices.into_iter().map(|ref_voc| VoiceAssign::new(ref_voc)).collect();

        Multi {
            voices: voice_assigns,
            mixer: mixer,
            last_voice: 0
        }
    }

    fn pick_voice(&mut self) -> &mut VoiceAssign {
        let num_voices = self.voices.len();
        let maybe_index = (0..num_voices)
            .map(|i| (self.last_voice + i) % num_voices)
            .filter(|i| self.voices[*i].note.is_none())
            .next();

        // replace a note if there isn't a silent voice available
        let index = maybe_index.unwrap_or((self.last_voice + 1) % num_voices);
        self.last_voice = index;
        &mut self.voices[index]
    }

    pub fn midi_message(&mut self, message: &Message) {
        match *message {
            Message::NoteOn(_, pitch, _) => {
                // pick a voice to use
                let mut voice = self.pick_voice();
                voice.note = Some(pitch);
                voice.voice.borrow_mut().midi_message(message);
            }

            Message::NoteOff(_, pitch, _) => {
                // send to appropriate voice(s) and unassign their notes
                for voice in self.voices.iter_mut().filter(|v| v.note == Some(pitch)) {
                    voice.note = None;
                    voice.voice.borrow_mut().midi_message(message);
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
