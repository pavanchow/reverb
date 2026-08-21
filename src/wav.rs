use std::fs::File;
use std::io::{self, Write};

/// Write a mono 16-bit PCM WAV file by hand: RIFF header, fmt chunk, data chunk.
/// Samples are expected in [-1.0, 1.0] and are clamped before quantizing.
pub fn write_wav(path: &str, samples: &[f32], sample_rate: u32) -> io::Result<()> {
    let bytes = wav_bytes(samples, sample_rate);
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

/// Build the full byte contents of a mono 16-bit PCM WAV file.
pub fn wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_size = (samples.len() * (bits_per_sample as usize / 8)) as u32;
    let riff_size = 36 + data_size;

    let mut out = Vec::with_capacity(44 + data_size as usize);

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_size.to_le_bytes());
    out.extend_from_slice(b"WAVE");

    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    out.extend_from_slice(&num_channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());

    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    for s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let quantized = (clamped * i16::MAX as f32).round() as i16;
        out.extend_from_slice(&quantized.to_le_bytes());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_has_riff_wave_fmt_and_data() {
        let samples = vec![0.0f32; 100];
        let bytes = wav_bytes(&samples, 44100);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
    }

    #[test]
    fn data_chunk_length_matches_sample_count_and_bit_depth() {
        let samples = vec![0.0f32; 44100];
        let bytes = wav_bytes(&samples, 44100);
        let data_size = u32::from_le_bytes(bytes[40..44].try_into().unwrap());
        assert_eq!(data_size, 44100 * 2); // 16-bit = 2 bytes per sample
        assert_eq!(bytes.len(), 44 + data_size as usize);
    }

    #[test]
    fn bits_per_sample_field_is_16() {
        let samples = vec![0.0f32; 10];
        let bytes = wav_bytes(&samples, 44100);
        let bits = u16::from_le_bytes(bytes[34..36].try_into().unwrap());
        assert_eq!(bits, 16);
    }
}
