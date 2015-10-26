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
    use basic_types::threaded_connection;
    pub type UnthreadedInput = threaded_connection::ThreadedInput;
    pub type UnthreadedOutput = threaded_connection::ThreadedOutput;

    pub fn new() -> (UnthreadedOutput, UnthreadedInput) {
        threaded_connection::new()
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
            self.send(buffer);
        }
    }

    impl Input for ThreadedInput {
        fn get_audio(&self) -> AudioBuffer {
            self.recv().unwrap()
        }
    }
}
