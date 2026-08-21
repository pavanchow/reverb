**A small, readable audio synthesis engine in Rust.**

Reverb generates real sound from scratch and writes standard WAV files. No audio crates, no hidden DSP library, just oscillators, an envelope, a mixer, and a WAV writer you can read end to end in one sitting.

## What it does

- Four oscillator waveforms: sine, square, sawtooth, triangle, at any frequency and sample rate.
- An ADSR envelope (attack, decay, sustain, release) shapes each note so it doesn't click on and off.
- A sequencer strings notes into a track, a mixer sums multiple tracks together, and normalization keeps the result inside the valid sample range.
- A hand-written 16-bit PCM WAV file writer: RIFF header, fmt chunk, data chunk, byte for byte, no crate does it for you.
- A note-name helper so you can write `A4` or `C#3` instead of raw hertz.

## Pipeline

```
note name -> frequency -> oscillator -> ADSR envelope -> sequence -> mix -> normalize -> WAV bytes
```

Each stage is a plain function or small struct in `src/`, see `DESIGN.md` for the details of each one.

## Usage

Build it:

```
cargo build --release
```

Render a single tone:

```
reverb tone --freq 440 --secs 2 -o out.wav
```

Render a short built-in melody:

```
reverb demo -o demo.wav
```

## Tests

```
cargo test
```

Covers sample counts and range, WAV header correctness, envelope shape, and waveform values.

By Pavan Nallamothu.
