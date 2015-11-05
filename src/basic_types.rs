// audio buffer type
pub const BUFFER_SIZE: usize = 16;
pub type AudioBuffer = [f32; BUFFER_SIZE];
pub const BLANK_BUFFER: AudioBuffer = [0.0; BUFFER_SIZE];

// audio connection traits
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


pub mod threaded_connection {
    use std::sync::mpsc::{SyncSender, Receiver, sync_channel};
    use basic_types::{AudioBuffer, Output, Input};

    pub type ThreadedOutput = SyncSender<AudioBuffer>;
    pub type ThreadedInput = Receiver<AudioBuffer>;

    pub fn new() -> (ThreadedOutput, ThreadedInput) {
        sync_channel(1)
    }

    impl Output for ThreadedOutput {
        fn supply_audio(&self, buffer: AudioBuffer) {
            self.send(buffer).unwrap();
        }
    }

    impl Input for ThreadedInput {
        fn get_audio(&self) -> AudioBuffer {
            self.recv().unwrap()
        }
    }
}
