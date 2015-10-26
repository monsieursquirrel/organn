# organn

An attempt at something resembling a drawbar organ. Largely an experiment in audio coding, ffi etc.
It's playable using midi and sounds something like [this](https://soundcloud.com/whatmilk/organn-v030).
The drawbar controls are mapped to midi cc numbers 2, 3, 4, 5, 6, 8, 9 and 12.
These were chosen because I had a nanoKontrol to hand and that's the default mapping.

Only supports CoreAudio/CoreMidi on OSX. Apologies to linux/windows people. I eventually plan to get some other audio support working.

Big thanks to [RustAudio](https://github.com/RustAudio) for the library bindings; the sound output side of this has been easy so far.
