//! Public-API integration tests for reverb, the from-scratch synth + WAV encoder.
//! Drives the crate through its re-exported surface only, as a dependent would.

use reverb::{
    mix_and_normalize, note_to_freq, render_sequence, wav_bytes, write_wav, Adsr, Note, Oscillator,
    Waveform,
};

fn approx(a: f32, b: f32, eps: f32) -> bool {
    (a - b).abs() < eps
}

// ---- note_to_freq: authoritative equal-temperament vectors -----------------

#[test]
fn note_to_freq_matches_published_reference_pitches() {
    // Standard twelve-tone equal-temperament values, A4 = 440 Hz.
    for (name, hz) in [
        ("A4", 440.0),
        ("C4", 261.626),
        ("A0", 27.5),
        ("C0", 16.3516),
        ("A5", 880.0),
        ("A3", 220.0),
        ("E2", 82.4069),
        ("G9", 12543.85),
    ] {
        let f = note_to_freq(name).unwrap_or_else(|| panic!("{name} failed to parse"));
        assert!(approx(f, hz, hz * 1e-3), "{name}: got {f}, want {hz}");
    }
}

#[test]
fn octave_up_doubles_frequency() {
    let a4 = note_to_freq("A4").unwrap();
    let a5 = note_to_freq("A5").unwrap();
    let a3 = note_to_freq("A3").unwrap();
    assert!(approx(a5 / a4, 2.0, 1e-4));
    assert!(approx(a4 / a3, 2.0, 1e-4));
}

#[test]
fn sharp_and_flat_are_enharmonically_equal() {
    assert!(approx(
        note_to_freq("C#4").unwrap(),
        note_to_freq("Db4").unwrap(),
        1e-4
    ));
    // A#3 == Bb3
    assert!(approx(
        note_to_freq("A#3").unwrap(),
        note_to_freq("Bb3").unwrap(),
        1e-4
    ));
}

#[test]
fn note_to_freq_rejects_malformed_input() {
    assert!(note_to_freq("").is_none()); // empty
    assert!(note_to_freq("H4").is_none()); // not a note letter
    assert!(note_to_freq("A").is_none()); // missing octave
    assert!(note_to_freq("Ax").is_none()); // unparseable octave
    assert!(note_to_freq("4A").is_none()); // digit first
}

// ---- oscillator: shape and range ------------------------------------------

#[test]
fn every_waveform_stays_in_range_and_has_requested_length() {
    let sr = 44100;
    for wf in [
        Waveform::Sine,
        Waveform::Square,
        Waveform::Sawtooth,
        Waveform::Triangle,
    ] {
        let osc = Oscillator::new(wf, 440.0, sr);
        let s = osc.render(sr as usize);
        assert_eq!(s.len(), sr as usize);
        assert!(s.iter().all(|v| *v >= -1.0 && *v <= 1.0), "{wf:?} out of range");
    }
}

#[test]
fn square_wave_takes_exactly_two_levels_sine_starts_at_zero() {
    let sq = Oscillator::new(Waveform::Square, 220.0, 44100).render(4410);
    let mut levels: Vec<f32> = Vec::new();
    for v in sq {
        if !levels.iter().any(|d| approx(*d, v, 1e-6)) {
            levels.push(v);
        }
    }
    assert_eq!(levels.len(), 2);

    let sine = Oscillator::new(Waveform::Sine, 440.0, 44100).render(10);
    assert!(approx(sine[0], 0.0, 1e-6)); // sin(0) == 0
}

#[test]
fn zero_samples_render_is_empty() {
    let osc = Oscillator::new(Waveform::Triangle, 100.0, 8000);
    assert!(osc.render(0).is_empty());
}

// ---- ADSR envelope --------------------------------------------------------

#[test]
fn envelope_rises_from_zero_holds_sustain_and_releases_to_zero() {
    let env = Adsr::new(0.1, 0.1, 0.7, 0.2);
    let note_len = 1.0;
    assert!(env.amplitude_at(0.0, note_len) < 0.05); // silent at attack start
    assert!(env.amplitude_at(0.05, note_len) > env.amplitude_at(0.0, note_len)); // rising
    // Sustain plateau: between end of decay and start of release.
    assert!(approx(env.amplitude_at(0.5, note_len), 0.7, 1e-3));
    assert!(env.amplitude_at(note_len - 0.001, note_len) < 0.05); // released
    assert_eq!(env.amplitude_at(note_len + 1.0, note_len), 0.0); // past the note
}

#[test]
fn apply_preserves_length_and_attenuates_the_edges() {
    let env = Adsr::new(0.1, 0.05, 0.8, 0.1);
    let raw = vec![1.0f32; 44100];
    let shaped = env.apply(&raw, 44100);
    assert_eq!(shaped.len(), raw.len());
    assert!(shaped[0].abs() < 0.05); // faded in
    assert!(shaped[shaped.len() - 1].abs() < 0.1); // faded out
    assert!(shaped[22050] > 0.5); // full-ish in the middle
}

