use crate::envelope::Adsr;
use crate::oscillator::{Oscillator, Waveform};

/// A single note: a frequency held for a duration, in seconds.
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub freq_hz: f32,
    pub duration_secs: f32,
}

impl Note {
    pub fn new(freq_hz: f32, duration_secs: f32) -> Self {
        Self {
            freq_hz,
            duration_secs,
        }
    }
}

/// Render a sequence of notes, back to back, into one sample buffer.
pub fn render_sequence(
    notes: &[Note],
    waveform: Waveform,
    envelope: Adsr,
    sample_rate: u32,
) -> Vec<f32> {
    let mut out = Vec::new();
    for note in notes {
        let num_samples = (note.duration_secs * sample_rate as f32).round() as usize;
        let osc = Oscillator::new(waveform, note.freq_hz, sample_rate);
        let raw = osc.render(num_samples);
        let shaped = envelope.apply(&raw, sample_rate);
        out.extend(shaped);
    }
    out
}

/// Mix multiple tracks by summing them sample-by-sample, padding shorter
/// tracks with silence, then normalize to fit within [-1, 1].
pub fn mix_and_normalize(tracks: &[Vec<f32>]) -> Vec<f32> {
    let max_len = tracks.iter().map(|t| t.len()).max().unwrap_or(0);
    let mut mixed = vec![0.0f32; max_len];
    for track in tracks {
        for (i, s) in track.iter().enumerate() {
            mixed[i] += s;
        }
    }

    let peak = mixed.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    if peak > 1.0 {
        for s in mixed.iter_mut() {
            *s /= peak;
        }
    }
    mixed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_sequence_concatenates_note_lengths() {
        let notes = vec![Note::new(440.0, 0.5), Note::new(220.0, 0.5)];
        let env = Adsr::new(0.01, 0.01, 0.8, 0.01);
        let samples = render_sequence(&notes, Waveform::Sine, env, 44100);
        assert_eq!(samples.len(), 44100);
    }

    #[test]
    fn mix_and_normalize_clamps_to_range() {
        let a = vec![1.0f32; 100];
        let b = vec![1.0f32; 100];
        let mixed = mix_and_normalize(&[a, b]);
        for s in mixed {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn mix_and_normalize_pads_shorter_tracks() {
        let a = vec![0.5f32; 10];
        let b = vec![0.5f32; 5];
        let mixed = mix_and_normalize(&[a, b]);
        assert_eq!(mixed.len(), 10);
    }
}
