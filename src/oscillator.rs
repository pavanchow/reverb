use std::f32::consts::PI;

/// Waveform shapes supported by the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

/// A single-frequency oscillator sampled at a fixed rate.
pub struct Oscillator {
    pub waveform: Waveform,
    pub freq_hz: f32,
    pub sample_rate: u32,
}

impl Oscillator {
    pub fn new(waveform: Waveform, freq_hz: f32, sample_rate: u32) -> Self {
        Self {
            waveform,
            freq_hz,
            sample_rate,
        }
    }

    /// Generate `num_samples` of the waveform, each in [-1.0, 1.0].
    pub fn render(&self, num_samples: usize) -> Vec<f32> {
        let mut out = Vec::with_capacity(num_samples);
        for n in 0..num_samples {
            let t = n as f32 / self.sample_rate as f32;
            // phase in [0, 1)
            let phase = (self.freq_hz * t).fract();
            out.push(sample_at_phase(self.waveform, phase));
        }
        out
    }
}

fn sample_at_phase(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (2.0 * PI * phase).sin(),
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        Waveform::Sawtooth => 2.0 * phase - 1.0,
        Waveform::Triangle => {
            // rises from -1 to 1 over the first half, falls back over the second half
            if phase < 0.5 {
                4.0 * phase - 1.0
            } else {
                3.0 - 4.0 * phase
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_440_one_second_sample_count_and_range() {
        let osc = Oscillator::new(Waveform::Sine, 440.0, 44100);
        let samples = osc.render(44100);
        assert_eq!(samples.len(), 44100);
        for s in &samples {
            assert!(*s >= -1.0 && *s <= 1.0, "sample out of range: {s}");
        }
    }

    #[test]
    fn square_wave_has_two_distinct_values() {
        let osc = Oscillator::new(Waveform::Square, 220.0, 44100);
        let samples = osc.render(4410);
        let mut distinct: Vec<f32> = Vec::new();
        for s in samples {
            if !distinct.iter().any(|d: &f32| (d - s).abs() < 1e-6) {
                distinct.push(s);
            }
        }
        assert_eq!(distinct.len(), 2);
    }
}
