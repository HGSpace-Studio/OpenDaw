use crate::midi::{
    ActiveMidiNote, MidiClip, MidiInstrument, MidiNote, MidiSynthSettings, active_notes_at,
    build_midi_clip_from_notes, is_midi_path, load_midi_file, render_midi_to_audio,
};
use crate::midi_keyboard::{
    KeyboardMessageListener, KeyboardMonitorSession, KeyboardSession, connect_keyboard,
    list_input_ports, monitor_keyboard_input,
};
use crate::mixer::{
    AudioFileInfo, LoadedAudio, RenderTrackSpec, load_audio_file, mix_render_tracks_to_file,
};
use crate::player::{LatencyMode, PlaybackSession, start_playback};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const RECORDED_MIDI_PATH: &str = "<recorded-midi>";

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
    speed: f32,
    pitch_semitones: f32,
    source: TrackSource,
}

pub fn run_shell() -> Result<(), String> {
    let mut shell = TrackShell::default();

    println!("OpenDAW shell");
    println!("type `help` for commands");

    loop {
        print!("opendaw> ");
        io::stdout()
            .flush()
            .map_err(|err| format!("failed to flush stdout: {err}"))?;

        let mut line = String::new();
        let bytes = io::stdin()
            .read_line(&mut line)
            .map_err(|err| format!("failed to read input: {err}"))?;

        if bytes == 0 {
            println!();
            let _ = shell.stop_recording(false);
            shell.stop_playback(false);
            shell.disconnect_keyboard(false);
            shell.stop_keyboard_monitor(false);
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match shell.handle_command(line) {
            Ok(ShellControl::Continue) => {}
            Ok(ShellControl::Exit) => {
                let _ = shell.stop_recording(false);
                shell.stop_playback(false);
                shell.disconnect_keyboard(false);
                shell.stop_keyboard_monitor(false);
                break;
            }
            Err(err) => eprintln!("error: {err}"),
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellControl {
    Continue,
    Exit,
}

#[derive(Default)]
struct TrackShell {
    tracks: Vec<Track>,
    playback: Option<PlaybackSession>,
    keyboard: Option<KeyboardSession>,
    keyboard_track: Option<usize>,
    keyboard_port_index: Option<usize>,
    keyboard_monitor: Option<KeyboardMonitorSession>,
    recording: Option<RecordingSession>,
    latency_mode: LatencyMode,
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

impl TrackShell {
    fn handle_command(&mut self, line: &str) -> Result<ShellControl, String> {
        let mut parts = line.split_whitespace();
        let command = parts.next().ok_or_else(|| "empty command".to_string())?;

        match command {
            "help" => {
                self.print_help();
                Ok(ShellControl::Continue)
            }
            "load" => {
                let path = parts
                    .next()
                    .ok_or_else(|| "usage: load <path.wav|path.mid> [gain]".to_string())?;
                let gain = match parts.next() {
                    Some(raw) => raw
                        .parse::<f32>()
                        .map_err(|_| format!("invalid gain `{raw}`"))?,
                    None => 1.0,
                };
                self.load_track(path, gain)?;
                Ok(ShellControl::Continue)
            }
            "midi-live" => {
                let mut pieces = line.splitn(2, ' ');
                let _ = pieces.next();
                let name = pieces.next().unwrap_or("live_midi");
                self.create_live_midi_track(name)?;
                Ok(ShellControl::Continue)
            }
            "soundfont" => {
                let index = parse_index(parts.next().ok_or_else(|| {
                    "usage: soundfont <track_index> <path.sf2|basic>".to_string()
                })?)?;
                let value = parts
                    .next()
                    .ok_or_else(|| "usage: soundfont <track_index> <path.sf2|basic>".to_string())?;
                self.set_soundfont(index, value)?;
                Ok(ShellControl::Continue)
            }
            "keyboard" => {
                let action = parts.next().ok_or_else(|| {
                    "usage: keyboard <list|connect|disconnect|status|monitor> ...".to_string()
                })?;
                match action {
                    "list" => self.list_keyboard_ports()?,
                    "connect" => {
                        let port_index = parse_index(parts.next().ok_or_else(|| {
                            "usage: keyboard connect <port_index> <track_index>".to_string()
                        })?)?;
                        let track_index = parse_index(parts.next().ok_or_else(|| {
                            "usage: keyboard connect <port_index> <track_index>".to_string()
                        })?)?;
                        self.connect_keyboard_to_track(port_index, track_index)?;
                    }
                    "disconnect" => self.disconnect_keyboard(true),
                    "status" => self.print_keyboard_status(),
                    "monitor" => {
                        let value = parts.next().ok_or_else(|| {
                            "usage: keyboard monitor <port_index|stop|status>".to_string()
                        })?;
                        match value {
                            "stop" => self.stop_keyboard_monitor(true),
                            "status" => self.print_keyboard_monitor_status(),
                            _ => self.start_keyboard_monitor(parse_index(value)?)?,
                        }
                    }
                    _ => {
                        return Err(format!(
                            "unknown keyboard action `{action}`; use list, connect, disconnect, status, or monitor"
                        ));
                    }
                }
                Ok(ShellControl::Continue)
            }
            "record" => {
                let action = parts
                    .next()
                    .ok_or_else(|| "usage: record <start|stop|status> ...".to_string())?;
                match action {
                    "start" => {
                        let track_index = match parts.next() {
                            Some(raw) => parse_index(raw)?,
                            None => self
                                .keyboard_track
                                .ok_or_else(|| "usage: record start <track_index>".to_string())?,
                        };
                        self.start_recording(track_index)?;
                    }
                    "stop" => self.stop_recording(true)?,
                    "status" => self.print_recording_status(),
                    _ => {
                        return Err(format!(
                            "unknown record action `{action}`; use start, stop, or status"
                        ));
                    }
                }
                Ok(ShellControl::Continue)
            }
            "list" => {
                self.list_tracks();
                Ok(ShellControl::Continue)
            }
            "gain" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: gain <track_index> <gain>".to_string())?,
                )?;
                let gain = parts
                    .next()
                    .ok_or_else(|| "usage: gain <track_index> <gain>".to_string())?
                    .parse::<f32>()
                    .map_err(|_| "invalid gain".to_string())?;
                self.set_gain(index, gain)?;
                Ok(ShellControl::Continue)
            }
            "mute" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: mute <track_index> [on|off|toggle]".to_string())?,
                )?;
                let action = parts.next().unwrap_or("toggle");
                self.set_toggle(index, action, ToggleField::Mute)?;
                Ok(ShellControl::Continue)
            }
            "solo" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: solo <track_index> [on|off|toggle]".to_string())?,
                )?;
                let action = parts.next().unwrap_or("toggle");
                self.set_toggle(index, action, ToggleField::Solo)?;
                Ok(ShellControl::Continue)
            }
            "rename" => {
                let mut pieces = line.splitn(3, ' ');
                let _ = pieces.next();
                let index = parse_index(
                    pieces
                        .next()
                        .ok_or_else(|| "usage: rename <track_index> <new_name>".to_string())?,
                )?;
                let name = pieces
                    .next()
                    .ok_or_else(|| "usage: rename <track_index> <new_name>".to_string())?;
                self.rename_track(index, name)?;
                Ok(ShellControl::Continue)
            }
            "offset" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: offset <track_index> <seconds>".to_string())?,
                )?;
                let seconds = parts
                    .next()
                    .ok_or_else(|| "usage: offset <track_index> <seconds>".to_string())?
                    .parse::<f32>()
                    .map_err(|_| "invalid offset seconds".to_string())?;
                self.set_offset(index, seconds)?;
                Ok(ShellControl::Continue)
            }
            "speed" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: speed <track_index> <ratio>".to_string())?,
                )?;
                let speed = parts
                    .next()
                    .ok_or_else(|| "usage: speed <track_index> <ratio>".to_string())?
                    .parse::<f32>()
                    .map_err(|_| "invalid speed ratio".to_string())?;
                self.set_speed(index, speed)?;
                Ok(ShellControl::Continue)
            }
            "pitch" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: pitch <track_index> <semitones>".to_string())?,
                )?;
                let semitones = parts
                    .next()
                    .ok_or_else(|| "usage: pitch <track_index> <semitones>".to_string())?
                    .parse::<f32>()
                    .map_err(|_| "invalid semitone value".to_string())?;
                self.set_pitch(index, semitones)?;
                Ok(ShellControl::Continue)
            }
            "notes" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: notes <track_index> <seconds>".to_string())?,
                )?;
                let seconds = parts
                    .next()
                    .ok_or_else(|| "usage: notes <track_index> <seconds>".to_string())?
                    .parse::<f32>()
                    .map_err(|_| "invalid seconds".to_string())?;
                self.print_notes_at(index, seconds)?;
                Ok(ShellControl::Continue)
            }
            "remove" => {
                let index = parse_index(
                    parts
                        .next()
                        .ok_or_else(|| "usage: remove <track_index>".to_string())?,
                )?;
                self.remove_track(index)?;
                Ok(ShellControl::Continue)
            }
            "clear" => {
                let _ = self.stop_recording(false);
                self.stop_playback(true);
                self.disconnect_keyboard(false);
                self.tracks.clear();
                println!("cleared all tracks");
                Ok(ShellControl::Continue)
            }
            "mix" => {
                let output = parts
                    .next()
                    .ok_or_else(|| "usage: mix <output.wav>".to_string())?;
                self.mix(output)?;
                Ok(ShellControl::Continue)
            }
            "play" => {
                self.play()?;
                Ok(ShellControl::Continue)
            }
            "pause" => {
                self.pause()?;
                Ok(ShellControl::Continue)
            }
            "resume" => {
                self.resume()?;
                Ok(ShellControl::Continue)
            }
            "stop" => {
                self.stop_playback(true);
                Ok(ShellControl::Continue)
            }
            "latency" => {
                let mode = parts
                    .next()
                    .ok_or_else(|| "usage: latency <low|balanced|safe>".to_string())?;
                self.set_latency_mode(mode)?;
                Ok(ShellControl::Continue)
            }
            "save" => {
                let project_path = parts
                    .next()
                    .ok_or_else(|| "usage: save <project.json>".to_string())?;
                self.save_project(project_path)?;
                Ok(ShellControl::Continue)
            }
            "open" => {
                let project_path = parts
                    .next()
                    .ok_or_else(|| "usage: open <project.json>".to_string())?;
                self.open_project(project_path)?;
                Ok(ShellControl::Continue)
            }
            "quit" | "exit" => Ok(ShellControl::Exit),
            _ => Err(format!("unknown command `{command}`; type `help`")),
        }
    }

    fn print_help(&self) {
        println!("commands:");
        println!("  help                            show this help");
        println!("  load <path.wav|path.mid> [gain] load an audio or MIDI track");
        println!("  midi-live <name>                create a live MIDI performance track");
        println!("  soundfont <index> <path.sf2|basic> set MIDI instrument source");
        println!("  keyboard list                   list MIDI keyboard input ports");
        println!("  keyboard connect <port> <track> connect keyboard to a MIDI track");
        println!("  keyboard disconnect             disconnect MIDI keyboard");
        println!("  keyboard status                 show keyboard routing");
        println!("  keyboard monitor <port>         print incoming MIDI events for a port");
        println!("  keyboard monitor stop           stop MIDI event monitor");
        println!("  keyboard monitor status         show monitor status");
        println!("  record start <track>            start MIDI recording on a midi-live track");
        println!(
            "  record stop                     stop recording and commit notes into the track"
        );
        println!("  record status                   show recording status");
        println!("  list                            list loaded tracks");
        println!("  gain <index> <gain>             change track gain");
        println!("  mute <index> [on|off|toggle]    set mute state");
        println!("  solo <index> [on|off|toggle]    set solo state");
        println!("  rename <index> <new_name>       rename a track");
        println!("  offset <index> <seconds>        set track start offset");
        println!("  speed <index> <ratio>           set playback speed");
        println!("  pitch <index> <semitones>       set pitch in semitones");
        println!("  notes <index> <seconds>         print active MIDI notes at time");
        println!("  remove <index>                  remove a track");
        println!("  clear                           remove all tracks");
        println!("  play                            start realtime playback");
        println!("  pause                           pause playback");
        println!("  resume                          resume playback");
        println!("  stop                            stop playback");
        println!("  latency <low|balanced|safe>     set output latency mode");
        println!("  mix <output.wav>                export the current mix");
        println!("  save <project.json>             save a simple project file");
        println!("  open <project.json>             load a project file");
        println!("  quit                            exit the shell");
    }

    fn load_track(&mut self, raw_path: &str, gain: f32) -> Result<(), String> {
        let path = PathBuf::from(raw_path);
        let source = load_track_source(&path)?;

        self.stop_playback_if_running("project changed, playback stopped");

        let display_name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("track")
            .to_string();
        let info = info_from_track_source(&source);

        self.tracks.push(Track {
            name: display_name.clone(),
            path: path.clone(),
            gain,
            mute: false,
            solo: false,
            offset_seconds: 0.0,
            speed: 1.0,
            pitch_semitones: 0.0,
            source,
        });

        println!(
            "loaded [{}] {} [{}] ({:.2}s, {} Hz, {} ch, gain {:.2})",
            self.tracks.len() - 1,
            display_name,
            self.tracks
                .last()
                .map(|track| track_type_label(&track.source))
                .unwrap_or("unknown"),
            info.duration_seconds,
            info.sample_rate,
            info.channels,
            gain
        );

        Ok(())
    }

    fn create_live_midi_track(&mut self, name: &str) -> Result<(), String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("track name cannot be empty".to_string());
        }

        self.stop_playback_if_running("project changed, playback stopped");
        self.tracks.push(Track {
            name: trimmed.to_string(),
            path: PathBuf::from("<live-midi>"),
            gain: 1.0,
            mute: false,
            solo: false,
            offset_seconds: 0.0,
            speed: 1.0,
            pitch_semitones: 0.0,
            source: TrackSource::MidiLive {
                instrument: MidiInstrument::Basic,
            },
        });

        println!(
            "created live MIDI track [{}] {}",
            self.tracks.len() - 1,
            trimmed
        );
        Ok(())
    }

    fn list_tracks(&self) {
        if self.tracks.is_empty() {
            println!("no tracks loaded");
            return;
        }

        for (index, track) in self.tracks.iter().enumerate() {
            let info = info_from_track_source(&track.source);
            println!(
                "[{}] {} | type {} | file {} | gain {:.2} | mute {} | solo {} | offset {:.3}s | speed {:.3}x | pitch {:+.2} st | {:.2}s | {} Hz | {} ch",
                index,
                track.name,
                track_type_label(&track.source),
                track.path.display(),
                track.gain,
                track.mute,
                track.solo,
                track.offset_seconds,
                track.speed,
                track.pitch_semitones,
                info.duration_seconds,
                info.sample_rate,
                info.channels
            );
        }

        if let Some(playback) = &self.playback {
            if playback.is_finished() {
                println!("playback status: finished");
            } else if playback.is_paused() {
                println!(
                    "playback status: paused ({} Hz, {} ch, latency {})",
                    playback.output_sample_rate,
                    playback.output_channels,
                    self.latency_mode.label()
                );
            } else {
                println!(
                    "playback status: running ({} Hz, {} ch, latency {})",
                    playback.output_sample_rate,
                    playback.output_channels,
                    self.latency_mode.label()
                );
            }
        }

        if self.recording.is_some() {
            self.print_recording_status();
        }
    }

    fn set_gain(&mut self, index: usize, gain: f32) -> Result<(), String> {
        let track = self.track_mut(index)?;
        track.gain = gain;
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] gain set to {:.2}", index, gain);
        Ok(())
    }

    fn set_toggle(&mut self, index: usize, action: &str, field: ToggleField) -> Result<(), String> {
        let track = self.track_mut(index)?;
        let value = match action {
            "toggle" => !field.get(track),
            "on" => true,
            "off" => false,
            _ => {
                return Err(format!(
                    "invalid toggle action `{action}`; use on, off, or toggle"
                ));
            }
        };
        field.set(track, value);
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] {} = {}", index, field.label(), value);
        Ok(())
    }

    fn rename_track(&mut self, index: usize, name: &str) -> Result<(), String> {
        let track = self.track_mut(index)?;
        track.name = name.to_string();
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] renamed to {}", index, name);
        Ok(())
    }

    fn set_offset(&mut self, index: usize, seconds: f32) -> Result<(), String> {
        if seconds < 0.0 {
            return Err("offset must be >= 0".to_string());
        }
        let track = self.track_mut(index)?;
        track.offset_seconds = seconds;
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] offset set to {:.3}s", index, seconds);
        Ok(())
    }

    fn set_speed(&mut self, index: usize, speed: f32) -> Result<(), String> {
        if !speed.is_finite() || speed <= 0.0 {
            return Err("speed must be a finite number > 0".to_string());
        }
        let track = self.track_mut(index)?;
        track.speed = speed;
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] speed set to {:.3}x", index, speed);
        Ok(())
    }

    fn set_pitch(&mut self, index: usize, semitones: f32) -> Result<(), String> {
        if !semitones.is_finite() {
            return Err("pitch semitones must be a finite number".to_string());
        }
        let track = self.track_mut(index)?;
        track.pitch_semitones = semitones;
        self.stop_playback_if_running("project changed, playback stopped");
        println!("track [{}] pitch set to {:+.2} st", index, semitones);
        Ok(())
    }

    fn print_notes_at(&self, index: usize, seconds: f32) -> Result<(), String> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("seconds must be a finite number >= 0".to_string());
        }

        let track = self
            .tracks
            .get(index)
            .ok_or_else(|| format!("track index {} out of range", index))?;

        let TrackSource::Midi { clip, .. } = &track.source else {
            return Err(format!("track [{}] is not a MIDI track", index));
        };

        let local_seconds = seconds - track.offset_seconds;
        if local_seconds < 0.0 {
            println!("no active notes at {:.3}s", seconds);
            return Ok(());
        }

        let notes = active_notes_at(clip, local_seconds, track.speed, track.pitch_semitones);
        if notes.is_empty() {
            println!("no active notes at {:.3}s", seconds);
            return Ok(());
        }

        println!(
            "active MIDI notes at {:.3}s on track [{}] {}:",
            seconds, index, track.name
        );
        for note in notes {
            print_active_note(&note);
        }

        Ok(())
    }

    fn remove_track(&mut self, index: usize) -> Result<(), String> {
        if index >= self.tracks.len() {
            return Err(format!("track index {} out of range", index));
        }
        if self
            .recording
            .as_ref()
            .map(|recording| recording.track_index)
            == Some(index)
        {
            self.stop_recording(false)?;
        }
        if self.keyboard_track == Some(index) {
            self.disconnect_keyboard(false);
        } else if let Some(current) = self.keyboard_track {
            if index < current {
                self.keyboard_track = Some(current - 1);
            }
        }
        self.stop_playback_if_running("project changed, playback stopped");
        let removed = self.tracks.remove(index);
        println!("removed {}", removed.name);
        Ok(())
    }

    fn set_soundfont(&mut self, index: usize, value: &str) -> Result<(), String> {
        {
            let track = self.track_mut(index)?;
            let instrument = match &mut track.source {
                TrackSource::Midi { instrument, .. } => instrument,
                TrackSource::MidiLive { instrument } => instrument,
                TrackSource::Audio(_) => {
                    return Err(format!("track [{}] is not a MIDI track", index));
                }
            };

            *instrument = if value == "basic" {
                MidiInstrument::Basic
            } else {
                let path = PathBuf::from(value);
                if !path.exists() {
                    return Err(format!("soundfont `{}` does not exist", path.display()));
                }
                MidiInstrument::SoundFont(path)
            };
        }

        let reconnect_port_index = if self.keyboard_track == Some(index) {
            self.keyboard_port_index
        } else {
            None
        };
        self.stop_playback_if_running("project changed, playback stopped");

        if let Some(port_index) = reconnect_port_index {
            self.disconnect_keyboard(false);
            match self.connect_keyboard_internal(port_index, index) {
                Ok(port_name) => {
                    println!("track [{}] instrument set", index);
                    println!(
                        "keyboard reconnected: port {} -> track [{}] {}",
                        port_name, index, self.tracks[index].name
                    );
                }
                Err(err) => {
                    println!("track [{}] instrument set", index);
                    println!("keyboard disconnected because the routed track instrument changed");
                    return Err(format!(
                        "failed to reconnect keyboard on port {}: {err}",
                        port_index
                    ));
                }
            }
        } else {
            println!("track [{}] instrument set", index);
        }
        Ok(())
    }

    fn list_keyboard_ports(&self) -> Result<(), String> {
        let ports = list_input_ports()?;
        if ports.is_empty() {
            println!("no MIDI keyboard input ports found");
            return Ok(());
        }

        for (index, port) in ports.iter().enumerate() {
            if port.is_virtual_loopback {
                println!(
                    "[{}] {} [loopback, not a physical keyboard]",
                    index, port.name
                );
            } else {
                println!("[{}] {}", index, port.name);
            }
        }
        Ok(())
    }

    fn connect_keyboard_to_track(
        &mut self,
        port_index: usize,
        track_index: usize,
    ) -> Result<(), String> {
        self.disconnect_keyboard(false);
        let port_name = self.connect_keyboard_internal(port_index, track_index)?;
        println!(
            "keyboard connected: port {} -> track [{}] {}",
            port_name, track_index, self.tracks[track_index].name
        );
        Ok(())
    }

    fn disconnect_keyboard(&mut self, print_message: bool) {
        if self.recording.is_some() {
            let _ = self.stop_recording(false);
        }
        if self.keyboard.take().is_some() {
            self.keyboard_track = None;
            self.keyboard_port_index = None;
            if print_message {
                println!("keyboard disconnected");
            }
        }
    }

    fn print_keyboard_status(&self) {
        match (&self.keyboard, self.keyboard_track) {
            (Some(session), Some(track_index)) => {
                let track_name = self
                    .tracks
                    .get(track_index)
                    .map(|track| track.name.as_str())
                    .unwrap_or("<missing>");
                let port_index = self
                    .keyboard_port_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "?".to_string());
                println!(
                    "keyboard connected: {} (port #{}) -> track [{}] {}",
                    session.port_name(),
                    port_index,
                    track_index,
                    track_name
                );
            }
            _ => println!("keyboard disconnected"),
        }
    }

    fn start_keyboard_monitor(&mut self, port_index: usize) -> Result<(), String> {
        self.stop_keyboard_monitor(false);
        let session = monitor_keyboard_input(port_index)?;
        let port_name = session.port_name().to_string();
        self.keyboard_monitor = Some(session);
        println!(
            "keyboard monitor started on port {} ({})",
            port_index, port_name
        );
        Ok(())
    }

    fn stop_keyboard_monitor(&mut self, print_message: bool) {
        if self.keyboard_monitor.take().is_some() && print_message {
            println!("keyboard monitor stopped");
        }
    }

    fn print_keyboard_monitor_status(&self) {
        match &self.keyboard_monitor {
            Some(session) => println!("keyboard monitor active: {}", session.port_name()),
            None => println!("keyboard monitor stopped"),
        }
    }

    fn start_recording(&mut self, track_index: usize) -> Result<(), String> {
        if self.recording.is_some() {
            return Err("recording is already running".to_string());
        }

        if self.keyboard_track != Some(track_index) || self.keyboard.is_none() {
            return Err(
                "recording requires the keyboard to be connected to the target track".to_string(),
            );
        }

        let instrument = match self
            .tracks
            .get(track_index)
            .ok_or_else(|| format!("track index {} out of range", track_index))?
            .source
        {
            TrackSource::MidiLive { .. } => true,
            TrackSource::Midi { .. } => false,
            TrackSource::Audio(_) => {
                return Err(format!("track [{}] is not a MIDI track", track_index));
            }
        };

        if !instrument {
            return Err("recording currently only supports `midi-live` tracks".to_string());
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

        println!(
            "recording started on track [{}] {}",
            track_index, self.tracks[track_index].name
        );
        Ok(())
    }

    fn stop_recording(&mut self, print_message: bool) -> Result<(), String> {
        let Some(recording) = self.recording.take() else {
            if print_message {
                println!("recording stopped");
            }
            return Ok(());
        };

        if let Some(session) = &self.keyboard {
            session.set_message_listener(None);
        }

        let elapsed = recording.started_at.elapsed().as_secs_f32().max(0.01);
        let (notes, note_count, duration_seconds) =
            finalize_recording_state(&recording.state, elapsed);

        if note_count == 0 {
            if print_message {
                println!("recording stopped: no notes captured");
            }
            return Ok(());
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

        if print_message {
            println!(
                "recording stopped: captured {} notes into track [{}] {} ({:.2}s)",
                note_count, recording.track_index, track.name, duration_seconds
            );
        }

        Ok(())
    }

    fn print_recording_status(&self) {
        match &self.recording {
            Some(recording) => {
                let elapsed = recording.started_at.elapsed().as_secs_f32();
                let note_count = recording
                    .state
                    .lock()
                    .map(|state| state.notes.len())
                    .unwrap_or(0);
                let track_name = self
                    .tracks
                    .get(recording.track_index)
                    .map(|track| track.name.as_str())
                    .unwrap_or("<missing>");
                println!(
                    "recording status: active -> track [{}] {} | {:.2}s | {} committed notes",
                    recording.track_index, track_name, elapsed, note_count
                );
            }
            None => println!("recording status: stopped"),
        }
    }

    fn keyboard_instrument_for_track(&self, track_index: usize) -> Result<&MidiInstrument, String> {
        let track = self
            .tracks
            .get(track_index)
            .ok_or_else(|| format!("track index {} out of range", track_index))?;
        match &track.source {
            TrackSource::Midi { instrument, .. } => Ok(instrument),
            TrackSource::MidiLive { instrument } => Ok(instrument),
            TrackSource::Audio(_) => Err(format!("track [{}] is not a MIDI track", track_index)),
        }
    }

    fn mix(&self, output: &str) -> Result<(), String> {
        if self.tracks.is_empty() {
            return Err("no tracks loaded".to_string());
        }

        let render_specs = self.build_render_specs()?;
        let summary = mix_render_tracks_to_file(Path::new(output), &render_specs)?;
        println!(
            "mixed {} tracks into {} ({} Hz, {} ch, {} frames)",
            summary.track_count, output, summary.sample_rate, summary.channels, summary.frames
        );
        Ok(())
    }

    fn play(&mut self) -> Result<(), String> {
        if self.tracks.is_empty() {
            return Err("no tracks loaded".to_string());
        }

        self.stop_playback(false);
        let render_specs = self.build_render_specs()?;
        let playback = start_playback(&render_specs, self.latency_mode, 0.0, None)?;
        println!(
            "playback started ({} Hz, {} ch, latency {})",
            playback.output_sample_rate,
            playback.output_channels,
            self.latency_mode.label()
        );
        self.playback = Some(playback);
        Ok(())
    }

    fn pause(&mut self) -> Result<(), String> {
        let playback = self
            .playback
            .as_mut()
            .ok_or_else(|| "playback is not running".to_string())?;
        playback.pause()?;
        println!("playback paused");
        Ok(())
    }

    fn resume(&mut self) -> Result<(), String> {
        let playback = self
            .playback
            .as_mut()
            .ok_or_else(|| "playback is not running".to_string())?;
        playback.resume()?;
        println!("playback resumed");
        Ok(())
    }

    fn stop_playback(&mut self, print_message: bool) {
        if self.playback.take().is_some() && print_message {
            println!("playback stopped");
        }
    }

    fn stop_playback_if_running(&mut self, message: &str) {
        if self.playback.take().is_some() {
            println!("{message}");
        }
    }

    fn set_latency_mode(&mut self, raw: &str) -> Result<(), String> {
        self.latency_mode = LatencyMode::parse(raw)?;
        self.stop_playback_if_running("latency mode changed, playback stopped");
        println!("latency mode set to {}", self.latency_mode.label());
        Ok(())
    }

    fn save_project(&self, project_path: &str) -> Result<(), String> {
        let project_base_dir = Path::new(project_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let file = File::create(project_path)
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
                    speed: track.speed,
                    pitch_semitones: track.pitch_semitones,
                    midi_soundfont: track_midi_soundfont(&track.source),
                    embedded_midi: embedded_midi_clip(track),
                })
                .collect(),
        };

        serde_json::to_writer_pretty(writer, &project)
            .map_err(|err| format!("failed to save `{project_path}`: {err}"))?;
        println!("saved project to {}", project_path);
        Ok(())
    }

    fn open_project(&mut self, project_path: &str) -> Result<(), String> {
        let file = File::open(project_path)
            .map_err(|err| format!("failed to open `{project_path}`: {err}"))?;
        let reader = BufReader::new(file);
        let project: ProjectFile = serde_json::from_reader(reader)
            .map_err(|err| format!("failed to parse `{project_path}`: {err}"))?;

        if project.version != 1 {
            return Err(format!("unsupported project version {}", project.version));
        }

        let base_dir = Path::new(project_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let mut loaded_tracks: Vec<Track> = Vec::with_capacity(project.tracks.len());
        for track in project.tracks {
            let (path, source) = load_project_track_source(&base_dir, &track)?;
            loaded_tracks.push(Track {
                name: track.name,
                path,
                gain: track.gain,
                mute: track.mute,
                solo: track.solo,
                offset_seconds: track.offset_seconds,
                speed: track.speed,
                pitch_semitones: track.pitch_semitones,
                source: apply_saved_instrument(source, track.midi_soundfont)?,
            });
        }

        let _ = self.stop_recording(false);
        self.stop_playback(false);
        self.disconnect_keyboard(false);
        self.tracks = loaded_tracks;
        println!("loaded project from {}", project_path);
        Ok(())
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
                render_specs.push(RenderTrackSpec {
                    offset_frames: seconds_to_frames(track.offset_seconds, audio.sample_rate),
                    clip_start_frames: 0,
                    clip_frame_count: audio.frames,
                    audio,
                    gain: track.gain,
                    mute: track.mute,
                    solo: track.solo,
                    speed: track.speed,
                    pitch_semitones: track.pitch_semitones,
                });
            }
        }

        if render_specs.is_empty() {
            let has_live_tracks = self
                .tracks
                .iter()
                .any(|track| matches!(track.source, TrackSource::MidiLive { .. }));
            if has_live_tracks {
                return Err(
                    "no renderable tracks loaded; `midi-live` tracks are realtime-only and are not included in `play`/`mix`"
                        .to_string(),
                );
            }
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

    fn track_mut(&mut self, index: usize) -> Result<&mut Track, String> {
        self.tracks
            .get_mut(index)
            .ok_or_else(|| format!("track index {} out of range", index))
    }

    fn connect_keyboard_internal(
        &mut self,
        port_index: usize,
        track_index: usize,
    ) -> Result<String, String> {
        let instrument = self.keyboard_instrument_for_track(track_index)?.clone();
        let session = connect_keyboard(port_index, &instrument)?;
        if let Some(recording) = &self.recording {
            if recording.track_index == track_index {
                let state_for_listener = recording.state.clone();
                let started_at = recording.started_at;
                let listener: Arc<KeyboardMessageListener> = Arc::new(move |message: &[u8]| {
                    record_midi_message(&state_for_listener, started_at, message);
                });
                session.set_message_listener(Some(listener));
            }
        }
        let port_name = session.port_name().to_string();
        self.keyboard = Some(session);
        self.keyboard_track = Some(track_index);
        self.keyboard_port_index = Some(port_index);
        Ok(port_name)
    }
}

