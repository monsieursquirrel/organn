// a combined set of voices

use basic_types::unthreaded_connection;
use voice::Voice;
use mixer::Mixer;
use midi::{self, Message};

// voices with note assignments
struct VoiceAssign {
    voice: Voice,
    note: Option<midi::U7>,
}

impl VoiceAssign {
    fn new(voice: Voice) -> Self {
        VoiceAssign {
            voice: voice,
            note: None
        }
    }
}

pub struct Multi {
    voices: Vec<VoiceAssign>,
    mixer: Mixer<unthreaded_connection::UnthreadedInput, unthreaded_connection::UnthreadedOutput>,
    last_voice: usize
}

impl Multi {
    pub fn new(num_voices: usize, sample_rate: u32) -> (Self, unthreaded_connection::UnthreadedInput) {

        let mut voices = Vec::new();
        let mut voice_connections = Vec::new();

        for _ in (0..num_voices) {
            let (voice, conn) = Voice::new(sample_rate);

            voices.push(voice);
            voice_connections.push(conn);
        }

        let (output, input) = unthreaded_connection::new();
        let mixer = Mixer::new(voice_connections, vec![0.25; num_voices], output);

        let voice_assigns = voices.into_iter().map(|ref_voc| VoiceAssign::new(ref_voc)).collect();

        (
            Multi {
                voices: voice_assigns,
                mixer: mixer,
                last_voice: 0
            },
            input
        )
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
                voice.voice.midi_message(message);
            }

            Message::NoteOff(_, pitch, _) => {
                // send to appropriate voice(s) and unassign their notes
                for voice in self.voices.iter_mut().filter(|v| v.note == Some(pitch)) {
                    voice.note = None;
                    voice.voice.midi_message(message);
                }
            }

            _ => {
                // send to everything!
            }
        }
    }

    pub fn run(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.voice.run();
        }
        self.mixer.run();
    }
}
