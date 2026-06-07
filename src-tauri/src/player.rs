use crate::mixer::{PreparedMix, RenderTrackSpec, mix_chunk_into, prepare_mix};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, Stream, StreamConfig};
use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const PRODUCER_CHUNK_FRAMES: usize = 16_384;
const QUEUE_CHUNKS: usize = 8;

pub struct PlaybackSession {
    stream: Stream,
    finished: Arc<AtomicBool>,
    paused: bool,
    stop_flag: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    played_output_frames: Arc<AtomicU64>,
    total_output_frames: u64,
    start_seconds: f32,
    pub output_sample_rate: u32,
    pub output_channels: u16,
}

impl PlaybackSession {
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn current_position_seconds(&self) -> f32 {
        let frames = self
            .played_output_frames
            .load(Ordering::Relaxed)
            .min(self.total_output_frames);
        self.start_seconds + frames as f32 / self.output_sample_rate as f32
    }

    pub fn pause(&mut self) -> Result<(), String> {
        if !self.paused {
            self.stream
                .pause()
                .map_err(|err| format!("failed to pause output stream: {err}"))?;
            self.paused = true;
        }
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        if self.paused {
            self.stream
                .play()
                .map_err(|err| format!("failed to resume output stream: {err}"))?;
            self.paused = false;
        }
        Ok(())
    }
}

