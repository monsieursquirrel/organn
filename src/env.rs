// really simple envelope, short linear attack/release, mostly for preventing clicks

use produce_audio::{ProduceAudio, ProduceAudioMut};

enum State {
    Off,
    Up,
    On,
    Down
}

pub struct Env<T> where T: ProduceAudio {
    input: T,
    state: State,
    pos: u32,
    ramp_samples: u32,
}

impl<T> Env<T> where T: ProduceAudio {
    pub fn new(input: T, time_ms: u32, sample_rate: u32) -> Self {
        Env {
            input: input,
            state: State::Off,
            pos: 0,
            ramp_samples: (time_ms * sample_rate) / 1000
        }
    }

    pub fn note_on(&mut self) {
        self.state = State::Up;
    }

    pub fn note_off(&mut self) {
        self.state = State::Down;
    }

    // update pos/state once per sample
    fn update(&mut self) {
        match self.state {
            State::Up => {
                if self.pos < self.ramp_samples {
                    self.pos += 1;
                }
                else {
                    self.state = State::On;
                }
            }
            State::Down => {
                if self.pos > 0 {
                    self.pos -= 1;
                }
                else {
                    self.state = State::Off;
                }
            }
            _ => {}
        }
    }
}

impl<T> ProduceAudioMut for Env<T> where T: ProduceAudio {
    fn next_sample(&mut self) -> f32 {
        self.update();
        let gain = (self.pos as f32) / (self.ramp_samples as f32);
        self.input.next_sample() * gain
    }
}
