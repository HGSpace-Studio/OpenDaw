use crate::mixer::LoadedAudio;
use midly::num::{u24, u28};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::collections::HashMap;
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DEFAULT_TEMPO_US_PER_QUARTER: u32 = 500_000;
const RELEASE_SECONDS: f32 = 0.08;

#[derive(Debug, Clone)]
pub struct MidiClip {
    pub duration_seconds: f32,
    pub notes: Vec<MidiNote>,
    pub smf_bytes: Arc<[u8]>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MidiNote {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub start_seconds: f32,
    pub end_seconds: f32,
}

#[derive(Debug, Clone)]
pub struct ActiveMidiNote {
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub frequency_hz: f32,
}

#[derive(Debug, Clone)]
pub enum MidiInstrument {
    Basic,
    SoundFont(PathBuf),
}

#[derive(Debug, Clone, Copy)]
pub struct MidiSynthSettings {
    pub gain: f32,
    pub speed: f32,
    pub pitch_semitones: f32,
    pub output_sample_rate: u32,
    pub output_channels: u16,
}

#[derive(Debug, Clone)]
pub struct PreparedMidiTrack {
    pub sample_rate: u32,
    pub channels: u16,
    pub total_frames: usize,
    pub instrument: PreparedMidiInstrument,
}

#[derive(Debug, Clone)]
pub enum PreparedMidiInstrument {
    Basic {
        notes: Vec<PreparedMidiNote>,
    },
    SoundFont {
        midi_file: Arc<MidiFile>,
        sound_font: Arc<SoundFont>,
        speed: f64,
        gain: f32,
    },
}

#[derive(Debug, Clone)]
pub struct PreparedMidiNote {
    pub start_frame: usize,
    pub end_frame: usize,
    pub dry_duration_frames: usize,
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub left_gain: f32,
    pub right_gain: f32,
}

#[derive(Debug, Default)]
pub struct PreparedMidiPlaybackState {
    pub basic: BasicMidiPlaybackState,
    pub soundfont: Option<SoundFontPlaybackState>,
}

#[derive(Debug, Default)]
pub struct BasicMidiPlaybackState {
    pub next_note_index: usize,
    pub active_note_indices: Vec<usize>,
}

pub struct SoundFontPlaybackState {
    pub sequencer: MidiFileSequencer,
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl std::fmt::Debug for SoundFontPlaybackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundFontPlaybackState")
            .field("left_len", &self.left.len())
            .field("right_len", &self.right.len())
            .finish()
    }
}

pub fn is_midi_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "mid" | "midi"))
        .unwrap_or(false)
}

pub fn load_midi_file(path: &Path) -> Result<MidiClip, String> {
    let data =
        fs::read(path).map_err(|err| format!("failed to read `{}`: {err}", path.display()))?;
    let bytes = Arc::<[u8]>::from(data);
    let smf_bytes = bytes.clone();
    let smf = Smf::parse(smf_bytes.as_ref())
        .map_err(|err| format!("failed to parse MIDI `{}`: {err}", path.display()))?;
    parse_midi_smf(&smf, bytes)
}

pub fn build_midi_clip_from_notes(
    mut notes: Vec<MidiNote>,
    duration_seconds: f32,
) -> Result<MidiClip, String> {
    notes.sort_by(|left, right| left.start_seconds.total_cmp(&right.start_seconds));
    let resolved_duration = duration_seconds.max(
        notes
            .iter()
            .map(|note| note.end_seconds)
            .fold(0.0_f32, f32::max),
    );
    let bytes = encode_midi_smf(&notes, resolved_duration)?;

    Ok(MidiClip {
        duration_seconds: resolved_duration,
        notes,
        smf_bytes: Arc::<[u8]>::from(bytes),
    })
}

pub fn active_notes_at(
    clip: &MidiClip,
    seconds: f32,
    speed: f32,
    pitch_semitones: f32,
) -> Vec<ActiveMidiNote> {
    if !seconds.is_finite() || seconds < 0.0 || !speed.is_finite() || speed <= 0.0 {
        return Vec::new();
    }

    let source_seconds = seconds * speed;
    clip.notes
        .iter()
        .filter(|note| source_seconds >= note.start_seconds && source_seconds < note.end_seconds)
        .map(|note| ActiveMidiNote {
            note: transpose_note(note.note, pitch_semitones),
            velocity: note.velocity,
            channel: note.channel,
            start_seconds: note.start_seconds / speed,
            end_seconds: note.end_seconds / speed,
            frequency_hz: midi_note_to_frequency(note.note as f32 + pitch_semitones),
        })
        .collect()
}