impl Drop for PlaybackSession {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyMode {
    Low,
    Balanced,
    Safe,
}

impl LatencyMode {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "low" => Ok(Self::Low),
            "balanced" => Ok(Self::Balanced),
            "safe" => Ok(Self::Safe),
            _ => Err(format!(
                "invalid latency mode `{raw}`; use low, balanced, or safe"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Balanced => "balanced",
            Self::Safe => "safe",
        }
    }
}

pub fn start_playback(
    track_specs: &[RenderTrackSpec],
    latency_mode: LatencyMode,
    start_seconds: f32,
    stop_seconds: Option<f32>,
) -> Result<PlaybackSession, String> {
    let prepared = prepare_mix(track_specs)?;

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device available".to_string())?;
    let default_config = device
        .default_output_config()
        .map_err(|err| format!("failed to get default output config: {err}"))?;

    let sample_format = default_config.sample_format();
    let mut config: StreamConfig = default_config.config();
    tune_buffer_size(&device, &default_config, &mut config, latency_mode);

    let output_sample_rate = config.sample_rate;
    let output_channels = config.channels;
    let prepared = Arc::new(prepared);
    let queue = Arc::new(AudioQueue::new());
    let finished = Arc::new(AtomicBool::new(false));
    let stop_flag = Arc::new(AtomicBool::new(false));
    let played_output_frames = Arc::new(AtomicU64::new(0));

    let prepared_duration_seconds = prepared.frames as f32 / prepared.sample_rate as f32;
    let start_seconds = start_seconds.clamp(0.0, prepared_duration_seconds);
    let stop_seconds = stop_seconds
        .map(|value| value.clamp(start_seconds, prepared_duration_seconds))
        .filter(|value| *value > start_seconds + 1e-4);
    let total_output_frames = (((stop_seconds.unwrap_or(prepared_duration_seconds)
        - start_seconds)
        * output_sample_rate as f32)
        .ceil()
        .max(0.0)) as u64;

    let worker = spawn_mix_worker(
        prepared.clone(),
        output_sample_rate,
        output_channels,
        start_seconds,
        stop_seconds,
        queue.clone(),
        finished.clone(),
        stop_flag.clone(),
    );
    let err_fn = |err| eprintln!("audio stream error: {err}");

    let stream = match sample_format {
        SampleFormat::F32 => build_output_stream::<f32>(
            &device,
            &config,
            queue.clone(),
            finished.clone(),
            played_output_frames.clone(),
            err_fn,
        ),
        SampleFormat::I16 => build_output_stream::<i16>(
            &device,
            &config,
            queue.clone(),
            finished.clone(),
            played_output_frames.clone(),
            err_fn,
        ),
        SampleFormat::U16 => build_output_stream::<u16>(
            &device,
            &config,
            queue,
            finished.clone(),
            played_output_frames.clone(),
            err_fn,
        ),
        other => Err(format!("unsupported output sample format `{other:?}`")),
    }?;

    stream
        .play()
        .map_err(|err| format!("failed to start output stream: {err}"))?;

    Ok(PlaybackSession {
        stream,
        finished,
        paused: false,
        stop_flag,
        worker: Some(worker),
        played_output_frames,
        total_output_frames,
        start_seconds,
        output_sample_rate,
        output_channels,
    })
}

fn tune_buffer_size(
    device: &cpal::Device,
    default_config: &cpal::SupportedStreamConfig,
    config: &mut StreamConfig,
    latency_mode: LatencyMode,
) {
    let Ok(configs) = device.supported_output_configs() else {
        return;
    };

    for supported in configs {
        if supported.sample_format() != default_config.sample_format()
            || supported.channels() != default_config.channels()
        {
            continue;
        }

        let min_rate = supported.min_sample_rate();
        let max_rate = supported.max_sample_rate();
        if config.sample_rate < min_rate || config.sample_rate > max_rate {
            continue;
        }

        if let cpal::SupportedBufferSize::Range { min, max } = supported.buffer_size() {
            let target = match latency_mode {
                LatencyMode::Low => 128,
                LatencyMode::Balanced => 256,
                LatencyMode::Safe => 512,
            };
            let preferred = (*min).max(target).min(*max);
            config.buffer_size = cpal::BufferSize::Fixed(preferred);
        }
        break;
    }
}

fn spawn_mix_worker(
    prepared: Arc<PreparedMix>,
    output_sample_rate: u32,
    output_channels: u16,
    start_seconds: f32,
    stop_seconds: Option<f32>,
    queue: Arc<AudioQueue>,
    finished: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let source_rate = prepared.sample_rate;
        let source_frame_start = (start_seconds.max(0.0) * source_rate as f32).floor() as usize;
        let source_frame_end = stop_seconds
            .map(|value| (value.max(start_seconds) * source_rate as f32).ceil() as usize)
            .unwrap_or(prepared.frames)
            .min(prepared.frames);
        if source_frame_end <= source_frame_start {
            finished.store(true, Ordering::Relaxed);
            queue.mark_finished();
            return;
        }

        let total_source_frames = source_frame_end - source_frame_start;
        let output_frames = ((total_source_frames as f64 * output_sample_rate as f64)
            / source_rate as f64)
            .ceil() as usize;
        let mut output_frame_start = 0usize;

        while output_frame_start < output_frames && !stop_flag.load(Ordering::Relaxed) {
            queue.wait_for_capacity(stop_flag.as_ref());

            let chunk_output_frames =
                (output_frames - output_frame_start).min(PRODUCER_CHUNK_FRAMES);
            let source_chunk_start = source_frame_start
                + ((output_frame_start as f64 * source_rate as f64) / output_sample_rate as f64)
                    .floor() as usize;
            let source_chunk_end = source_frame_start
                + (((output_frame_start + chunk_output_frames) as f64 * source_rate as f64)
                    / output_sample_rate as f64)
                    .ceil() as usize;
            let source_chunk_end = source_chunk_end.min(source_frame_end);
            let source_frames = source_chunk_end.saturating_sub(source_chunk_start).max(1);
            let mut source_chunk = vec![0.0_f32; source_frames * usize::from(prepared.channels)];
            mix_chunk_into(
                &prepared,
                source_chunk_start,
                source_frames,
                &mut source_chunk,
            );

            let resampled = resample_chunk(
                &source_chunk,
                source_frames,
                prepared.channels,
                source_rate,
                output_sample_rate,
                output_channels,
                output_frame_start,
                chunk_output_frames,
            );
            queue.push(resampled, stop_flag.as_ref());
            output_frame_start += chunk_output_frames;
        }

        finished.store(true, Ordering::Relaxed);
        queue.mark_finished();
    })
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    queue: Arc<AudioQueue>,
    finished: Arc<AtomicBool>,
    played_output_frames: Arc<AtomicU64>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String>
where
    T: Sample + FromSample<f32> + cpal::SizedSample,
{
    let mut current_chunk: Vec<f32> = Vec::new();
    let mut current_pos = 0usize;
    let output_channels = usize::from(config.channels);

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                let mut written = 0usize;

                while written < data.len() {
                    if current_pos >= current_chunk.len() {
                        if let Some(chunk) = queue.pop() {
                            current_chunk = chunk;
                            current_pos = 0;
                        } else {
                            break;
                        }
                    }

                    let rem_src = current_chunk.len().saturating_sub(current_pos);
                    let rem_dst = data.len() - written;
                    let copy_len = rem_src.min(rem_dst);

                    for (dst, src) in data[written..written + copy_len]
                        .iter_mut()
                        .zip(current_chunk[current_pos..current_pos + copy_len].iter())
                    {
                        *dst = T::from_sample((*src).clamp(-1.0, 1.0));
                    }

                    played_output_frames
                        .fetch_add((copy_len / output_channels) as u64, Ordering::Relaxed);
                    written += copy_len;
                    current_pos += copy_len;
                }

                for dst in data.iter_mut().skip(written) {
                    *dst = T::EQUILIBRIUM;
                }

                if queue.is_finished() && current_pos >= current_chunk.len() {
                    finished.store(true, Ordering::Relaxed);
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("failed to build output stream: {err}"))
}

