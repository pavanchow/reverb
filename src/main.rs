use clap::{Parser, Subcommand};
use reverb::{mix_and_normalize, note_to_freq, render_sequence, write_wav, Adsr, Note, Waveform};

const SAMPLE_RATE: u32 = 44100;

#[derive(Parser)]
#[command(name = "reverb", about = "A from-scratch audio synthesis engine")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a single steady tone to a WAV file.
    Tone {
        #[arg(long, default_value_t = 440.0)]
        freq: f32,
        #[arg(long, default_value_t = 2.0)]
        secs: f32,
        #[arg(long, value_enum, default_value = "sine")]
        wave: WaveArg,
        #[arg(short, long)]
        o: String,
    },
    /// Render a short built-in melody to a WAV file.
    Demo {
        #[arg(short, long)]
        o: String,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum WaveArg {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl From<WaveArg> for Waveform {
    fn from(w: WaveArg) -> Self {
        match w {
            WaveArg::Sine => Waveform::Sine,
            WaveArg::Square => Waveform::Square,
            WaveArg::Sawtooth => Waveform::Sawtooth,
            WaveArg::Triangle => Waveform::Triangle,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Tone {
            freq,
            secs,
            wave,
            o,
        } => {
            let env = Adsr::new(0.02, 0.05, 0.8, 0.1);
            let notes = [Note::new(freq, secs)];
            let samples = render_sequence(&notes, wave.into(), env, SAMPLE_RATE);
            write_wav(&o, &samples, SAMPLE_RATE).expect("failed to write WAV");
            println!("wrote {o} ({} samples)", samples.len());
        }
        Commands::Demo { o } => {
            let melody = ["C4", "E4", "G4", "C5", "G4", "E4", "C4"];
            let env = Adsr::new(0.02, 0.08, 0.7, 0.15);
            let notes: Vec<Note> = melody
                .iter()
                .map(|name| Note::new(note_to_freq(name).unwrap(), 0.35))
                .collect();

            let lead = render_sequence(&notes, Waveform::Triangle, env, SAMPLE_RATE);

            let bass_notes = [Note::new(note_to_freq("C3").unwrap(), 0.35 * 7.0)];
            let bass_env = Adsr::new(0.05, 0.1, 0.6, 0.3);
            let bass = render_sequence(&bass_notes, Waveform::Sine, bass_env, SAMPLE_RATE);

            let mixed = mix_and_normalize(&[lead, bass]);
            write_wav(&o, &mixed, SAMPLE_RATE).expect("failed to write WAV");
            println!("wrote {o} ({} samples)", mixed.len());
        }
    }
}
