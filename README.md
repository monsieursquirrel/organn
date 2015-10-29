# Organn

An attempt at something resembling a drawbar organ. Largely an experiment in audio coding, ffi etc.
It's playable using midi and sounds something like [this](https://soundcloud.com/whatmilk/organn-v040).
The drawbar controls are mapped to midi cc numbers 2, 3, 4, 5, 6, 8, 9, 12 and 13.
These were chosen because I had a nanoKontrol to hand.

Only supports CoreAudio/CoreMidi on OSX. Apologies to linux/windows people. I eventually plan to get some other audio support working.

## Building/running

I recommend using release mode builds, use `cargo build --release` and `cargo run --release`.
Currently there are no command line options. Organn will run until you press enter.

## Aknowledgments

Big thanks to [RustAudio](https://github.com/RustAudio) for the library bindings to CoreAudio.
Also, thanks to Valentin Ochs for his [article on numerically controlled oscillators](http://0au.de/2015/07/numerically-controlled-oscillators/).
