use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const WRITE_CHUNK_FRAMES: usize = 16_384;

#[derive(Debug, Clone)]
pub struct TrackSpec {
    pub path: PathBuf,
    pub gain: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MixSummary {
    pub track_count: usize,
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioFileInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone)]
pub struct LoadedAudio {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
    pub duration_seconds: f32,
    samples: Arc<[f32]>,
}

impl LoadedAudio {
    pub fn from_interleaved(
        sample_rate: u32,
        channels: u16,
        samples: Arc<[f32]>,
        frames: usize,
    ) -> Self {
        let duration_seconds = if sample_rate == 0 {
            0.0
        } else {
            frames as f32 / sample_rate as f32
        };

        Self {
            sample_rate,
            channels,
            frames,
            duration_seconds,
            samples,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderTrackSpec {
    pub audio: LoadedAudio,
    pub gain: f32,
    pub mute: bool,
    pub solo: bool,
    pub offset_frames: usize,
    pub clip_start_frames: usize,
    pub clip_frame_count: usize,
    pub speed: f32,
    pub pitch_semitones: f32,
}

#[derive(Debug, Clone)]
pub struct PreparedTrack {
    pub offset_frames: usize,
    pub clip_start_frames: usize,
    pub clip_frame_count: usize,
    pub gain: f32,
    pub audio: LoadedAudio,
}

#[derive(Debug, Clone)]
pub struct PreparedMix {
    pub sample_rate: u32,
    pub channels: u16,
    pub frames: usize,
    pub tracks: Vec<PreparedTrack>,
}

pub fn parse_track_spec(raw: String) -> Result<TrackSpec, String> {
    let Some((path, gain)) = raw.rsplit_once(':') else {
        return Err(format!(
            "invalid track spec `{raw}`; expected format `path.wav:gain`"
        ));
    };

    let gain = gain
        .parse::<f32>()
        .map_err(|_| format!("invalid gain `{gain}` in `{raw}`"))?;

    Ok(TrackSpec {
        path: PathBuf::from(path),
        gain,
    })
}

pub fn load_audio_file(path: &Path) -> Result<LoadedAudio, String> {
    let mut reader = WavReader::open(path)
        .map_err(|err| format!("failed to open `{}`: {err}", path.display()))?;
    let spec = reader.spec();

    if spec.channels == 0 {
        return Err(format!("`{}` has invalid channel count 0", path.display()));
    }

    let samples = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|sample| {
                sample
                    .map(|value| f32::from(value) / f32::from(i16::MAX))
                    .map_err(|err| format!("failed reading `{}`: {err}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .map(|sample| {
                sample.map_err(|err| format!("failed reading `{}`: {err}", path.display()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        _ => {
            return Err(format!(
                "`{}` uses unsupported WAV format: {:?} / {} bits; supported inputs are 16-bit PCM or 32-bit float WAV",
                path.display(),
                spec.sample_format,
                spec.bits_per_sample
            ));
        }
    };

    let frames = samples.len() / usize::from(spec.channels);
    Ok(LoadedAudio::from_interleaved(
        spec.sample_rate,
        spec.channels,
        Arc::<[f32]>::from(samples),
        frames,
    ))
}

pub fn mix_to_file(output: &Path, track_specs: &[TrackSpec]) -> Result<MixSummary, String> {
    let render_specs = track_specs
        .iter()
        .map(|spec| {
            load_audio_file(&spec.path).map(|audio| RenderTrackSpec {
                clip_frame_count: audio.frames,
                audio,
                gain: spec.gain,
                mute: false,
                solo: false,
                offset_frames: 0,
                clip_start_frames: 0,
                speed: 1.0,
                pitch_semitones: 0.0,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    mix_render_tracks_to_file(output, &render_specs)
}

pub fn mix_render_tracks_to_file(
    output: &Path,
    track_specs: &[RenderTrackSpec],
) -> Result<MixSummary, String> {
    let prepared = prepare_mix(track_specs)?;
    write_prepared_mix_to_file(output, &prepared)?;

    Ok(MixSummary {
        track_count: prepared.tracks.len(),
        sample_rate: prepared.sample_rate,
        channels: prepared.channels,
        frames: prepared.frames,
    })
}

pub fn prepare_mix(track_specs: &[RenderTrackSpec]) -> Result<PreparedMix, String> {
    let active_tracks = collect_active_tracks(track_specs)?;
    let transformed_tracks = active_tracks
        .par_iter()
        .map(transform_track)
        .collect::<Result<Vec<_>, _>>()?;
    let first = transformed_tracks
        .first()
        .ok_or_else(|| "no tracks to mix".to_string())?;

    let sample_rate = first.audio.sample_rate;
    let channels = transformed_tracks
        .iter()
        .map(|track| track.audio.channels)
        .max()
        .unwrap_or(first.audio.channels);
    let frames = transformed_tracks
        .iter()
        .map(|track| track.offset_frames + track.clip_frame_count)
        .max()
        .unwrap_or(0);

    for (index, track) in transformed_tracks.iter().enumerate().skip(1) {
        if track.audio.sample_rate != sample_rate {
            return Err(format!(
                "track {} has sample rate {}, expected {}",
                index + 1,
                track.audio.sample_rate,
                sample_rate
            ));
        }
    }

    let tracks = transformed_tracks
        .into_iter()
        .map(|track| PreparedTrack {
            offset_frames: track.offset_frames,
            clip_start_frames: track.clip_start_frames,
            clip_frame_count: track.clip_frame_count,
            gain: track.gain,
            audio: track.audio,
        })
        .collect();

    Ok(PreparedMix {
        sample_rate,
        channels,
        frames,
        tracks,
    })
}

pub fn mix_chunk_into(
    prepared: &PreparedMix,
    frame_start: usize,
    frame_count: usize,
    dst: &mut [f32],
) {
    let output_channels = usize::from(prepared.channels);
    let chunk_end = frame_start.saturating_add(frame_count);
    dst.fill(0.0);

    for track in &prepared.tracks {
        let track_start = track.offset_frames;
        let track_end = track.offset_frames + track.clip_frame_count;
        if chunk_end <= track_start || frame_start >= track_end {
            continue;
        }

        let overlap_start = frame_start.max(track_start);
        let overlap_end = chunk_end.min(track_end);
        let src_frame_start = track.clip_start_frames + (overlap_start - track_start);
        let dst_frame_start = overlap_start - frame_start;

        for frame_offset in 0..(overlap_end - overlap_start) {
            let src_frame = src_frame_start + frame_offset;
            let dst_base = (dst_frame_start + frame_offset) * output_channels;

            for out_ch in 0..output_channels {
                dst[dst_base + out_ch] +=
                    sample_at_output_channel(&track.audio, src_frame, out_ch, output_channels)
                        * track.gain;
            }
        }
    }

    for sample in dst.iter_mut() {
        *sample = sample.clamp(-1.0, 1.0);
    }
}

pub fn write_prepared_mix_to_file(path: &Path, prepared: &PreparedMix) -> Result<(), String> {
    let spec = WavSpec {
        channels: prepared.channels,
        sample_rate: prepared.sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)
        .map_err(|err| format!("failed to create `{}`: {err}", path.display()))?;

    let channels = usize::from(prepared.channels);
    let mut chunk = vec![0.0_f32; WRITE_CHUNK_FRAMES * channels];
    let mut frame_start = 0usize;

    while frame_start < prepared.frames {
        let frame_count = (prepared.frames - frame_start).min(WRITE_CHUNK_FRAMES);
        let sample_count = frame_count * channels;
        mix_chunk_into(
            prepared,
            frame_start,
            frame_count,
            &mut chunk[..sample_count],
        );

        let mut sample_writer = writer.get_i16_writer(sample_count as u32);
        for sample in &chunk[..sample_count] {
            let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
            sample_writer.write_sample(value);
        }
        sample_writer
            .flush()
            .map_err(|err| format!("failed to write `{}`: {err}", path.display()))?;

        frame_start += frame_count;
    }

    writer
        .finalize()
        .map_err(|err| format!("failed to finalize `{}`: {err}", path.display()))
}

fn collect_active_tracks(track_specs: &[RenderTrackSpec]) -> Result<Vec<RenderTrackSpec>, String> {
    let has_solo = track_specs.iter().any(|track| track.solo);
    let active_tracks = track_specs
        .iter()
        .filter(|track| !track.mute && (!has_solo || track.solo))
        .cloned()
        .collect::<Vec<_>>();

    if active_tracks.is_empty() {
        return Err("no audible tracks to mix".to_string());
    }

    Ok(active_tracks)
}

fn transform_track(track: &RenderTrackSpec) -> Result<RenderTrackSpec, String> {
    validate_transform_params(track)?;

    if approx_eq(track.speed, 1.0) && approx_eq(track.pitch_semitones, 0.0) {
        return Ok(track.clone());
    }

    let pitch_ratio = semitone_ratio(track.pitch_semitones);
    let stretched = if approx_eq(pitch_ratio, 1.0) {
        time_stretch_audio(&track.audio, 1.0 / track.speed)?
    } else {
        let pitched = resample_audio(&track.audio, pitch_ratio)?;
        time_stretch_audio(&pitched, pitch_ratio / track.speed)?
    };

    let stretched_frames = stretched.frames;
    Ok(RenderTrackSpec {
        audio: stretched,
        gain: track.gain,
        mute: track.mute,
        solo: track.solo,
        offset_frames: track.offset_frames,
        clip_start_frames: track.clip_start_frames,
        clip_frame_count: track.clip_frame_count.min(stretched_frames),
        speed: track.speed,
        pitch_semitones: track.pitch_semitones,
    })
}

fn validate_transform_params(track: &RenderTrackSpec) -> Result<(), String> {
    if !track.speed.is_finite() || track.speed <= 0.0 {
        return Err(format!("invalid speed {}; expected > 0", track.speed));
    }
    if !track.pitch_semitones.is_finite() {
        return Err(format!(
            "invalid pitch semitones {}; expected a finite number",
            track.pitch_semitones
        ));
    }
    if track.clip_start_frames > track.audio.frames {
        return Err("clip start exceeds source length".to_string());
    }
    if track.clip_start_frames + track.clip_frame_count > track.audio.frames {
        return Err("clip end exceeds source length".to_string());
    }
    Ok(())
}

fn resample_audio(audio: &LoadedAudio, rate: f32) -> Result<LoadedAudio, String> {
    if !rate.is_finite() || rate <= 0.0 {
        return Err(format!("invalid resample rate {rate}; expected > 0"));
    }

    let channels = usize::from(audio.channels);
    let output_frames = ((audio.frames as f32) / rate).ceil().max(1.0) as usize;
    let mut output = vec![0.0_f32; output_frames * channels];

    for out_frame in 0..output_frames {
        let src_pos = out_frame as f32 * rate;
        let frame0 = src_pos.floor() as usize;
        let frame1 = (frame0 + 1).min(audio.frames.saturating_sub(1));
        let frac = src_pos - frame0 as f32;

        for channel in 0..channels {
            let sample0 =
                sample_from_audio(audio, frame0.min(audio.frames.saturating_sub(1)), channel);
            let sample1 = sample_from_audio(audio, frame1, channel);
            output[out_frame * channels + channel] = sample0 + (sample1 - sample0) * frac;
        }
    }

    Ok(LoadedAudio::from_interleaved(
        audio.sample_rate,
        audio.channels,
        Arc::<[f32]>::from(output),
        output_frames,
    ))
}

fn time_stretch_audio(audio: &LoadedAudio, stretch: f32) -> Result<LoadedAudio, String> {
    if !stretch.is_finite() || stretch <= 0.0 {
        return Err(format!("invalid stretch factor {stretch}; expected > 0"));
    }
    if approx_eq(stretch, 1.0) || audio.frames < 2 {
        return Ok(audio.clone());
    }

    let channels = usize::from(audio.channels);
    let window_size = 1024usize.min(audio.frames.max(1));
    let analysis_hop = (window_size / 4).max(1);
    let synthesis_hop = ((analysis_hop as f32) * stretch).round().max(1.0) as usize;
    let estimated_frames = ((audio.frames as f32) * stretch).ceil().max(1.0) as usize;
    let output_frames = estimated_frames + window_size;
    let mut output = vec![0.0_f32; output_frames * channels];
    let mut weights = vec![0.0_f32; output_frames];
    let window = hann_window(window_size);

    let mut input_pos = 0usize;
    let mut output_pos = 0usize;

    while input_pos < audio.frames {
        let copy_frames = (audio.frames - input_pos).min(window_size);

        for frame in 0..copy_frames {
            let weight = window[frame];
            let out_frame = output_pos + frame;
            weights[out_frame] += weight;

            for channel in 0..channels {
                let sample = sample_from_audio(audio, input_pos + frame, channel);
                output[out_frame * channels + channel] += sample * weight;
            }
        }

        input_pos = input_pos.saturating_add(analysis_hop);
        output_pos = output_pos.saturating_add(synthesis_hop);
    }

    for frame in 0..output_frames {
        let weight = weights[frame];
        if weight > 1e-6 {
            let base = frame * channels;
            for channel in 0..channels {
                output[base + channel] /= weight;
            }
        }
    }

    let trimmed_frames = estimated_frames.max(1).min(output_frames);
    output.truncate(trimmed_frames * channels);

    Ok(LoadedAudio::from_interleaved(
        audio.sample_rate,
        audio.channels,
        Arc::<[f32]>::from(output),
        trimmed_frames,
    ))
}

fn sample_from_audio(audio: &LoadedAudio, frame: usize, channel: usize) -> f32 {
    let channels = usize::from(audio.channels);
    audio.samples[frame * channels + channel.min(channels.saturating_sub(1))]
}

fn hann_window(size: usize) -> Vec<f32> {
    if size <= 1 {
        return vec![1.0; size.max(1)];
    }

    let denom = (size - 1) as f32;
    (0..size)
        .map(|index| {
            let phase = 2.0 * std::f32::consts::PI * index as f32 / denom;
            0.5 - 0.5 * phase.cos()
        })
        .collect()
}

fn semitone_ratio(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

fn approx_eq(left: f32, right: f32) -> bool {
    (left - right).abs() <= 1e-4
}

fn sample_at_output_channel(
    audio: &LoadedAudio,
    frame: usize,
    out_ch: usize,
    output_channels: usize,
) -> f32 {
    match (usize::from(audio.channels), output_channels) {
        (1, 1) => audio.samples[frame],
        (1, _) => audio.samples[frame],
        (2, 1) => {
            let base = frame * 2;
            (audio.samples[base] + audio.samples[base + 1]) * 0.5
        }
        (2, _) => {
            let base = frame * 2;
            audio.samples[base + out_ch.min(1)]
        }
        (input_channels, _) => {
            let src_ch = out_ch.min(input_channels - 1);
            audio.samples[frame * input_channels + src_ch]
        }
    }
}
