use basic_types::{Input, Output, BLANK_BUFFER, AudioBuffer};

pub struct Mixer<T, U> where T: Input, U: Output {
    levels: Vec<f32>,
    inputs: Vec<T>,
    output: U
}

impl<T, U> Mixer<T, U> where T: Input, U: Output {
    pub fn new(inputs: Vec<T>, levels: Vec<f32>, output: U) -> Self {
        Mixer {
            levels: levels,
            inputs: inputs,
            output: output
        }
    }

    pub fn set_level(&mut self, input_num: usize, level: f32) {
        self.levels[input_num] = level;
    }

    pub fn run(&mut self) {
        let mut samples: AudioBuffer = BLANK_BUFFER;

        for (input, level) in self.inputs.iter().zip(self.levels.iter()) {
            let in_samples = input.get_audio();
            for (sample, in_sample) in samples.iter_mut().zip(in_samples.iter()) {
                *sample += *in_sample * level;
            }
        }

        self.output.supply_audio(samples);
    }
}
