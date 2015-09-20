use produce_audio::{ProduceAudio, ProduceAudioMut};

pub struct Mixer<T> where T: ProduceAudio {
    levels: Vec<f32>,
    inputs: Vec<T>
}

impl<T> Mixer<T> where T: ProduceAudio {
    pub fn new(inputs: Vec<T>, levels: Vec<f32>) -> Self {
        Mixer {
            levels: levels,
            inputs: inputs
        }
    }

    pub fn set_level(&mut self, input_num: usize, level: f32) {
        self.levels[input_num] = level;
    }
}

impl<T> ProduceAudioMut for Mixer<T> where T: ProduceAudio {
    fn next_sample(&mut self) -> f32 {
        self.inputs.iter()
        .zip(self.levels.iter())
        .map(|(input, level)| input.next_sample() * level)
        .fold(0.0, |input, sum| input + sum)
    }
}