pub fn render_midi_to_audio(
    clip: &MidiClip,
    settings: MidiSynthSettings,
    instrument: &MidiInstrument,
) -> Result<LoadedAudio, String> {
    let prepared = prepare_midi_track(clip, settings, instrument)?;
    let frames = prepared.total_frames.max(1);
    let channels = usize::from(prepared.channels);
    let mut output = vec![0.0_f32; frames * channels];
    let mut state = create_playback_state(&prepared)?;

    render_prepared_midi_chunk(
        &prepared,
        &mut state,
        0,
        frames,
        prepared.channels,
        &mut output,
    )?;

    for sample in &mut output {
        *sample = sample.clamp(-1.0, 1.0);
    }

    Ok(LoadedAudio::from_interleaved(
        prepared.sample_rate,
        prepared.channels,
        Arc::<[f32]>::from(output),
        frames,
    ))
}

pub fn prepare_midi_track(
    clip: &MidiClip,
    settings: MidiSynthSettings,
    instrument: &MidiInstrument,
) -> Result<PreparedMidiTrack, String> {
    validate_settings(settings)?;

    let total_frames = (((clip.duration_seconds / settings.speed).max(0.0) + RELEASE_SECONDS)
        * settings.output_sample_rate as f32)
        .ceil() as usize;

    let instrument = match instrument {
        MidiInstrument::Basic => PreparedMidiInstrument::Basic {
            notes: prepare_basic_notes(clip, settings),
        },
        MidiInstrument::SoundFont(path) => {
            let midi_file = Arc::new(
                MidiFile::new(&mut Cursor::new(clip.smf_bytes.as_ref()))
                    .map_err(|err| format!("failed to build MIDI stream for sf2: {err}"))?,
            );
            let sf2_data = fs::read(path)
                .map_err(|err| format!("failed to read soundfont `{}`: {err}", path.display()))?;
            let sound_font =
                Arc::new(SoundFont::new(&mut Cursor::new(sf2_data)).map_err(|err| {
                    format!("failed to parse soundfont `{}`: {err}", path.display())
                })?);

            PreparedMidiInstrument::SoundFont {
                midi_file,
                sound_font,
                speed: settings.speed as f64,
                gain: settings.gain,
            }
        }
    };

    Ok(PreparedMidiTrack {
        sample_rate: settings.output_sample_rate,
        channels: settings.output_channels,
        total_frames,
        instrument,
    })
}

pub fn create_playback_state(
    prepared: &PreparedMidiTrack,
) -> Result<PreparedMidiPlaybackState, String> {
    match &prepared.instrument {
        PreparedMidiInstrument::Basic { .. } => Ok(PreparedMidiPlaybackState::default()),
        PreparedMidiInstrument::SoundFont {
            midi_file,
            sound_font,
            speed,
            ..
        } => {
            let mut synth_settings = SynthesizerSettings::new(prepared.sample_rate as i32);
            synth_settings.block_size = 256;
            let synthesizer = Synthesizer::new(sound_font, &synth_settings)
                .map_err(|err| format!("synth init failed: {err}"))?;
            let mut sequencer = MidiFileSequencer::new(synthesizer);
            sequencer.play(midi_file, false);
            sequencer.set_speed(*speed);

            Ok(PreparedMidiPlaybackState {
                basic: BasicMidiPlaybackState::default(),
                soundfont: Some(SoundFontPlaybackState {
                    sequencer,
                    left: Vec::new(),
                    right: Vec::new(),
                }),
            })
        }
    }
}

