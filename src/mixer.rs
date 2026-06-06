use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

const WRITE_CHUNK_SAMPLES: usize = 65_536;

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
pub struct AudioMetadata {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f32,
}

#[derive(Debug, Clone)]
struct WavData {
    sample_rate: u32,
    channels: u16,
    samples: Vec<f32>,
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

pub fn mix_to_file(output: &Path, track_specs: &[TrackSpec]) -> Result<MixSummary, String> {
    let tracks = read_tracks(track_specs)?;
    let mixed = mix_tracks(&tracks)?;
    write_wav(output, &mixed)?;

    Ok(MixSummary {
        track_count: tracks.len(),
        sample_rate: mixed.sample_rate,
        channels: mixed.channels,
        frames: mixed.samples.len() / usize::from(mixed.channels),
    })
}

pub fn read_audio_metadata(path: &Path) -> Result<AudioMetadata, String> {
    let reader =
        WavReader::open(path).map_err(|err| format!("failed to open `{}`: {err}", path.display()))?;
    let spec = reader.spec();

    if spec.channels == 0 {
        return Err(format!("`{}` has invalid channel count 0", path.display()));
    }

    let total_samples = reader.duration() as usize;
    let frames = total_samples / usize::from(spec.channels);
    let duration_seconds = frames as f32 / spec.sample_rate as f32;

    Ok(AudioMetadata {
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        duration_seconds,
    })
}

fn read_tracks(track_specs: &[TrackSpec]) -> Result<Vec<(WavData, f32)>, String> {
    track_specs
        .par_iter()
        .map(|spec| read_wav(&spec.path).map(|wav| (wav, spec.gain)))
        .collect()
}

fn mix_tracks(tracks: &[(WavData, f32)]) -> Result<WavData, String> {
    let Some((first, _)) = tracks.first() else {
        return Err("no tracks to mix".to_string());
    };

    let sample_rate = first.sample_rate;
    let output_channels = tracks
        .iter()
        .map(|(wav, _)| wav.channels)
        .max()
        .unwrap_or(first.channels);
    let output_channels_usize = usize::from(output_channels);
    let max_frames = tracks
        .iter()
        .map(|(wav, _)| wav.samples.len() / usize::from(wav.channels))
        .max()
        .unwrap_or(0);

    for (index, (wav, _)) in tracks.iter().enumerate().skip(1) {
        if wav.sample_rate != sample_rate {
            return Err(format!(
                "track {} has sample rate {}, expected {}",
                index + 1,
                wav.sample_rate,
                sample_rate
            ));
        }
        if wav.channels == 0 {
            return Err(format!("track {} has invalid channel count 0", index + 1));
        }
    }

    let mut mixed = vec![0.0_f32; max_frames * output_channels_usize];
    let chunk_frames = 16_384usize;

    mixed
        .par_chunks_mut(chunk_frames * output_channels_usize)
        .enumerate()
        .for_each(|(chunk_index, chunk)| {
            let frame_start = chunk_index * chunk_frames;
            let frames_in_chunk = chunk.len() / output_channels_usize;

            for (wav, gain) in tracks {
                let input_channels = usize::from(wav.channels);
                let input_frames = wav.samples.len() / input_channels;
                if frame_start >= input_frames {
                    continue;
                }

                let frames_to_mix = frames_in_chunk.min(input_frames - frame_start);
                let samples = &wav.samples;

                match (input_channels, output_channels_usize) {
                    (1, 1) => {
                        let src = &samples[frame_start..frame_start + frames_to_mix];
                        let dst = &mut chunk[..frames_to_mix];
                        for (dst_sample, src_sample) in dst.iter_mut().zip(src) {
                            *dst_sample += *src_sample * *gain;
                        }
                    }
                    (1, 2) => {
                        let src = &samples[frame_start..frame_start + frames_to_mix];
                        let dst = &mut chunk[..frames_to_mix * 2];
                        for (src_sample, dst_frame) in src.iter().zip(dst.chunks_exact_mut(2)) {
                            let value = *src_sample * *gain;
                            dst_frame[0] += value;
                            dst_frame[1] += value;
                        }
                    }
                    (2, 1) => {
                        let src = &samples[frame_start * 2..(frame_start + frames_to_mix) * 2];
                        let dst = &mut chunk[..frames_to_mix];
                        for (src_frame, dst_sample) in src.chunks_exact(2).zip(dst.iter_mut()) {
                            *dst_sample += (src_frame[0] + src_frame[1]) * 0.5 * *gain;
                        }
                    }
                    (2, 2) => {
                        let src = &samples[frame_start * 2..(frame_start + frames_to_mix) * 2];
                        let dst = &mut chunk[..frames_to_mix * 2];
                        for (dst_sample, src_sample) in dst.iter_mut().zip(src) {
                            *dst_sample += *src_sample * *gain;
                        }
                    }
                    _ => {
                        for frame_offset in 0..frames_to_mix {
                            let dst_base = frame_offset * output_channels_usize;
                            let src_base = (frame_start + frame_offset) * input_channels;
                            for out_ch in 0..output_channels_usize {
                                let src_ch = out_ch.min(input_channels - 1);
                                chunk[dst_base + out_ch] += samples[src_base + src_ch] * *gain;
                            }
                        }
                    }
                }
            }

            for sample in chunk {
                *sample = sample.clamp(-1.0, 1.0);
            }
        });

    Ok(WavData {
        sample_rate,
        channels: output_channels,
        samples: mixed,
    })
}

fn read_wav(path: &Path) -> Result<WavData, String> {
    let mut reader =
        WavReader::open(path).map_err(|err| format!("failed to open `{}`: {err}", path.display()))?;
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

    Ok(WavData {
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        samples,
    })
}

fn write_wav(path: &Path, wav: &WavData) -> Result<(), String> {
    let spec = WavSpec {
        channels: wav.channels,
        sample_rate: wav.sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec)
        .map_err(|err| format!("failed to create `{}`: {err}", path.display()))?;

    for chunk in wav.samples.chunks(WRITE_CHUNK_SAMPLES) {
        let mut sample_writer = writer.get_i16_writer(chunk.len() as u32);
        for sample in chunk {
            let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
            sample_writer.write_sample(value);
        }
        sample_writer
            .flush()
            .map_err(|err| format!("failed to write `{}`: {err}", path.display()))?;
    }

    writer
        .finalize()
        .map_err(|err| format!("failed to finalize `{}`: {err}", path.display()))
}
