use crate::midi::{
    MidiClip, MidiInstrument, MidiNote, MidiSynthSettings, build_midi_clip_from_notes,
    is_midi_path, load_midi_file, render_midi_to_audio,
};
use crate::midi_keyboard::{
    KeyboardMessageListener, KeyboardSession, MidiInputPortInfo, connect_keyboard, list_input_ports,
};
use crate::mixer::{
    AudioFileInfo, LoadedAudio, RenderTrackSpec, load_audio_file, mix_render_tracks_to_file,
};
use crate::player::{LatencyMode, PlaybackSession, start_playback};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const RECORDED_MIDI_PATH: &str = "<recorded-midi>";
const SOUNDFONT_SCAN_DIR: &str = "./test/src";

#[derive(Debug, Clone)]
enum TrackSource {
    Audio(LoadedAudio),
    Midi {
        clip: MidiClip,
        instrument: MidiInstrument,
    },
    MidiLive {
        instrument: MidiInstrument,
    },
}

#[derive(Debug, Clone)]
struct Track {
    name: String,
    path: PathBuf,
    gain: f32,
    mute: bool,
    solo: bool,
    offset_seconds: f32,
    trim_start_seconds: f32,
    trim_end_seconds: f32,
    speed: f32,
    pitch_semitones: f32,
    source: TrackSource,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiTrack {
    pub index: usize,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub instrument: String,
    pub soundfont_path: Option<String>,
    pub gain: f32,
    pub mute: bool,
    pub solo: bool,
    pub offset_seconds: f32,
    pub trim_start_seconds: f32,
    pub trim_end_seconds: f32,
    pub speed: f32,
    pub pitch_semitones: f32,
    pub duration_seconds: f32,
    pub sample_rate: u32,
    pub channels: u16,
    pub notes: Vec<MidiNote>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiKeyboardRoute {
    pub port_index: usize,
    pub port_name: String,
    pub track_index: usize,
    pub track_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiPlaybackState {
    pub running: bool,
    pub paused: bool,
    pub position_seconds: f32,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiTransportState {
    pub playhead_seconds: f32,
    pub project_duration_seconds: f32,
    pub loop_enabled: bool,
    pub loop_start_seconds: f32,
    pub loop_end_seconds: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiProjectState {
    pub tracks: Vec<UiTrack>,
    pub latency_mode: String,
    pub playback: UiPlaybackState,
    pub transport: UiTransportState,
    pub keyboard_route: Option<UiKeyboardRoute>,
    pub recording: UiRecordingState,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackPatch {
    pub name: Option<String>,
    pub gain: Option<f32>,
    pub mute: Option<bool>,
    pub solo: Option<bool>,
    pub offset_seconds: Option<f32>,
    pub trim_start_seconds: Option<f32>,
    pub trim_end_seconds: Option<f32>,
    pub speed: Option<f32>,
    pub pitch_semitones: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiOperationResult {
    pub message: String,
    pub state: UiProjectState,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSoundFontEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiRecordingState {
    pub active: bool,
    pub track_index: Option<usize>,
    pub track_name: Option<String>,
    pub elapsed_seconds: f32,
    pub note_count: usize,
}

struct RecordingSession {
    track_index: usize,
    started_at: Instant,
    state: Arc<Mutex<RecordingState>>,
}

#[derive(Default)]
struct RecordingState {
    notes: Vec<MidiNote>,
    pending: HashMap<(u8, u8), Vec<(f32, u8)>>,
    max_time_seconds: f32,
}

pub struct AppEngine {
    tracks: Vec<Track>,
    playback: Option<PlaybackSession>,
    latency_mode: LatencyMode,
    keyboard: Option<KeyboardSession>,
    keyboard_track: Option<usize>,
    keyboard_port_index: Option<usize>,
    recording: Option<RecordingSession>,
    playhead_seconds: f32,
    loop_enabled: bool,
    loop_start_seconds: f32,
    loop_end_seconds: f32,
}

impl Default for AppEngine {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            playback: None,
            latency_mode: LatencyMode::Balanced,
            keyboard: None,
            keyboard_track: None,
            keyboard_port_index: None,
            recording: None,
            playhead_seconds: 0.0,
            loop_enabled: false,
            loop_start_seconds: 0.0,
            loop_end_seconds: 0.0,
        }
    }
}

impl AppEngine {
    pub fn project_state(&self) -> UiProjectState {
        let playhead_seconds = self.current_playhead_seconds();
        let project_duration_seconds = self.project_duration_seconds();
        let tracks = self
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                let info = info_from_track_source(&track.source);
                UiTrack {
                    index,
                    name: track.name.clone(),
                    path: track.path.display().to_string(),
                    kind: track_type_label(&track.source).to_string(),
                    instrument: instrument_label(&track.source).to_string(),
                    soundfont_path: track_midi_soundfont(&track.source),
                    gain: track.gain,
                    mute: track.mute,
                    solo: track.solo,
                    offset_seconds: track.offset_seconds,
                    trim_start_seconds: track.trim_start_seconds,
                    trim_end_seconds: track.trim_end_seconds,
                    speed: track.speed,
                    pitch_semitones: track.pitch_semitones,
                    duration_seconds: track_timeline_duration_seconds(track, &info),
                    sample_rate: info.sample_rate,
                    channels: info.channels,
                    notes: midi_notes(&track.source),
                }
            })
            .collect::<Vec<_>>();

        let playback = match &self.playback {
            Some(playback) => UiPlaybackState {
                running: !playback.is_finished(),
                paused: playback.is_paused(),
                position_seconds: playback.current_position_seconds(),
                sample_rate: playback.output_sample_rate,
                channels: playback.output_channels,
            },
            None => UiPlaybackState {
                running: false,
                paused: false,
                position_seconds: playhead_seconds,
                sample_rate: 0,
                channels: 0,
            },
        };

        let transport = UiTransportState {
            playhead_seconds,
            project_duration_seconds,
            loop_enabled: self.loop_enabled && self.loop_end_seconds > self.loop_start_seconds,
            loop_start_seconds: self.loop_start_seconds.clamp(0.0, project_duration_seconds),
            loop_end_seconds: self
                .loop_end_seconds
                .clamp(0.0, project_duration_seconds.max(self.loop_start_seconds)),
        };

        let keyboard_route = match (
            &self.keyboard,
            self.keyboard_track,
            self.keyboard_port_index,
        ) {
            (Some(session), Some(track_index), Some(port_index)) => Some(UiKeyboardRoute {
                port_index,
                port_name: session.port_name().to_string(),
                track_index,
                track_name: self
                    .tracks
                    .get(track_index)
                    .map(|track| track.name.clone())
                    .unwrap_or_else(|| "<missing>".to_string()),
            }),
            _ => None,
        };

        let recording = match &self.recording {
            Some(recording) => UiRecordingState {
                active: true,
                track_index: Some(recording.track_index),
                track_name: self
                    .tracks
                    .get(recording.track_index)
                    .map(|track| track.name.clone()),
                elapsed_seconds: recording.started_at.elapsed().as_secs_f32(),
                note_count: recording
                    .state
                    .lock()
                    .map(|state| state.notes.len())
                    .unwrap_or(0),
            },
            None => UiRecordingState {
                active: false,
                track_index: None,
                track_name: None,
                elapsed_seconds: 0.0,
                note_count: 0,
            },
        };

        UiProjectState {
            tracks,
            latency_mode: self.latency_mode.label().to_string(),
            playback,
            transport,
            keyboard_route,
            recording,
        }
    }

    pub fn load_track(
        &mut self,
        raw_path: impl AsRef<str>,
        gain: Option<f32>,
    ) -> Result<UiOperationResult, String> {
        let path = PathBuf::from(raw_path.as_ref().trim());
        if path.as_os_str().is_empty() {
            return Err("track path is empty".to_string());
        }

        let source = load_track_source(&path)?;
        let info = info_from_track_source(&source);
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("track")
            .to_string();
        self.stop_playback(false);
        let _ = self.stop_recording(false);

        self.tracks.push(Track {
            name: name.clone(),
            path,
            gain: gain.unwrap_or(1.0),
            mute: false,
            solo: false,
            offset_seconds: 0.0,
            trim_start_seconds: 0.0,
            trim_end_seconds: info.duration_seconds,
            speed: 1.0,
            pitch_semitones: 0.0,
            source,
        });

        Ok(self.result(format!(
            "loaded {} ({:.2}s, {} Hz, {} ch)",
            name, info.duration_seconds, info.sample_rate, info.channels
        )))
    }

    pub fn create_midi_track(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<UiOperationResult, String> {
        let trimmed = name.as_ref().trim();
        if trimmed.is_empty() {
            return Err("track name cannot be empty".to_string());
        }

        let clip = build_midi_clip_from_notes(Vec::new(), 2.0)?;
        self.stop_playback(false);
        let _ = self.stop_recording(false);
        self.tracks.push(Track {
            name: trimmed.to_string(),
            path: PathBuf::from(RECORDED_MIDI_PATH),
            gain: 1.0,
            mute: false,
            solo: false,
            offset_seconds: 0.0,
            trim_start_seconds: 0.0,
            trim_end_seconds: 2.0,
            speed: 1.0,
            pitch_semitones: 0.0,
            source: TrackSource::Midi {
                clip,
                instrument: MidiInstrument::Basic,
            },
        });

        Ok(self.result(format!("created MIDI track {}", trimmed)))
    }

    pub fn create_live_midi_track(
        &mut self,
        name: impl AsRef<str>,
    ) -> Result<UiOperationResult, String> {
        let trimmed = name.as_ref().trim();
        if trimmed.is_empty() {
            return Err("track name cannot be empty".to_string());
        }

        self.stop_playback(false);
        let _ = self.stop_recording(false);
        self.tracks.push(Track {
            name: trimmed.to_string(),
            path: PathBuf::from("<live-midi>"),
            gain: 1.0,
            mute: false,
            solo: false,
            offset_seconds: 0.0,
            trim_start_seconds: 0.0,
            trim_end_seconds: 2.0,
            speed: 1.0,
            pitch_semitones: 0.0,
            source: TrackSource::MidiLive {
                instrument: MidiInstrument::Basic,
            },
        });

        Ok(self.result(format!("created live MIDI track {}", trimmed)))
    }

    pub fn duplicate_track(&mut self, index: usize) -> Result<UiOperationResult, String> {
        let mut cloned = self.track(index)?.clone();
        cloned.name = format!("{} Copy", cloned.name);
        self.stop_playback(false);
        let _ = self.stop_recording(false);
        self.tracks.push(cloned);
        Ok(self.result(format!("duplicated track {}", index)))
    }

    pub fn remove_track(&mut self, index: usize) -> Result<UiOperationResult, String> {
        if index >= self.tracks.len() {
            return Err(format!("track index {} out of range", index));
        }

        if self.keyboard_track == Some(index) {
            self.disconnect_keyboard();
        } else if let Some(current) = self.keyboard_track {
            if index < current {
                self.keyboard_track = Some(current - 1);
            }
        }

        self.stop_playback(false);
        let _ = self.stop_recording(false);
        let removed = self.tracks.remove(index);
        Ok(self.result(format!("removed {}", removed.name)))
    }

    pub fn clear(&mut self) -> UiOperationResult {
        self.stop_playback(false);
        self.disconnect_keyboard();
        let _ = self.stop_recording(false);
        self.tracks.clear();
        self.result("cleared project".to_string())
    }

    pub fn patch_track(
        &mut self,
        index: usize,
        patch: TrackPatch,
    ) -> Result<UiOperationResult, String> {
        let track = self.track_mut(index)?;
        if let Some(name) = patch.name {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return Err("track name cannot be empty".to_string());
            }
            track.name = trimmed.to_string();
        }
        if let Some(gain) = patch.gain {
            track.gain = gain;
        }
        if let Some(mute) = patch.mute {
            track.mute = mute;
        }
        if let Some(solo) = patch.solo {
            track.solo = solo;
        }
        if let Some(offset_seconds) = patch.offset_seconds {
            if !offset_seconds.is_finite() || offset_seconds < 0.0 {
                return Err("offset_seconds must be finite and >= 0".to_string());
            }
            track.offset_seconds = offset_seconds;
        }
        if let Some(trim_start_seconds) = patch.trim_start_seconds {
            if !trim_start_seconds.is_finite() || trim_start_seconds < 0.0 {
                return Err("trim_start_seconds must be finite and >= 0".to_string());
            }
            track.trim_start_seconds = trim_start_seconds;
        }
        if let Some(trim_end_seconds) = patch.trim_end_seconds {
            if !trim_end_seconds.is_finite() || trim_end_seconds < 0.0 {
                return Err("trim_end_seconds must be finite and >= 0".to_string());
            }
            track.trim_end_seconds = trim_end_seconds;
        }
        if let Some(speed) = patch.speed {
            if !speed.is_finite() || speed <= 0.0 {
                return Err("speed must be a finite number > 0".to_string());
            }
            track.speed = speed;
        }
        if let Some(pitch) = patch.pitch_semitones {
            if !pitch.is_finite() {
                return Err("pitch_semitones must be finite".to_string());
            }
            track.pitch_semitones = pitch;
        }

        self.validate_track_bounds(index)?;
        let _ = self.stop_recording(false);
        self.restart_running_playback()?;
        Ok(self.result(format!("updated track {}", index)))
    }

    pub fn set_track_instrument(
        &mut self,
        index: usize,
        instrument_kind: impl AsRef<str>,
        soundfont_path: Option<String>,
    ) -> Result<UiOperationResult, String> {
        let instrument = match instrument_kind.as_ref() {
            "basic" => MidiInstrument::Basic,
            "soundfont" => {
                let path = PathBuf::from(
                    soundfont_path
                        .as_deref()
                        .ok_or_else(|| "soundfont path is required".to_string())?,
                );
                if !path.exists() {
                    return Err(format!("soundfont `{}` does not exist", path.display()));
                }
                MidiInstrument::SoundFont(path)
            }
            other => return Err(format!("unsupported instrument kind `{other}`")),
        };

        match &mut self.track_mut(index)?.source {
            TrackSource::Midi {
                instrument: slot, ..
            } => *slot = instrument,
            TrackSource::MidiLive { instrument: slot } => *slot = instrument,
            TrackSource::Audio(_) => {
                return Err(format!("track [{}] is not a MIDI track", index));
            }
        }

        self.restart_running_playback()?;
        let reconnect = self.keyboard_track == Some(index) && self.keyboard_port_index.is_some();
        let reconnect_port = self.keyboard_port_index;
        if reconnect {
            self.disconnect_keyboard();
            if let Some(port_index) = reconnect_port {
                let instrument = self.keyboard_instrument_for_track(index)?.clone();
                let session = connect_keyboard(port_index, &instrument)?;
                self.keyboard = Some(session);
                self.keyboard_track = Some(index);
                self.keyboard_port_index = Some(port_index);
            }
        }
        Ok(self.result(format!("set instrument for track {}", index)))
    }

    pub fn set_track_notes(
        &mut self,
        index: usize,
        notes: Vec<MidiNote>,
        duration_seconds: f32,
    ) -> Result<UiOperationResult, String> {
        if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
            return Err("duration_seconds must be a finite number > 0".to_string());
        }

        let track = self.track_mut(index)?;
        let instrument = match &track.source {
            TrackSource::Midi { instrument, .. } => instrument.clone(),
            TrackSource::MidiLive { instrument } => instrument.clone(),
            TrackSource::Audio(_) => {
                return Err(format!("track [{}] is not a MIDI track", index));
            }
        };

        let clip = build_midi_clip_from_notes(notes, duration_seconds)?;
        track.path = PathBuf::from(RECORDED_MIDI_PATH);
        track.source = TrackSource::Midi { clip, instrument };
        track.trim_start_seconds = 0.0;
        track.trim_end_seconds = duration_seconds;
        let _ = self.stop_recording(false);
        self.restart_running_playback()?;
        Ok(self.result(format!("updated MIDI notes for track {}", index)))
    }

    pub fn save_project(&self, project_path: impl AsRef<str>) -> Result<UiOperationResult, String> {
        let project_path = project_path.as_ref().trim().to_string();
        if project_path.is_empty() {
            return Err("project path is empty".to_string());
        }

        let project_base_dir = Path::new(&project_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let file = File::create(&project_path)
            .map_err(|err| format!("failed to create `{project_path}`: {err}"))?;
        let writer = BufWriter::new(file);

        let project = ProjectFile {
            version: 1,
            tracks: self
                .tracks
                .iter()
                .map(|track| ProjectTrackFile {
                    source_kind: project_track_source_kind(track).to_string(),
                    name: track.name.clone(),
                    path: project_track_path(&project_base_dir, track),
                    gain: track.gain,
                    mute: track.mute,
                    solo: track.solo,
                    offset_seconds: track.offset_seconds,
                    trim_start_seconds: track.trim_start_seconds,
                    trim_end_seconds: track.trim_end_seconds,
                    speed: track.speed,
                    pitch_semitones: track.pitch_semitones,
                    midi_soundfont: track_midi_soundfont(&track.source),
                    embedded_midi: embedded_midi_clip(track),
                })
                .collect(),
        };

        serde_json::to_writer_pretty(writer, &project)
            .map_err(|err| format!("failed to save `{project_path}`: {err}"))?;
        Ok(self.result(format!("saved project to {}", project_path)))
    }

    pub fn open_project(
        &mut self,
        project_path: impl AsRef<str>,
    ) -> Result<UiOperationResult, String> {
        let project_path = project_path.as_ref().trim().to_string();
        if project_path.is_empty() {
            return Err("project path is empty".to_string());
        }

        let file = File::open(&project_path)
            .map_err(|err| format!("failed to open `{project_path}`: {err}"))?;
        let reader = BufReader::new(file);
        let project: ProjectFile = serde_json::from_reader(reader)
            .map_err(|err| format!("failed to parse `{project_path}`: {err}"))?;
        if project.version != 1 {
            return Err(format!("unsupported project version {}", project.version));
        }

        let base_dir = Path::new(&project_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let mut loaded_tracks = Vec::with_capacity(project.tracks.len());
        for track in project.tracks {
            let (path, source) = load_project_track_source(&base_dir, &track)?;
            loaded_tracks.push(Track {
                name: track.name,
                path,
                gain: track.gain,
                mute: track.mute,
                solo: track.solo,
                offset_seconds: track.offset_seconds,
                trim_start_seconds: track.trim_start_seconds,
                trim_end_seconds: track.trim_end_seconds,
                speed: track.speed,
                pitch_semitones: track.pitch_semitones,
                source: apply_saved_instrument(source, track.midi_soundfont)?,
            });
        }

        self.stop_playback(false);
        self.disconnect_keyboard();
        let _ = self.stop_recording(false);
        self.tracks = loaded_tracks;
        self.playhead_seconds = 0.0;
        Ok(self.result(format!("loaded project from {}", project_path)))
    }

    pub fn mix_project(&self, output_path: impl AsRef<str>) -> Result<UiOperationResult, String> {
        let output = output_path.as_ref().trim();
        if output.is_empty() {
            return Err("output path is empty".to_string());
        }

        let render_specs = self.build_render_specs()?;
        let summary = mix_render_tracks_to_file(Path::new(output), &render_specs)?;
        Ok(self.result(format!(
            "mixed {} tracks into {} ({} Hz, {} ch, {} frames)",
            summary.track_count, output, summary.sample_rate, summary.channels, summary.frames
        )))
    }

    pub fn play(&mut self) -> Result<UiOperationResult, String> {
        self.stop_playback(false);
        let render_specs = self.build_render_specs()?;
        let playback = start_playback(
            &render_specs,
            self.latency_mode,
            self.playhead_seconds,
            self.playback_stop_seconds(),
        )?;
        self.playback = Some(playback);
        Ok(self.result("playback started".to_string()))
    }

    pub fn pause(&mut self) -> Result<UiOperationResult, String> {
        let playback = self
            .playback
            .as_mut()
            .ok_or_else(|| "playback is not running".to_string())?;
        playback.pause()?;
        Ok(self.result("playback paused".to_string()))
    }

    pub fn resume(&mut self) -> Result<UiOperationResult, String> {
        let playback = self
            .playback
            .as_mut()
            .ok_or_else(|| "playback is not running".to_string())?;
        playback.resume()?;
        Ok(self.result("playback resumed".to_string()))
    }

    pub fn stop(&mut self) -> UiOperationResult {
        self.stop_playback(false);
        self.result("playback stopped".to_string())
    }

    pub fn set_latency_mode(&mut self, mode: impl AsRef<str>) -> Result<UiOperationResult, String> {
        self.latency_mode = LatencyMode::parse(mode.as_ref())?;
        self.restart_running_playback()?;
        Ok(self.result(format!("latency mode set to {}", self.latency_mode.label())))
    }

    pub fn set_playhead(&mut self, seconds: f32) -> Result<UiOperationResult, String> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("playhead must be a finite number >= 0".to_string());
        }
        self.playhead_seconds = seconds.min(self.project_duration_seconds());
        self.restart_running_playback()?;
        Ok(self.result(format!("playhead set to {:.3}s", self.playhead_seconds)))
    }

    pub fn set_loop(
        &mut self,
        enabled: bool,
        start_seconds: f32,
        end_seconds: f32,
    ) -> Result<UiOperationResult, String> {
        if !start_seconds.is_finite() || !end_seconds.is_finite() || start_seconds < 0.0 {
            return Err("loop range must be finite and >= 0".to_string());
        }
        if enabled && end_seconds <= start_seconds {
            return Err("loop end must be greater than loop start".to_string());
        }

        let max_duration = self.project_duration_seconds();
        self.loop_enabled = enabled;
        self.loop_start_seconds = start_seconds.min(max_duration);
        self.loop_end_seconds = end_seconds.min(max_duration.max(self.loop_start_seconds));
        self.restart_running_playback()?;
        Ok(self.result(if enabled {
            format!(
                "loop enabled {:.3}s -> {:.3}s",
                self.loop_start_seconds, self.loop_end_seconds
            )
        } else {
            "loop disabled".to_string()
        }))
    }

    pub fn list_keyboard_ports(&self) -> Result<Vec<MidiInputPortInfo>, String> {
        list_input_ports()
    }

    pub fn list_soundfonts(&self) -> Result<Vec<UiSoundFontEntry>, String> {
        let mut entries = fs::read_dir(SOUNDFONT_SCAN_DIR)
            .map_err(|err| format!("failed to read `{SOUNDFONT_SCAN_DIR}`: {err}"))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                let extension = path.extension()?.to_str()?.to_ascii_lowercase();
                if extension != "sf2" {
                    return None;
                }
                let name = path.file_stem()?.to_str()?.to_string();
                Some(UiSoundFontEntry {
                    name,
                    path: path.to_string_lossy().into_owned(),
                })
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    pub fn connect_keyboard(
        &mut self,
        port_index: usize,
        track_index: usize,
    ) -> Result<UiOperationResult, String> {
        self.disconnect_keyboard();
        let instrument = self.keyboard_instrument_for_track(track_index)?.clone();
        let session = connect_keyboard(port_index, &instrument)?;
        self.keyboard = Some(session);
        self.keyboard_track = Some(track_index);
        self.keyboard_port_index = Some(port_index);
        Ok(self.result(format!("keyboard connected to track {}", track_index)))
    }

    pub fn disconnect_keyboard(&mut self) -> UiOperationResult {
        let _ = self.stop_recording(false);
        self.keyboard = None;
        self.keyboard_track = None;
        self.keyboard_port_index = None;
        self.result("keyboard disconnected".to_string())
    }

    pub fn start_recording(&mut self, track_index: usize) -> Result<UiOperationResult, String> {
        if self.recording.is_some() {
            return Err("recording is already running".to_string());
        }
        if self.keyboard_track != Some(track_index) || self.keyboard.is_none() {
            return Err(
                "recording requires the keyboard to be connected to the target track".to_string(),
            );
        }

        match self.track(track_index)?.source {
            TrackSource::MidiLive { .. } => {}
            TrackSource::Midi { .. } => {
                return Err("recording currently only supports `midi-live` tracks".to_string());
            }
            TrackSource::Audio(_) => {
                return Err(format!("track [{}] is not a MIDI track", track_index));
            }
        }

        let started_at = Instant::now();
        let state = Arc::new(Mutex::new(RecordingState::default()));
        let state_for_listener = state.clone();
        let listener: Arc<KeyboardMessageListener> = Arc::new(move |message: &[u8]| {
            record_midi_message(&state_for_listener, started_at, message);
        });

        if let Some(session) = &self.keyboard {
            session.set_message_listener(Some(listener));
        }

        self.recording = Some(RecordingSession {
            track_index,
            started_at,
            state,
        });
        Ok(self.result(format!("recording started on track {}", track_index)))
    }

    pub fn stop_recording(&mut self, print_message: bool) -> Result<UiOperationResult, String> {
        let Some(recording) = self.recording.take() else {
            return Ok(self.result(if print_message {
                "recording stopped".to_string()
            } else {
                "recording idle".to_string()
            }));
        };

        if let Some(session) = &self.keyboard {
            session.set_message_listener(None);
        }

        let elapsed = recording.started_at.elapsed().as_secs_f32().max(0.01);
        let (notes, note_count, duration_seconds) =
            finalize_recording_state(&recording.state, elapsed);

        if note_count == 0 {
            return Ok(self.result("recording stopped: no notes captured".to_string()));
        }

        let track = self.track_mut(recording.track_index)?;
        let instrument = match &track.source {
            TrackSource::MidiLive { instrument } => instrument.clone(),
            TrackSource::Midi { instrument, .. } => instrument.clone(),
            TrackSource::Audio(_) => {
                return Err("recording target is no longer a MIDI track".to_string());
            }
        };

        let clip = build_midi_clip_from_notes(notes, duration_seconds)?;
        track.path = PathBuf::from(RECORDED_MIDI_PATH);
        track.source = TrackSource::Midi { clip, instrument };
        track.trim_start_seconds = 0.0;
        track.trim_end_seconds = duration_seconds;

        Ok(self.result(format!(
            "recording stopped: captured {} notes into track {}",
            note_count, recording.track_index
        )))
    }

    fn result(&self, message: String) -> UiOperationResult {
        UiOperationResult {
            message,
            state: self.project_state(),
        }
    }

    fn track(&self, index: usize) -> Result<&Track, String> {
        self.tracks
            .get(index)
            .ok_or_else(|| format!("track index {} out of range", index))
    }

    fn track_mut(&mut self, index: usize) -> Result<&mut Track, String> {
        self.tracks
            .get_mut(index)
            .ok_or_else(|| format!("track index {} out of range", index))
    }

    fn build_render_specs(&self) -> Result<Vec<RenderTrackSpec>, String> {
        let output_sample_rate = self.project_sample_rate();
        let mut render_specs = Vec::new();

        for track in &self.tracks {
            let audio = match &track.source {
                TrackSource::Audio(audio) => Some(audio.clone()),
                TrackSource::Midi { clip, instrument } => Some(render_midi_to_audio(
                    clip,
                    MidiSynthSettings {
                        gain: 1.0,
                        speed: 1.0,
                        pitch_semitones: 0.0,
                        output_sample_rate,
                        output_channels: 2,
                    },
                    instrument,
                )?),
                TrackSource::MidiLive { .. } => None,
            };

            if let Some(audio) = audio {
                let displayed_duration = track_timeline_duration_seconds(
                    track,
                    &AudioFileInfo {
                        sample_rate: audio.sample_rate,
                        channels: audio.channels,
                        duration_seconds: audio.duration_seconds,
                    },
                );
                render_specs.push(RenderTrackSpec {
                    audio,
                    gain: track.gain,
                    mute: track.mute,
                    solo: track.solo,
                    offset_frames: seconds_to_frames(track.offset_seconds, output_sample_rate),
                    clip_start_frames: seconds_to_frames(
                        track.trim_start_seconds,
                        output_sample_rate,
                    ),
                    clip_frame_count: seconds_to_frames(displayed_duration, output_sample_rate),
                    speed: track.speed,
                    pitch_semitones: track.pitch_semitones,
                });
            }
        }

        if render_specs.is_empty() {
            return Err("no renderable tracks loaded".to_string());
        }

        Ok(render_specs)
    }

    fn project_sample_rate(&self) -> u32 {
        self.tracks
            .iter()
            .find_map(|track| match &track.source {
                TrackSource::Audio(audio) => Some(audio.sample_rate),
                TrackSource::Midi { .. } | TrackSource::MidiLive { .. } => None,
            })
            .unwrap_or(44_100)
    }

    fn keyboard_instrument_for_track(&self, track_index: usize) -> Result<&MidiInstrument, String> {
        match &self.track(track_index)?.source {
            TrackSource::Midi { instrument, .. } => Ok(instrument),
            TrackSource::MidiLive { instrument } => Ok(instrument),
            TrackSource::Audio(_) => Err(format!("track [{}] is not a MIDI track", track_index)),
        }
    }

    fn validate_track_bounds(&self, index: usize) -> Result<(), String> {
        let track = self.track(index)?;
        let info = info_from_track_source(&track.source);
        let base_duration = transformed_source_duration_seconds(info.duration_seconds, track.speed);
        if track.trim_start_seconds > base_duration {
            return Err("trim start exceeds clip length".to_string());
        }
        if track.trim_end_seconds <= track.trim_start_seconds {
            return Err("trim end must be greater than trim start".to_string());
        }
        Ok(())
    }

    fn current_playhead_seconds(&self) -> f32 {
        match &self.playback {
            Some(playback) => playback.current_position_seconds(),
            None => self.playhead_seconds,
        }
    }

    fn project_duration_seconds(&self) -> f32 {
        self.tracks
            .iter()
            .map(track_timeline_end_seconds)
            .fold(0.0_f32, f32::max)
            .max(2.0)
    }

    fn playback_stop_seconds(&self) -> Option<f32> {
        if self.loop_enabled && self.loop_end_seconds > self.loop_start_seconds {
            Some(self.loop_end_seconds)
        } else {
            None
        }
    }

    fn restart_running_playback(&mut self) -> Result<(), String> {
        let Some(playback) = &self.playback else {
            return Ok(());
        };
        if playback.is_paused() || playback.is_finished() {
            self.playhead_seconds = playback.current_position_seconds();
            self.stop_playback(false);
            return Ok(());
        }

        let current_position = playback.current_position_seconds();
        self.stop_playback(false);
        self.playhead_seconds = current_position.min(self.project_duration_seconds());
        let render_specs = self.build_render_specs()?;
        let playback = start_playback(
            &render_specs,
            self.latency_mode,
            self.playhead_seconds,
            self.playback_stop_seconds(),
        )?;
        self.playback = Some(playback);
        Ok(())
    }

    fn stop_playback(&mut self, _print: bool) {
        if let Some(playback) = &self.playback {
            self.playhead_seconds = playback.current_position_seconds();
        }
        let _ = self.playback.take();
    }
}

fn record_midi_message(state: &Arc<Mutex<RecordingState>>, started_at: Instant, message: &[u8]) {
    if message.is_empty() {
        return;
    }

    let status = message[0];
    let command = status & 0xF0;
    if command != 0x80 && command != 0x90 {
        return;
    }

    let channel = status & 0x0F;
    let note = *message.get(1).unwrap_or(&0);
    let velocity = *message.get(2).unwrap_or(&0);
    let now_seconds = started_at.elapsed().as_secs_f32().max(0.0);

    let Ok(mut state) = state.lock() else {
        return;
    };
    state.max_time_seconds = state.max_time_seconds.max(now_seconds);

    match command {
        0x90 if velocity > 0 => {
            state
                .pending
                .entry((channel, note))
                .or_default()
                .push((now_seconds, velocity));
        }
        0x80 | 0x90 => {
            let mut recorded_note = None;
            let mut remove_entry = false;
            if let Some(stack) = state.pending.get_mut(&(channel, note)) {
                if let Some((start_seconds, start_velocity)) = stack.pop() {
                    recorded_note = Some(MidiNote {
                        note,
                        velocity: start_velocity,
                        channel,
                        start_seconds,
                        end_seconds: now_seconds.max(start_seconds),
                    });
                }
                remove_entry = stack.is_empty();
            }
            if remove_entry {
                state.pending.remove(&(channel, note));
            }
            if let Some(note) = recorded_note {
                state.notes.push(note);
            }
        }
        _ => {}
    }
}

fn finalize_recording_state(
    state: &Arc<Mutex<RecordingState>>,
    stop_time_seconds: f32,
) -> (Vec<MidiNote>, usize, f32) {
    let Ok(mut state) = state.lock() else {
        return (Vec::new(), 0, stop_time_seconds.max(0.01));
    };

    let keys: Vec<(u8, u8)> = state.pending.keys().copied().collect();
    for key in keys {
        if let Some(mut stack) = state.pending.remove(&key) {
            while let Some((start_seconds, velocity)) = stack.pop() {
                state.notes.push(MidiNote {
                    note: key.1,
                    velocity,
                    channel: key.0,
                    start_seconds,
                    end_seconds: stop_time_seconds.max(start_seconds),
                });
            }
        }
    }

    state
        .notes
        .sort_by(|left, right| left.start_seconds.total_cmp(&right.start_seconds));
    let note_count = state.notes.len();
    let duration_seconds = state.max_time_seconds.max(stop_time_seconds).max(
        state
            .notes
            .iter()
            .map(|note| note.end_seconds)
            .fold(0.0_f32, f32::max),
    );

    (state.notes.clone(), note_count, duration_seconds)
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    tracks: Vec<ProjectTrackFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectTrackFile {
    #[serde(default = "default_source_kind")]
    source_kind: String,
    name: String,
    path: String,
    gain: f32,
    mute: bool,
    solo: bool,
    offset_seconds: f32,
    #[serde(default)]
    trim_start_seconds: f32,
    #[serde(default = "default_trim_end")]
    trim_end_seconds: f32,
    #[serde(default = "default_speed")]
    speed: f32,
    #[serde(default)]
    pitch_semitones: f32,
    #[serde(default)]
    midi_soundfont: Option<String>,
    #[serde(default)]
    embedded_midi: Option<EmbeddedMidiClipFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddedMidiClipFile {
    duration_seconds: f32,
    notes: Vec<MidiNote>,
}

fn load_track_source(path: &Path) -> Result<TrackSource, String> {
    if is_midi_path(path) {
        return Ok(TrackSource::Midi {
            clip: load_midi_file(path)?,
            instrument: MidiInstrument::Basic,
        });
    }
    Ok(TrackSource::Audio(load_audio_file(path)?))
}

fn load_project_track_source(
    base_dir: &Path,
    track: &ProjectTrackFile,
) -> Result<(PathBuf, TrackSource), String> {
    match track.source_kind.as_str() {
        "midi_live" => Ok((
            PathBuf::from("<live-midi>"),
            TrackSource::MidiLive {
                instrument: saved_instrument(track.midi_soundfont.clone()),
            },
        )),
        "midi_embedded" => {
            let embedded = track
                .embedded_midi
                .as_ref()
                .ok_or_else(|| "embedded MIDI track is missing note data".to_string())?;
            Ok((
                PathBuf::from(RECORDED_MIDI_PATH),
                TrackSource::Midi {
                    clip: build_midi_clip_from_notes(
                        embedded.notes.clone(),
                        embedded.duration_seconds,
                    )?,
                    instrument: saved_instrument(track.midi_soundfont.clone()),
                },
            ))
        }
        "midi" | "audio" => {
            let path = resolve_project_path(base_dir, &track.path);
            let source =
                apply_saved_instrument(load_track_source(&path)?, track.midi_soundfont.clone())?;
            Ok((path, source))
        }
        other => Err(format!("unsupported track source kind `{other}`")),
    }
}

fn resolve_project_path(base_dir: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() || path.exists() {
        path
    } else {
        base_dir.join(path)
    }
}

fn default_speed() -> f32 {
    1.0
}

fn default_trim_end() -> f32 {
    2.0
}

fn default_source_kind() -> String {
    "audio".to_string()
}

fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds.max(0.0) * sample_rate as f32).round() as usize
}

fn make_project_path(base_dir: &Path, path: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn project_track_path(base_dir: &Path, track: &Track) -> String {
    match &track.source {
        TrackSource::MidiLive { .. } => String::new(),
        TrackSource::Midi { .. } if is_embedded_midi_track(track) => String::new(),
        _ => make_project_path(base_dir, &track.path),
    }
}

fn is_embedded_midi_track(track: &Track) -> bool {
    matches!(track.source, TrackSource::Midi { .. }) && track.path == Path::new(RECORDED_MIDI_PATH)
}

fn track_source_kind(source: &TrackSource) -> &'static str {
    match source {
        TrackSource::Audio(_) => "audio",
        TrackSource::Midi { .. } => "midi",
        TrackSource::MidiLive { .. } => "midi_live",
    }
}

fn project_track_source_kind(track: &Track) -> &'static str {
    if is_embedded_midi_track(track) {
        "midi_embedded"
    } else {
        track_source_kind(&track.source)
    }
}

fn embedded_midi_clip(track: &Track) -> Option<EmbeddedMidiClipFile> {
    match &track.source {
        TrackSource::Midi { clip, .. } if is_embedded_midi_track(track) => {
            Some(EmbeddedMidiClipFile {
                duration_seconds: clip.duration_seconds,
                notes: clip.notes.clone(),
            })
        }
        _ => None,
    }
}

fn saved_instrument(midi_soundfont: Option<String>) -> MidiInstrument {
    match midi_soundfont {
        Some(path) => MidiInstrument::SoundFont(PathBuf::from(path)),
        None => MidiInstrument::Basic,
    }
}

fn apply_saved_instrument(
    source: TrackSource,
    midi_soundfont: Option<String>,
) -> Result<TrackSource, String> {
    match source {
        TrackSource::Midi { clip, .. } => Ok(TrackSource::Midi {
            clip,
            instrument: saved_instrument(midi_soundfont),
        }),
        TrackSource::MidiLive { .. } => Ok(TrackSource::MidiLive {
            instrument: saved_instrument(midi_soundfont),
        }),
        other => Ok(other),
    }
}

fn track_type_label(source: &TrackSource) -> &'static str {
    match source {
        TrackSource::Audio(_) => "audio",
        TrackSource::Midi { .. } => "midi",
        TrackSource::MidiLive { .. } => "midi-live",
    }
}

fn instrument_label(source: &TrackSource) -> &'static str {
    match source {
        TrackSource::Audio(_) => "audio",
        TrackSource::Midi {
            instrument: MidiInstrument::Basic,
            ..
        }
        | TrackSource::MidiLive {
            instrument: MidiInstrument::Basic,
        } => "basic",
        TrackSource::Midi {
            instrument: MidiInstrument::SoundFont(_),
            ..
        }
        | TrackSource::MidiLive {
            instrument: MidiInstrument::SoundFont(_),
        } => "soundfont",
    }
}

fn track_midi_soundfont(source: &TrackSource) -> Option<String> {
    match source {
        TrackSource::Midi {
            instrument: MidiInstrument::SoundFont(path),
            ..
        }
        | TrackSource::MidiLive {
            instrument: MidiInstrument::SoundFont(path),
        } => Some(path.to_string_lossy().into_owned()),
        _ => None,
    }
}

fn midi_notes(source: &TrackSource) -> Vec<MidiNote> {
    match source {
        TrackSource::Midi { clip, .. } => clip.notes.clone(),
        _ => Vec::new(),
    }
}

fn info_from_track_source(source: &TrackSource) -> AudioFileInfo {
    match source {
        TrackSource::Audio(audio) => AudioFileInfo {
            sample_rate: audio.sample_rate,
            channels: audio.channels,
            duration_seconds: audio.duration_seconds,
        },
        TrackSource::Midi { clip, .. } => AudioFileInfo {
            sample_rate: 44_100,
            channels: 2,
            duration_seconds: clip.duration_seconds,
        },
        TrackSource::MidiLive { .. } => AudioFileInfo {
            sample_rate: 44_100,
            channels: 2,
            duration_seconds: 0.0,
        },
    }
}

fn transformed_source_duration_seconds(source_duration_seconds: f32, speed: f32) -> f32 {
    if speed <= 0.0 {
        source_duration_seconds
    } else {
        source_duration_seconds / speed
    }
}

fn track_timeline_duration_seconds(track: &Track, info: &AudioFileInfo) -> f32 {
    let base_duration = transformed_source_duration_seconds(info.duration_seconds, track.speed);
    let start = track.trim_start_seconds.clamp(0.0, base_duration);
    let end = track
        .trim_end_seconds
        .clamp(start, base_duration.max(start));
    (end - start).max(0.001)
}

fn track_timeline_end_seconds(track: &Track) -> f32 {
    let info = info_from_track_source(&track.source);
    track.offset_seconds + track_timeline_duration_seconds(track, &info)
}
