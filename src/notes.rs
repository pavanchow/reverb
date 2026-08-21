/// Convert a note name like "A4", "C#3", "Db5" to a frequency in Hz using
/// twelve-tone equal temperament with A4 = 440.0.
pub fn note_to_freq(name: &str) -> Option<f32> {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let letter = bytes[0].to_ascii_uppercase();
    let semitone_from_c = match letter {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };

    let mut idx = 1;
    let mut accidental = 0i32;
    if idx < bytes.len() {
        match bytes[idx] {
            b'#' => {
                accidental = 1;
                idx += 1;
            }
            b'b' => {
                accidental = -1;
                idx += 1;
            }
            _ => {}
        }
    }

    let octave_str = &name[idx..];
    let octave: i32 = octave_str.parse().ok()?;

    let semitone = semitone_from_c + accidental;
    // MIDI note number, where C4 (middle C) has semitone_from_c=0, octave=4 -> midi 60
    let midi = (octave + 1) * 12 + semitone;
    let a4_midi = 69;
    let n = midi - a4_midi;
    Some(440.0 * 2f32.powf(n as f32 / 12.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a4_is_440() {
        let f = note_to_freq("A4").unwrap();
        assert!((f - 440.0).abs() < 0.01);
    }

    #[test]
    fn c4_is_middle_c() {
        let f = note_to_freq("C4").unwrap();
        assert!((f - 261.63).abs() < 0.1);
    }

    #[test]
    fn sharp_and_flat_neighbors_match() {
        let cs4 = note_to_freq("C#4").unwrap();
        let db4 = note_to_freq("Db4").unwrap();
        assert!((cs4 - db4).abs() < 0.01);
    }

    #[test]
    fn unknown_letter_is_none() {
        assert!(note_to_freq("H4").is_none());
    }
}
