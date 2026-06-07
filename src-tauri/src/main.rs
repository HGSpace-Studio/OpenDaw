mod engine;
mod midi;
mod midi_keyboard;
mod mixer;
mod player;
mod shell;
mod ui;

use std::env;
use std::path::Path;

use mixer::{mix_to_file, parse_track_spec};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        eprintln!();
        print_usage();
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = env::args().skip(1).collect::<Vec<_>>();

    if args.first().is_some_and(|arg| arg == "-ui") {
        return ui::run();
    }

    if args.first().is_some_and(|arg| arg == "-shell") {
        return shell::run_shell();
    }

    run_mix(&args)
}

fn run_mix(args: &[String]) -> Result<(), String> {
    let output = args
        .first()
        .ok_or_else(|| "missing output file".to_string())?;

    let track_specs = args[1..]
        .iter()
        .cloned()
        .map(parse_track_spec)
        .collect::<Result<Vec<_>, _>>()?;
    if track_specs.is_empty() {
        return Err("at least one input track is required".to_string());
    }

    let summary = mix_to_file(Path::new(output), &track_specs)?;
    println!(
        "mixed {} tracks into {} ({} Hz, {} ch, {} frames)",
        summary.track_count, output, summary.sample_rate, summary.channels, summary.frames
    );

    Ok(())
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  cargo run --release -- -ui");
    eprintln!("  cargo run --release -- -shell");
    eprintln!("  cargo run --release -- <output.wav> <input1.wav:gain> [input2.wav:gain] ...");
    eprintln!();
    eprintln!("examples:");
    eprintln!("  cargo run --release -- -ui");
    eprintln!("  cargo run --release -- -shell");
    eprintln!("  cargo run --release -- mixed.wav drums.wav:0.8 bass.wav:0.7 vocal.wav:1.0");
    eprintln!();
    eprintln!("notes:");
    eprintln!("  - `-ui` starts the Logic-style Tauri composer UI");
    eprintln!("  - `-shell` starts the interactive CLI shell");
    eprintln!("  - shell mode supports realtime playback, mute/solo, rename, offset, save/load");
    eprintln!("  - playback uses cpal for cross-platform output");
    eprintln!("  - supported inputs: 16-bit PCM WAV, 32-bit float WAV, and MIDI");
    eprintln!("  - all project tracks must share the same sample rate");
}
