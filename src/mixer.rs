use basic_types::{ProduceAudio, ProduceAudioMut, Input, Output, BUFFER_SIZE, AudioBuffer};

pub struct Mixer<T> where T: Input {
    levels: Vec<f32>,
    inputs: Vec<T>
}

impl<T> Mixer<T> where T: Input {
    pub fn new(inputs: Vec<T>, levels: Vec<f32>) -> Self {
        Mixer {
            levels: levels,
            inputs: inputs
        }
    }

    pub fn set_level(&mut self, input_num: usize, level: f32) {
        self.levels[input_num] = level;
    }

    pub fn run(&mut self) {
        let samples = self.inputs.iter()
            .map(|input| input.get_audio())
            .zip(self.levels.iter())
            .map(|(buffer, level)| buffer.map(|sample| sample * level).collect() )
            .fold([0.0; BUFFER_SIZE], |buffer, outbuf| outbuf.iter().zip(buffer).map(|input, sum| input + sum).collect() );
        self.output.supply_audio(samples);
    }
}
