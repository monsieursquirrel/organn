use std::cell::RefCell;
use std::rc::Rc;

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
