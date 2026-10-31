use std::error::Error;
use std::fs::File;
use std::io::{self, BufWriter, Write};

const SAMPLE_RATE: u32 = 44_100;
const STEPS_PER_BAR: usize = 16;
const BARS: usize = 4;

#[derive(Clone, Copy)]
enum Sound {
    Kick,
    Snare,
    Hat,
    Bass,
}

struct Track {
    name: &'static str,
    sound: Sound,
    steps: [bool; STEPS_PER_BAR],
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut tracks = vec![
        track("kick", Sound::Kick, &[0, 4, 8, 12]),
        track("snare", Sound::Snare, &[4, 12]),
        track("hat", Sound::Hat, &[2, 6, 10, 14]),
        track("bass", Sound::Bass, &[0, 7, 10]),
    ];
    let mut bpm = 120_u16;

    println!("Rust DAW | {bpm} BPM | type 'help' for commands");
    show_pattern(&tracks);

    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        print!("daw> ");
        io::stdout().flush()?;
        input.clear();
        if stdin.read_line(&mut input)? == 0 {
            break;
        }

        let mut parts = input.split_whitespace();
        let Some(command) = parts.next() else {
            continue;
        };

        match command {
            "help" => print_help(),
            "show" => show_pattern(&tracks),
            "bpm" => match parts.next().and_then(|value| value.parse::<u16>().ok()) {
                Some(value) if (40..=240).contains(&value) => {
                    bpm = value;
                    println!("Tempo set to {bpm} BPM.");
                }
                _ => println!("Usage: bpm <40-240>"),
            },
            "toggle" => {
                let name = parts.next();
                let step = parts.next().and_then(|value| value.parse::<usize>().ok());
                match (name, step) {
                    (Some(name), Some(step)) if (1..=STEPS_PER_BAR).contains(&step) => {
                        if let Some(track) = tracks.iter_mut().find(|track| track.name == name) {
                            track.steps[step - 1] = !track.steps[step - 1];
                            show_pattern(&tracks);
                        } else {
                            println!("Unknown track '{name}'. Use kick, snare, hat, or bass.");
                        }
                    }
                    _ => println!("Usage: toggle <kick|snare|hat|bass> <1-16>"),
                }
            }
            "export" => match parts.next() {
                Some(path) => {
                    let samples = render(&tracks, bpm);
                    write_wav(path, &samples)?;
                    println!("Exported {} bars to {path}.", BARS);
                }
                None => println!("Usage: export <file.wav>"),
            },
            "quit" | "exit" => break,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    Ok(())
}

fn track(name: &'static str, sound: Sound, active_steps: &[usize]) -> Track {
    let mut steps = [false; STEPS_PER_BAR];
    for &step in active_steps {
        steps[step] = true;
    }
    Track { name, sound, steps }
}

fn print_help() {
    println!("Commands:");
    println!("  show                         Display the 16-step pattern");
    println!("  toggle <track> <step>        Toggle a step (1-16)");
    println!("  bpm <value>                  Set tempo (40-240)");
    println!("  export <file.wav>            Render four bars to a WAV file");
    println!("  quit                         Exit the sequencer");
}

fn show_pattern(tracks: &[Track]) {
    print!("       ");
    for step in 1..=STEPS_PER_BAR {
        print!("{:02} ", step);
    }
    println!();

    for track in tracks {
        print!("{:<6} ", track.name);
        for &active in &track.steps {
            print!("{}  ", if active { "x" } else { "." });
        }
        println!();
    }
}

fn render(tracks: &[Track], bpm: u16) -> Vec<i16> {
    let samples_per_step = f64::from(SAMPLE_RATE) * 60.0 / f64::from(bpm) / 4.0;
    let total_samples = (samples_per_step * (STEPS_PER_BAR * BARS) as f64) as usize;
    let mut output = vec![0_i16; total_samples];

    for sequence_step in 0..STEPS_PER_BAR * BARS {
        let start = (sequence_step as f64 * samples_per_step) as usize;
        let end = (((sequence_step + 1) as f64 * samples_per_step) as usize).min(total_samples);
        let pattern_step = sequence_step % STEPS_PER_BAR;

        for track in tracks.iter().filter(|track| track.steps[pattern_step]) {
            for sample_index in start..end {
                let time = (sample_index - start) as f32 / SAMPLE_RATE as f32;
                let value = synthesize(track.sound, time, sample_index as u32, pattern_step);
                let mixed = f32::from(output[sample_index]) / f32::from(i16::MAX) + value;
                output[sample_index] = (mixed.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16;
            }
        }
    }

    output
}

fn synthesize(sound: Sound, time: f32, seed: u32, pattern_step: usize) -> f32 {
    match sound {
        Sound::Kick => {
            let phase = 2.0 * std::f32::consts::PI
                * (48.0 * time + 7.0 * (1.0 - (-18.0 * time).exp()));
            0.9 * (-9.0 * time).exp() * phase.sin()
        }
        Sound::Snare => {
            let noise = noise_sample(seed);
            0.38 * (-16.0 * time).exp() * (0.7 * noise + 0.3 * (2.0 * std::f32::consts::PI * 190.0 * time).sin())
        }
        Sound::Hat => 0.14 * (-42.0 * time).exp() * noise_sample(seed),
        Sound::Bass => {
            let note = [55.0, 65.41, 73.42, 82.41][(pattern_step / 4) % 4];
            0.32 * (-5.0 * time).exp() * (2.0 * std::f32::consts::PI * note * time).sin()
        }
    }
}

fn noise_sample(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    value ^= value >> 13;
    (value as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn write_wav(path: &str, samples: &[i16]) -> Result<(), Box<dyn Error>> {
    let data_size = (samples.len() * 2) as u32;
    let mut writer = BufWriter::new(File::create(path)?);

    writer.write_all(b"RIFF")?;
    writer.write_all(&(36 + data_size).to_le_bytes())?;
    writer.write_all(b"WAVEfmt ")?;
    writer.write_all(&16_u32.to_le_bytes())?;
    writer.write_all(&1_u16.to_le_bytes())?;
    writer.write_all(&1_u16.to_le_bytes())?;
    writer.write_all(&SAMPLE_RATE.to_le_bytes())?;
    writer.write_all(&(SAMPLE_RATE * 2).to_le_bytes())?;
    writer.write_all(&2_u16.to_le_bytes())?;
    writer.write_all(&16_u16.to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_size.to_le_bytes())?;
    for sample in samples {
        writer.write_all(&sample.to_le_bytes())?;
    }
    writer.flush()?;

    Ok(())
}