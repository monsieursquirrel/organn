use std::cell::RefCell;
use std::rc::Rc;

// audio buffer type
pub const BUFFER_SIZE: usize = 32;
pub type AudioBuffer = [f32; BUFFER_SIZE];


// audio connection traits, will eventually have threaded and unththreaded impls
// not sure which way round these shoukd be named, currently Input puts audio into the buffer...

pub trait Input {
    /// Write audio into the buffer
    fn supply_audio(&mut self, buffer: AudioBuffer);
}

pub trait Output {
    /// Get audio out of the buffer
    fn get_audio(&mut self) -> AudioBuffer;
}

// unththreaded audio buffer

pub struct UnthreadedBuffer {
    buffer: Option<AudioBuffer>
}

impl UnthreadedBuffer {
    pub fn new() -> Self {
        UnthreadedBuffer {
            buffer: None
        }
    }
}

impl Input for UnthreadedBuffer {
    fn supply_audio(&mut self, buffer: AudioBuffer) {
        self.buffer = Some(buffer);
    }
}

impl Output for UnthreadedBuffer {
    fn get_audio(&mut self) -> AudioBuffer {
        self.buffer.take().unwrap()
    }
}

// a trait for mutable things to produce audio, a way for things to consume audio from shared
// objects and a ref cell impl to bridge the gap

pub trait ProduceAudioMut {
    fn next_sample(&mut self) -> f32;
}

pub trait ProduceAudio {
    fn next_sample(&self) -> f32;
}

impl<T> ProduceAudio for RefCell<T> where T: ProduceAudioMut {
    fn next_sample(&self) -> f32 {
        let mut inner = self.borrow_mut();
        inner.next_sample()
    }
}

impl<T> ProduceAudio for Rc<T> where T: ProduceAudio {
    fn next_sample(&self) -> f32 {
        (**self).next_sample()
    }
}