// ---- sequencing and mixing ------------------------------------------------

#[test]
fn render_sequence_concatenates_note_durations() {
    let notes = vec![Note::new(440.0, 0.5), Note::new(220.0, 0.5)];
    let env = Adsr::new(0.01, 0.01, 0.8, 0.01);
    let s = render_sequence(&notes, Waveform::Sine, env, 44100);
    assert_eq!(s.len(), 44100); // 0.5s + 0.5s at 44.1kHz
    assert!(render_sequence(&[], Waveform::Sine, env, 44100).is_empty());
}

#[test]
fn mix_normalizes_peak_pads_shorter_and_leaves_quiet_mixes_untouched() {
    // Two hot tracks sum to 2.0 -> must be normalized back into range.
    let hot = mix_and_normalize(&[vec![1.0f32; 100], vec![1.0f32; 100]]);
    assert!(hot.iter().all(|s| (-1.0..=1.0).contains(s)));
    let peak = hot.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    assert!(approx(peak, 1.0, 1e-6));

    // Shorter track is padded with silence to the longest length.
    let padded = mix_and_normalize(&[vec![0.5f32; 10], vec![0.5f32; 5]]);
    assert_eq!(padded.len(), 10);
    assert!(approx(padded[9], 0.5, 1e-6)); // only the long track contributes here

    // A mix already within range is not rescaled.
    let quiet = mix_and_normalize(&[vec![0.3f32; 4], vec![0.2f32; 4]]);
    assert!(quiet.iter().all(|s| approx(*s, 0.5, 1e-6)));

    assert!(mix_and_normalize(&[]).is_empty());
}

// ---- WAV encoder: authoritative header + sample quantization --------------

fn read_u32(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(b[off..off + 4].try_into().unwrap())
}
fn read_u16(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(b[off..off + 2].try_into().unwrap())
}

#[test]
fn wav_bytes_produces_a_spec_correct_canonical_header() {
    let sr = 44100;
    let samples = vec![0.0f32; 1000];
    let b = wav_bytes(&samples, sr);

    assert_eq!(&b[0..4], b"RIFF");
    assert_eq!(read_u32(&b, 4), 36 + 1000 * 2); // RIFF size
    assert_eq!(&b[8..12], b"WAVE");
    assert_eq!(&b[12..16], b"fmt ");
    assert_eq!(read_u32(&b, 16), 16); // fmt chunk size
    assert_eq!(read_u16(&b, 20), 1); // PCM
    assert_eq!(read_u16(&b, 22), 1); // mono
    assert_eq!(read_u32(&b, 24), sr); // sample rate
    assert_eq!(read_u32(&b, 28), sr * 2); // byte rate = sr * channels * bytes/sample
    assert_eq!(read_u16(&b, 32), 2); // block align
    assert_eq!(read_u16(&b, 34), 16); // bits per sample
    assert_eq!(&b[36..40], b"data");
    assert_eq!(read_u32(&b, 40), 1000 * 2); // data size
    assert_eq!(b.len(), 44 + 1000 * 2);
}

#[test]
fn wav_bytes_quantizes_and_clamps_samples() {
    // Full-scale, silence, and out-of-range inputs.
    let b = wav_bytes(&[1.0, 0.0, -1.0, 2.0, -2.0], 8000);
    let decode = |i: usize| i16::from_le_bytes(b[44 + i * 2..44 + i * 2 + 2].try_into().unwrap());
    assert_eq!(decode(0), i16::MAX); // +1.0 -> 32767
    assert_eq!(decode(1), 0); // silence
    assert_eq!(decode(2), -i16::MAX); // -1.0 -> -32767
    assert_eq!(decode(3), i16::MAX); // clamped from +2.0
    assert_eq!(decode(4), -i16::MAX); // clamped from -2.0
}

#[test]
fn write_wav_round_trips_to_disk_and_reports_io_errors() {
    let samples = vec![0.25f32, -0.5, 0.75];
    let mut path = std::env::temp_dir();
    path.push(format!("reverb_it_{}.wav", std::process::id()));
    let p = path.to_str().unwrap();

    write_wav(p, &samples, 22050).expect("write should succeed");
    let on_disk = std::fs::read(p).unwrap();
    assert_eq!(on_disk, wav_bytes(&samples, 22050));
    let _ = std::fs::remove_file(p);

    // Writing under a directory that does not exist must surface an error,
    // not panic.
    let bad = write_wav("/no_such_dir_reverb_xyz/out.wav", &samples, 22050);
    assert!(bad.is_err());
}
