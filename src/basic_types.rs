use std::cell::RefCell;
use std::rc::Rc;

// audio buffer type
pub const BUFFER_SIZE: usize = 32;
pub type AudioBuffer = [f32; BUFFER_SIZE];


// audio connection traits, will eventually have threaded and unththreaded impls
// not sure which way round these shoukd be named, currently Output puts audio into the buffer...

pub trait Output {
    /// Write audio into the buffer
    fn supply_audio(&self, buffer: AudioBuffer);
}

pub trait Input {
    /// Get audio out of the buffer
    fn get_audio(&self) -> AudioBuffer;
}

// unththreaded audio buffer

pub mod unthreaded_connection {
    use std::cell::RefCell;
    use std::rc::Rc;
    use basic_types::{AudioBuffer, Output, Input};

    pub struct UnthreadedOutput {
        buffer: Rc<RefCell<Option<AudioBuffer>>>
    }

    pub struct UnthreadedInput {
        buffer: Rc<RefCell<Option<AudioBuffer>>>
    }

    pub fn new() -> (UnthreadedOutput, UnthreadedInput) {
        let inner_buf = Rc::new(RefCell::new(None));
        (
        UnthreadedOutput {
            buffer: inner_buf.clone()
        },
        UnthreadedInput {
            buffer: inner_buf.clone()
        }
        )
    }

    impl Output for UnthreadedOutput {
        fn supply_audio(&self, buffer: AudioBuffer) {
            let mut inner_buf = self.buffer.borrow_mut();
            *inner_buf = Some(buffer);
        }
    }

    impl Input for UnthreadedInput {
        fn get_audio(&self) -> AudioBuffer {
            self.buffer.borrow_mut().take().unwrap()
        }
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
