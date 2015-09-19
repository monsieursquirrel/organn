

pub struct Mixer{
    levels: Vec<f32>
}

impl Mixer {
    pub fn new(levels: Vec<f32>) -> Self {
        Mixer {
            levels: levels
        }
    }

    pub fn set_level(&mut self, input_num: usize, level: f32) {
        self.levels[input_num] = level;
    }

    pub fn mix(&self, inputs: &[f32]) -> f32 {
        inputs.iter()
        .zip(self.levels.iter())
        .map(|(input, level)| input * level)
        .fold(0.0, |input, sum| input + sum)
    }
}
