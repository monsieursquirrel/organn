// a combined set of voices

use basic_types::unthreaded_connection;
use basic_types::threaded_connection;
use voice::Voice;
use mixer::Mixer;
use midi::{self, Message};

use std::sync::mpsc;
use std::thread;

// voice inputs with note assignments
struct VoiceAssign {
    voice: mpsc::Sender<midi::Message>,
    note: Option<midi::U7>,
}

impl VoiceAssign {
    fn new(voice: mpsc::Sender<midi::Message>) -> Self {
        VoiceAssign {
            voice: voice,
            note: None
        }
    }
}


pub struct MultiMidiConn {
    voices: Vec<VoiceAssign>,
    last_voice: usize
}

impl MultiMidiConn {
    fn new(voice_inputs: Vec<mpsc::Sender<midi::Message>>) -> Self {
        let voice_assigns = voice_inputs
            .into_iter()
            .map(|v| {
                    VoiceAssign::new(v)
                })
            .collect();

        MultiMidiConn {
            voices: voice_assigns,
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
                voice.voice.send(message.clone());
            }

            Message::NoteOff(_, pitch, _) => {
                // send to appropriate voice(s) and unassign their notes
                for voice in self.voices.iter_mut().filter(|v| v.note == Some(pitch)) {
                    voice.note = None;
                    voice.voice.send(message.clone());
                }
            }

            _ => {
                // TODO: send to everything!
            }
        }
    }
}

pub struct Multi {
    voice_threads: Vec<thread::JoinHandle<()>>,
    mixer: Mixer<threaded_connection::ThreadedInput, unthreaded_connection::UnthreadedOutput>,
}

impl Multi {
    pub fn new(num_voices: usize, sample_rate: u32) -> (Self, MultiMidiConn, unthreaded_connection::UnthreadedInput) {

        let mut voice_threads = Vec::new();
        let mut midi_connections = Vec::new();
        let mut voice_connections = Vec::new();

        // spawn voice threads
        for _ in (0..4) {
            let mut voices = Vec::new();

            for _ in (0..(num_voices / 4)) {
                let (voice, midi_conn, conn) = Voice::new(sample_rate);

                voices.push(voice);
                midi_connections.push(midi_conn);
                voice_connections.push(conn);
            }

            let thread = thread::spawn(move || {
                    loop {
                        for voice in voices.iter_mut() {
                            if (voice.run().is_err()) {
                                return;
                            }
                        }
                    }
                });

            voice_threads.push(thread);
        }

        let midi_conn = MultiMidiConn::new(midi_connections);

        let (output, input) = unthreaded_connection::new();
        let mixer = Mixer::new(voice_connections, vec![0.25; num_voices], output);

        (
            Multi {
                voice_threads: voice_threads,
                mixer: mixer
            },
            midi_conn,
            input
        )
    }

    pub fn run(&mut self) {
        self.mixer.run();
    }
}