pub fn render_prepared_midi_chunk(
    prepared: &PreparedMidiTrack,
    state: &mut PreparedMidiPlaybackState,
    chunk_start_frame: usize,
    frame_count: usize,
    output_channels: u16,
    dst: &mut [f32],
) -> Result<(), String> {
    match &prepared.instrument {
        PreparedMidiInstrument::Basic { notes } => {
            render_basic_chunk(
                notes,
                prepared.sample_rate,
                &mut state.basic,
                chunk_start_frame,
                frame_count,
                output_channels,
                dst,
            );
            Ok(())
        }
        PreparedMidiInstrument::SoundFont { gain, .. } => render_soundfont_chunk(
            state
                .soundfont
                .as_mut()
                .ok_or_else(|| "missing soundfont playback state".to_string())?,
            frame_count,
            output_channels,
            *gain,
            dst,
        ),
    }
}

fn validate_settings(settings: MidiSynthSettings) -> Result<(), String> {
    if !settings.speed.is_finite() || settings.speed <= 0.0 {
        return Err(format!(
            "invalid MIDI speed {}; expected > 0",
            settings.speed
        ));
    }
    if !settings.pitch_semitones.is_finite() || !settings.gain.is_finite() {
        return Err("invalid MIDI synth settings".to_string());
    }
    Ok(())
}

fn prepare_basic_notes(clip: &MidiClip, settings: MidiSynthSettings) -> Vec<PreparedMidiNote> {
    let sample_rate_f32 = settings.output_sample_rate as f32;
    clip.notes
        .iter()
        .map(|note| {
            let note_start = note.start_seconds / settings.speed;
            let note_end = note.end_seconds / settings.speed;
            let start_frame = (note_start * sample_rate_f32).floor().max(0.0) as usize;
            let end_frame = ((note_end + RELEASE_SECONDS) * sample_rate_f32)
                .ceil()
                .max(0.0) as usize;
            let dry_duration_frames =
                ((note_end - note_start).max(0.0) * sample_rate_f32).round() as usize;
            let pan = midi_pan(note.channel);

            PreparedMidiNote {
                start_frame,
                end_frame,
                dry_duration_frames,
                frequency_hz: midi_note_to_frequency(note.note as f32 + settings.pitch_semitones),
                amplitude: velocity_to_gain(note.velocity) * settings.gain,
                left_gain: (1.0 - pan) * 0.5,
                right_gain: (1.0 + pan) * 0.5,
            }
        })
        .collect()
}

fn render_basic_chunk(
    notes: &[PreparedMidiNote],
    sample_rate: u32,
    state: &mut BasicMidiPlaybackState,
    chunk_start_frame: usize,
    frame_count: usize,
    output_channels: u16,
    dst: &mut [f32],
) {
    let chunk_end_frame = chunk_start_frame.saturating_add(frame_count);
    let out_channels = usize::from(output_channels);

    state
        .active_note_indices
        .retain(|&idx| notes[idx].end_frame > chunk_start_frame);

    while state.next_note_index < notes.len()
        && notes[state.next_note_index].start_frame < chunk_end_frame
    {
        let idx = state.next_note_index;
        if notes[idx].end_frame > chunk_start_frame {
            state.active_note_indices.push(idx);
        }
        state.next_note_index += 1;
    }

    for &note_idx in &state.active_note_indices {
        let note = &notes[note_idx];
        let overlap_start = note.start_frame.max(chunk_start_frame);
        let overlap_end = note.end_frame.min(chunk_end_frame);
        let dry_duration_seconds = note.dry_duration_frames as f32 / sample_rate as f32;

        for frame in overlap_start..overlap_end {
            let local_frame = frame - chunk_start_frame;
            let t = (frame - note.start_frame) as f32 / sample_rate as f32;
            let env = envelope(t, dry_duration_seconds);
            if env <= 0.0 {
                continue;
            }

            let phase = 2.0 * PI * note.frequency_hz * t;
            let sample =
                (phase.sin() * 0.72 + (phase * 2.0).sin() * 0.18 + (phase * 3.0).sin() * 0.10)
                    * env
                    * note.amplitude;
            let base = local_frame * out_channels;

            match out_channels {
                0 => {}
                1 => dst[base] += sample * 0.5 * (note.left_gain + note.right_gain),
                _ => {
                    dst[base] += sample * note.left_gain;
                    dst[base + 1] += sample * note.right_gain;
                    for channel in 2..out_channels {
                        dst[base + channel] += sample * 0.5;
                    }
                }
            }
        }
    }
}

