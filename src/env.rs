// really simple envelope, short linear attack/release, mostly for preventing clicks

use basic_types::{Input, Output, BUFFER_SIZE, AudioBuffer};

enum State {
    Off,
    Up,
    On,
    Down
}

pub struct Env<T, U> where T: Input, U: Output {
    input: T,
    output: U,
    state: State,
    pos: u32,
    ramp_samples: u32,
}

impl<T, U> Env<T, U> where T: Input, U: Output {
    pub fn new(input: T, output: U, time_ms: u32, sample_rate: u32) -> Self {
        Env {
            input: input,
            output: output,
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

    pub fn run(&mut self) {
        let mut samples: AudioBuffer = [0.0; BUFFER_SIZE];

        let in_samples = self.input.get_audio();
        for (sample, in_sample) in samples.iter_mut().zip(in_samples.iter()) {
            self.update();
            *sample = (*in_sample * self.pos as f32) / self.ramp_samples as f32;
        }

        self.output.supply_audio(samples);
    }
}
