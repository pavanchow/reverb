/// Attack-Decay-Sustain-Release envelope, all times in seconds, sustain as a level in [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct Adsr {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Adsr {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
    }

    /// Amplitude multiplier at `t` seconds into a note that lasts `note_len` seconds total,
    /// including the release tail, so the note plays for `note_len - release` seconds
    /// before releasing.
    pub fn amplitude_at(&self, t: f32, note_len: f32) -> f32 {
        let sustain_end = (note_len - self.release).max(0.0);
        if t < self.attack {
            if self.attack <= 0.0 {
                1.0
            } else {
                t / self.attack
            }
        } else if t < self.attack + self.decay {
            let dt = t - self.attack;
            let frac = if self.decay <= 0.0 { 1.0 } else { dt / self.decay };
            1.0 - frac * (1.0 - self.sustain)
        } else if t < sustain_end {
            self.sustain
        } else if t < note_len {
            let dt = t - sustain_end;
            let frac = if self.release <= 0.0 {
                1.0
            } else {
                dt / self.release
            };
            self.sustain * (1.0 - frac).max(0.0)
        } else {
            0.0
        }
    }

    /// Apply the envelope to a buffer of raw oscillator samples, one sample per frame.
    pub fn apply(&self, samples: &[f32], sample_rate: u32) -> Vec<f32> {
        let note_len = samples.len() as f32 / sample_rate as f32;
        samples
            .iter()
            .enumerate()
            .map(|(n, s)| {
                let t = n as f32 / sample_rate as f32;
                s * self.amplitude_at(t, note_len)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_starts_near_zero_rises_then_ends_near_zero() {
        let env = Adsr::new(0.1, 0.1, 0.7, 0.2);
        let note_len = 1.0;

        let start = env.amplitude_at(0.0, note_len);
        assert!(start < 0.05, "start amplitude too high: {start}");

        let mid_attack = env.amplitude_at(0.09, note_len);
        assert!(mid_attack > start, "envelope did not rise during attack");

        let end = env.amplitude_at(note_len - 0.001, note_len);
        assert!(end < 0.05, "end amplitude too high: {end}");
    }

    #[test]
    fn apply_scales_samples_down_at_edges() {
        let env = Adsr::new(0.1, 0.05, 0.8, 0.1);
        let samples = vec![1.0f32; 44100];
        let shaped = env.apply(&samples, 44100);
        assert!(shaped[0].abs() < 0.05);
        assert!(shaped[shaped.len() - 1].abs() < 0.1);
    }
}