fn render_soundfont_chunk(
    state: &mut SoundFontPlaybackState,
    frame_count: usize,
    output_channels: u16,
    gain: f32,
    dst: &mut [f32],
) -> Result<(), String> {
    if state.left.len() < frame_count {
        state.left.resize(frame_count, 0.0);
    }
    if state.right.len() < frame_count {
        state.right.resize(frame_count, 0.0);
    }

    let left = &mut state.left[..frame_count];
    let right = &mut state.right[..frame_count];
    left.fill(0.0);
    right.fill(0.0);
    state.sequencer.render(left, right);

    let out_channels = usize::from(output_channels);
    for frame in 0..frame_count {
        let l = left[frame] * gain;
        let r = right[frame] * gain;
        let base = frame * out_channels;

        match out_channels {
            0 => {}
            1 => dst[base] += (l + r) * 0.5,
            _ => {
                dst[base] += l;
                dst[base + 1] += r;
                for channel in 2..out_channels {
                    dst[base + channel] += (l + r) * 0.5;
                }
            }
        }
    }

    Ok(())
}

fn parse_midi_smf(smf: &Smf<'_>, smf_bytes: Arc<[u8]>) -> Result<MidiClip, String> {
    let ticks_per_quarter = match smf.header.timing {
        Timing::Metrical(ticks) => u32::from(ticks.as_int()),
        Timing::Timecode(_, _) => {
            return Err("SMPTE timecode MIDI is not supported yet".to_string());
        }
    };

    if ticks_per_quarter == 0 {
        return Err("invalid MIDI timing: ticks per quarter is 0".to_string());
    }

    let mut notes = Vec::new();
    let mut note_starts: HashMap<(u8, u8), Vec<(f64, u8)>> = HashMap::new();
    let mut max_time_seconds = 0.0_f64;

    for track in &smf.tracks {
        let mut tempo_us_per_quarter = DEFAULT_TEMPO_US_PER_QUARTER;
        let mut time_seconds = 0.0_f64;

        for event in track {
            let delta_ticks = u64::from(event.delta.as_int());
            time_seconds += ticks_to_seconds(delta_ticks, tempo_us_per_quarter, ticks_per_quarter);

            match event.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                    tempo_us_per_quarter = tempo.as_int();
                }
                TrackEventKind::Midi { channel, message } => match message {
                    MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                        note_starts
                            .entry((channel.as_int(), key.as_int()))
                            .or_default()
                            .push((time_seconds, vel.as_int()));
                    }
                    MidiMessage::NoteOn { key, vel } if vel.as_int() == 0 => {
                        flush_note(
                            &mut notes,
                            &mut note_starts,
                            channel.as_int(),
                            key.as_int(),
                            time_seconds,
                        );
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        flush_note(
                            &mut notes,
                            &mut note_starts,
                            channel.as_int(),
                            key.as_int(),
                            time_seconds,
                        );
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        max_time_seconds = max_time_seconds.max(time_seconds);
    }

    for ((channel, note), stack) in note_starts {
        for (start_seconds, velocity) in stack {
            let end_seconds = max_time_seconds.max(start_seconds + 0.05);
            notes.push(MidiNote {
                note,
                velocity,
                channel,
                start_seconds: start_seconds as f32,
                end_seconds: end_seconds as f32,
            });
        }
    }

    notes.sort_by(|left, right| left.start_seconds.total_cmp(&right.start_seconds));

    Ok(MidiClip {
        duration_seconds: max_time_seconds as f32,
        notes,
        smf_bytes,
    })
}

fn encode_midi_smf(notes: &[MidiNote], duration_seconds: f32) -> Result<Vec<u8>, String> {
    const TICKS_PER_QUARTER: u16 = 480;

    #[derive(Clone)]
    struct TimedEvent {
        tick: u32,
        order: u8,
        kind: TrackEventKind<'static>,
    }

    let mut events = Vec::with_capacity(notes.len() * 2 + 2);
    events.push(TimedEvent {
        tick: 0,
        order: 0,
        kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(DEFAULT_TEMPO_US_PER_QUARTER))),
    });

    for note in notes {
        let channel = note.channel.min(15).into();
        let key = note.note.min(127).into();
        let velocity = note.velocity.min(127).into();
        let start_tick = seconds_to_ticks(note.start_seconds, TICKS_PER_QUARTER);
        let end_tick = seconds_to_ticks(
            note.end_seconds.max(note.start_seconds + 0.01),
            TICKS_PER_QUARTER,
        );

        events.push(TimedEvent {
            tick: start_tick,
            order: 2,
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel: velocity },
            },
        });
        events.push(TimedEvent {
            tick: end_tick.max(start_tick),
            order: 1,
            kind: TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOff { key, vel: 0.into() },
            },
        });
    }

    let end_tick = seconds_to_ticks(duration_seconds.max(0.01), TICKS_PER_QUARTER);
    events.push(TimedEvent {
        tick: end_tick,
        order: 255,
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    events.sort_by(|left, right| {
        left.tick
            .cmp(&right.tick)
            .then(left.order.cmp(&right.order))
    });

    let mut previous_tick = 0_u32;
    let mut track = Vec::<TrackEvent<'static>>::with_capacity(events.len());
    for event in events {
        let delta = event.tick.saturating_sub(previous_tick);
        previous_tick = event.tick;
        track.push(TrackEvent {
            delta: u28::from(delta),
            kind: event.kind,
        });
    }

    let smf = Smf {
        header: Header::new(
            Format::SingleTrack,
            Timing::Metrical(TICKS_PER_QUARTER.into()),
        ),
        tracks: vec![track],
    };

    let mut bytes = Vec::new();
    smf.write_std(&mut bytes)
        .map_err(|err| format!("failed to encode recorded MIDI clip: {err}"))?;
    Ok(bytes)
}

fn flush_note(
    notes: &mut Vec<MidiNote>,
    note_starts: &mut HashMap<(u8, u8), Vec<(f64, u8)>>,
    channel: u8,
    note: u8,
    end_seconds: f64,
) {
    if let Some(stack) = note_starts.get_mut(&(channel, note)) {
        if let Some((start_seconds, velocity)) = stack.pop() {
            notes.push(MidiNote {
                note,
                velocity,
                channel,
                start_seconds: start_seconds as f32,
                end_seconds: end_seconds.max(start_seconds) as f32,
            });
        }
        if stack.is_empty() {
            note_starts.remove(&(channel, note));
        }
    }
}

fn ticks_to_seconds(delta_ticks: u64, tempo_us_per_quarter: u32, ticks_per_quarter: u32) -> f64 {
    let micros_per_tick = tempo_us_per_quarter as f64 / ticks_per_quarter as f64;
    delta_ticks as f64 * micros_per_tick / 1_000_000.0
}

fn seconds_to_ticks(seconds: f32, ticks_per_quarter: u16) -> u32 {
    let quarter_notes =
        seconds.max(0.0) as f64 / (DEFAULT_TEMPO_US_PER_QUARTER as f64 / 1_000_000.0);
    (quarter_notes * f64::from(ticks_per_quarter))
        .round()
        .max(0.0) as u32
}

fn midi_note_to_frequency(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}

fn velocity_to_gain(velocity: u8) -> f32 {
    let normalized = velocity as f32 / 127.0;
    normalized * normalized
}

fn envelope(t: f32, note_duration: f32) -> f32 {
    let attack = 0.005_f32;
    let decay = 0.030_f32;
    let sustain = 0.82_f32;
    let release = RELEASE_SECONDS;

    if t < attack {
        return (t / attack).clamp(0.0, 1.0);
    }
    if t < attack + decay {
        let alpha = (t - attack) / decay;
        return 1.0 + (sustain - 1.0) * alpha.clamp(0.0, 1.0);
    }
    if t <= note_duration {
        return sustain;
    }
    let release_t = (t - note_duration) / release;
    (sustain * (1.0 - release_t)).clamp(0.0, sustain)
}

fn transpose_note(note: u8, pitch_semitones: f32) -> u8 {
    (note as f32 + pitch_semitones).round().clamp(0.0, 127.0) as u8
}

fn midi_pan(channel: u8) -> f32 {
    match channel % 4 {
        0 => -0.2,
        1 => 0.2,
        2 => -0.08,
        _ => 0.08,
    }
}