fn resample_chunk(
    source: &[f32],
    source_frames: usize,
    source_channels: u16,
    source_rate: u32,
    output_rate: u32,
    output_channels: u16,
    output_frame_start: usize,
    output_frames: usize,
) -> Vec<f32> {
    let out_channels = usize::from(output_channels);
    let mut output = vec![0.0_f32; output_frames * out_channels];

    for output_frame in 0..output_frames {
        let global_output_frame = output_frame_start + output_frame;
        let source_pos = global_output_frame as f64 * source_rate as f64 / output_rate as f64;
        let local_source = source_pos
            - (output_frame_start as f64 * source_rate as f64 / output_rate as f64).floor();
        let frame0 = local_source.floor() as usize;
        let frame1 = (frame0 + 1).min(source_frames.saturating_sub(1));
        let frac = (local_source - frame0 as f64) as f32;

        for out_ch in 0..out_channels {
            let sample0 = sample_from_chunk(
                source,
                source_channels,
                source_frames,
                frame0,
                out_ch,
                out_channels,
            );
            let sample1 = sample_from_chunk(
                source,
                source_channels,
                source_frames,
                frame1,
                out_ch,
                out_channels,
            );
            output[output_frame * out_channels + out_ch] = sample0 + (sample1 - sample0) * frac;
        }
    }

    output
}

fn sample_from_chunk(
    source: &[f32],
    source_channels: u16,
    source_frames: usize,
    frame: usize,
    out_ch: usize,
    output_channels: usize,
) -> f32 {
    if source_frames == 0 {
        return 0.0;
    }
    let frame = frame.min(source_frames - 1);
    match (usize::from(source_channels), output_channels) {
        (1, 1) => source[frame],
        (1, _) => source[frame],
        (2, 1) => {
            let base = frame * 2;
            (source[base] + source[base + 1]) * 0.5
        }
        (2, _) => {
            let base = frame * 2;
            source[base + out_ch.min(1)]
        }
        (input_channels, _) => {
            let src_ch = out_ch.min(input_channels - 1);
            source[frame * input_channels + src_ch]
        }
    }
}

struct AudioQueue {
    inner: Mutex<AudioQueueInner>,
    cvar: Condvar,
}

struct AudioQueueInner {
    chunks: VecDeque<Vec<f32>>,
    finished: bool,
}

impl AudioQueue {
    fn new() -> Self {
        Self {
            inner: Mutex::new(AudioQueueInner {
                chunks: VecDeque::new(),
                finished: false,
            }),
            cvar: Condvar::new(),
        }
    }

    fn wait_for_capacity(&self, stop_flag: &AtomicBool) {
        let mut inner = self.inner.lock().expect("queue lock poisoned");
        while inner.chunks.len() >= QUEUE_CHUNKS && !stop_flag.load(Ordering::Relaxed) {
            inner = self.cvar.wait(inner).expect("queue wait poisoned");
        }
    }

    fn push(&self, chunk: Vec<f32>, stop_flag: &AtomicBool) {
        if stop_flag.load(Ordering::Relaxed) {
            return;
        }
        let mut inner = self.inner.lock().expect("queue lock poisoned");
        inner.chunks.push_back(chunk);
        self.cvar.notify_all();
    }

    fn pop(&self) -> Option<Vec<f32>> {
        let mut inner = self.inner.lock().expect("queue lock poisoned");
        loop {
            if let Some(chunk) = inner.chunks.pop_front() {
                self.cvar.notify_all();
                return Some(chunk);
            }
            if inner.finished {
                return None;
            }
            inner = self.cvar.wait(inner).expect("queue wait poisoned");
        }
    }

    fn mark_finished(&self) {
        let mut inner = self.inner.lock().expect("queue lock poisoned");
        inner.finished = true;
        self.cvar.notify_all();
    }

    fn is_finished(&self) -> bool {
        let inner = self.inner.lock().expect("queue lock poisoned");
        inner.finished && inner.chunks.is_empty()
    }
}