impl Default for LatencyMode {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Clone, Copy)]
enum ToggleField {
    Mute,
    Solo,
}

impl ToggleField {
    fn get(self, track: &Track) -> bool {
        match self {
            Self::Mute => track.mute,
            Self::Solo => track.solo,
        }
    }

    fn set(self, track: &mut Track, value: bool) {
        match self {
            Self::Mute => track.mute = value,
            Self::Solo => track.solo = value,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Mute => "mute",
            Self::Solo => "solo",
        }
    }
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

fn track_type_label(source: &TrackSource) -> &'static str {
    match source {
        TrackSource::Audio(_) => "audio",
        TrackSource::Midi { .. } => "midi",
        TrackSource::MidiLive { .. } => "midi-live",
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

fn print_active_note(note: &ActiveMidiNote) {
    println!(
        "  note {} | freq {:.2} Hz | vel {} | ch {} | start {:.3}s | end {:.3}s",
        note.note,
        note.frequency_hz,
        note.velocity,
        note.channel,
        note.start_seconds,
        note.end_seconds
    );
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

fn resolve_project_path(base_dir: &Path, raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() || path.exists() {
        path
    } else {
        base_dir.join(path)
    }
}

fn make_project_path(base_dir: &Path, path: &Path) -> String {
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn seconds_to_frames(seconds: f32, sample_rate: u32) -> usize {
    (seconds.max(0.0) * sample_rate as f32).round() as usize
}

fn parse_index(raw: &str) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|_| format!("invalid track index `{raw}`"))
}

fn default_speed() -> f32 {
    1.0
}

fn default_source_kind() -> String {
    "audio".to_string()
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

fn apply_saved_instrument(
    source: TrackSource,
    midi_soundfont: Option<String>,
) -> Result<TrackSource, String> {
    match source {
        TrackSource::Midi { clip, .. } => {
            let instrument = saved_instrument(midi_soundfont);
            Ok(TrackSource::Midi { clip, instrument })
        }
        TrackSource::MidiLive { .. } => Ok(TrackSource::MidiLive {
            instrument: saved_instrument(midi_soundfont),
        }),
        other => Ok(other),
    }
}

fn saved_instrument(midi_soundfont: Option<String>) -> MidiInstrument {
    match midi_soundfont {
        Some(path) => MidiInstrument::SoundFont(PathBuf::from(path)),
        None => MidiInstrument::Basic,
    }
}

fn track_source_kind(source: &TrackSource) -> &'static str {
    match source {
        TrackSource::Audio(_) => "audio",
        TrackSource::Midi { .. } => "midi",
        TrackSource::MidiLive { .. } => "midi_live",
    }
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

fn project_track_source_kind(track: &Track) -> &'static str {
    if is_embedded_midi_track(track) {
        "midi_embedded"
    } else {
        track_source_kind(&track.source)
    }
}
