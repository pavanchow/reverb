# Design

## Oscillators

An oscillator produces one sample per time step for a given waveform, frequency, and sample rate. Instead of tracking a running phase accumulator, each sample computes its phase directly from time: `phase = fract(freq_hz * t)`, where `t = n / sample_rate`. That keeps the oscillator stateless and easy to test in isolation.

Four shapes, each a function of phase in `[0, 1)`:

- Sine: `sin(2 * pi * phase)`.
- Square: `+1` for the first half of the cycle, `-1` for the second half.
- Sawtooth: a straight ramp from `-1` to `1` across the cycle.
- Triangle: rises from `-1` to `1` over the first half, falls back over the second half.

All four are pure functions of phase, so adding a waveform means adding one match arm.

## The ADSR envelope

An envelope shapes a note's amplitude over time so it doesn't start or stop abruptly. Four stages, each a straight line:

- Attack: ramp from 0 to 1 over `attack` seconds.
- Decay: ramp from 1 down to the `sustain` level over `decay` seconds.
- Sustain: hold flat at the `sustain` level.
- Release: ramp from `sustain` down to 0 over the final `release` seconds of the note.

The envelope only needs to know how long the whole note lasts and where it currently is in time, so `amplitude_at(t, note_len)` is a pure function too, no mutable state carried between calls. Applying it to a buffer of raw oscillator samples is a per-sample multiply.

## Sequencing, mixing, normalization

A `Note` is just a frequency and a duration. `render_sequence` walks a list of notes, renders each one through an oscillator and shapes it with an envelope, and concatenates the results into one track. That is the whole sequencer, no scheduling engine needed because everything plays back to back.

Multiple tracks (say, a lead melody and a bass line) can then be mixed by summing them sample by sample, padding the shorter track with silence so the lengths line up. Summing can push values outside `[-1, 1]`, so normalization finds the peak absolute value across the mixed buffer and, if it exceeds 1.0, divides every sample by that peak. Quiet mixes are left alone.

## The WAV file format, written by hand

A WAV file is a RIFF container with two chunks Reverb cares about, `fmt ` and `data`, wrapped in an outer RIFF header.

Byte layout (little-endian throughout):

| Offset | Bytes | Field |
|---|---|---|
| 0 | 4 | `"RIFF"` |
| 4 | 4 | file size minus 8, as a u32 |
| 8 | 4 | `"WAVE"` |
| 12 | 4 | `"fmt "` |
| 16 | 4 | fmt chunk size, always 16 for PCM |
| 20 | 2 | audio format, 1 for PCM |
| 22 | 2 | number of channels, 1 here |
| 24 | 4 | sample rate |
| 28 | 4 | byte rate: `sample_rate * channels * bytes_per_sample` |
| 32 | 2 | block align: `channels * bytes_per_sample` |
| 34 | 2 | bits per sample, 16 |
| 36 | 4 | `"data"` |
| 40 | 4 | data chunk size in bytes |
| 44 | ... | the samples themselves |

Each sample is a `f32` in `[-1.0, 1.0]`, clamped to that range, scaled by `i16::MAX`, rounded, and written as a little-endian 16-bit signed integer. The whole writer is one function that pushes bytes onto a `Vec<u8>` in order, no seeking, no chunk rewriting, because every size is known before the first byte is written.
