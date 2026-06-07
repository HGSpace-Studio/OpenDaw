use crate::engine::{AppEngine, TrackPatch, UiOperationResult, UiProjectState, UiSoundFontEntry};
use crate::midi::MidiNote;
use crate::midi_keyboard::MidiInputPortInfo;
use std::env;
use std::sync::Mutex;
use tauri::State;

type SharedEngine = Mutex<AppEngine>;

#[derive(Debug, serde::Deserialize)]
struct LoadTrackArgs {
    path: String,
    gain: Option<f32>,
}

#[derive(Debug, serde::Deserialize)]
struct CreateTrackArgs {
    name: String,
}

#[derive(Debug, serde::Deserialize)]
struct SaveProjectArgs {
    path: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenProjectArgs {
    path: String,
}

#[derive(Debug, serde::Deserialize)]
struct ExportMixArgs {
    path: String,
}

#[derive(Debug, serde::Deserialize)]
struct PatchTrackArgs {
    index: usize,
    patch: TrackPatch,
}

#[derive(Debug, serde::Deserialize)]
struct SetTrackInstrumentArgs {
    index: usize,
    instrument_kind: String,
    soundfont_path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SetTrackNotesArgs {
    index: usize,
    notes: Vec<MidiNote>,
    duration_seconds: f32,
}

#[derive(Debug, serde::Deserialize)]
struct IndexArgs {
    index: usize,
}

#[derive(Debug, serde::Deserialize)]
struct ConnectKeyboardArgs {
    port_index: usize,
    track_index: usize,
}

#[derive(Debug, serde::Deserialize)]
struct StartRecordingArgs {
    track_index: usize,
}

#[derive(Debug, serde::Deserialize)]
struct LatencyArgs {
    mode: String,
}

#[derive(Debug, serde::Deserialize)]
struct PlayheadArgs {
    seconds: f32,
}

#[derive(Debug, serde::Deserialize)]
struct LoopArgs {
    enabled: bool,
    start_seconds: f32,
    end_seconds: f32,
}

#[tauri::command]
fn project_state(state: State<'_, SharedEngine>) -> Result<UiProjectState, String> {
    let engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    Ok(engine.project_state())
}

#[tauri::command]
fn load_track(
    args: LoadTrackArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.load_track(args.path, args.gain)
}

#[tauri::command]
fn create_midi_track(
    args: CreateTrackArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.create_midi_track(args.name)
}

#[tauri::command]
fn create_live_midi_track(
    args: CreateTrackArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.create_live_midi_track(args.name)
}

#[tauri::command]
fn duplicate_track(
    args: IndexArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.duplicate_track(args.index)
}

#[tauri::command]
fn remove_track(
    args: IndexArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.remove_track(args.index)
}

#[tauri::command]
fn clear_project(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    Ok(engine.clear())
}

#[tauri::command]
fn patch_track(
    args: PatchTrackArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.patch_track(args.index, args.patch)
}

#[tauri::command]
fn set_track_instrument(
    args: SetTrackInstrumentArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.set_track_instrument(args.index, args.instrument_kind, args.soundfont_path)
}

#[tauri::command]
fn set_track_notes(
    args: SetTrackNotesArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.set_track_notes(args.index, args.notes, args.duration_seconds)
}

#[tauri::command]
fn save_project(
    args: SaveProjectArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.save_project(args.path)
}

#[tauri::command]
fn open_project(
    args: OpenProjectArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.open_project(args.path)
}

#[tauri::command]
fn export_mix(
    args: ExportMixArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.mix_project(args.path)
}

#[tauri::command]
fn play(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.play()
}

#[tauri::command]
fn pause(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.pause()
}

#[tauri::command]
fn resume(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.resume()
}

#[tauri::command]
fn stop(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    Ok(engine.stop())
}

#[tauri::command]
fn set_latency_mode(
    args: LatencyArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.set_latency_mode(args.mode)
}

#[tauri::command]
fn set_playhead(
    args: PlayheadArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.set_playhead(args.seconds)
}

#[tauri::command]
fn set_loop(args: LoopArgs, state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.set_loop(args.enabled, args.start_seconds, args.end_seconds)
}

#[tauri::command]
fn list_keyboard_ports(state: State<'_, SharedEngine>) -> Result<Vec<MidiInputPortInfo>, String> {
    let engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.list_keyboard_ports()
}

#[tauri::command]
fn list_soundfonts(state: State<'_, SharedEngine>) -> Result<Vec<UiSoundFontEntry>, String> {
    let engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.list_soundfonts()
}

#[tauri::command]
fn connect_keyboard(
    args: ConnectKeyboardArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.connect_keyboard(args.port_index, args.track_index)
}

#[tauri::command]
fn disconnect_keyboard(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    Ok(engine.disconnect_keyboard())
}

#[tauri::command]
fn start_recording(
    args: StartRecordingArgs,
    state: State<'_, SharedEngine>,
) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.start_recording(args.track_index)
}

#[tauri::command]
fn stop_recording(state: State<'_, SharedEngine>) -> Result<UiOperationResult, String> {
    let mut engine = state
        .lock()
        .map_err(|_| "failed to lock engine state".to_string())?;
    engine.stop_recording(true)
}

#[tauri::command]
fn system_locale() -> String {
    for key in ["LC_ALL", "LANGUAGE", "LANG"] {
        if let Ok(raw) = env::var(key) {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return trimmed
                    .split('.')
                    .next()
                    .unwrap_or(trimmed)
                    .replace('_', "-");
            }
        }
    }

    "en".to_string()
}

pub fn run() -> Result<(), String> {
    tauri::Builder::default()
        .manage(Mutex::new(AppEngine::default()))
        .invoke_handler(tauri::generate_handler![
            project_state,
            load_track,
            create_midi_track,
            create_live_midi_track,
            duplicate_track,
            remove_track,
            clear_project,
            patch_track,
            set_track_instrument,
            set_track_notes,
            save_project,
            open_project,
            export_mix,
            play,
            pause,
            resume,
            stop,
            set_latency_mode,
            set_playhead,
            set_loop,
            list_keyboard_ports,
            list_soundfonts,
            connect_keyboard,
            disconnect_keyboard,
            start_recording,
            stop_recording,
            system_locale,
        ])
        .run(tauri::generate_context!())
        .map_err(|err| err.to_string())
}
