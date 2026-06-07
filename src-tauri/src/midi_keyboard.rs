use crate::midi::MidiInstrument;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, Stream, StreamConfig};
use midir::{Ignore, MidiInput, MidiInputConnection};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const DEFAULT_SAMPLE_RATE: u32 = 44_100;

pub struct KeyboardSession {
    _stream: Stream,
    _connection: MidiInputConnection<()>,
    port_name: String,
    message_bus: MidiMessageBus,
}

impl KeyboardSession {
    pub fn port_name(&self) -> &str {
        &self.port_name
    }

    pub fn set_message_listener(&self, listener: Option<Arc<KeyboardMessageListener>>) {
        self.message_bus.set_listener(listener);
    }
}

pub struct KeyboardMonitorSession {
    _connection: MidiInputConnection<()>,
    port_name: String,
}

impl KeyboardMonitorSession {
    pub fn port_name(&self) -> &str {
        &self.port_name
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MidiInputPortInfo {
    pub name: String,
    pub is_virtual_loopback: bool,
}

pub type KeyboardMessageListener = dyn Fn(&[u8]) + Send + Sync + 'static;

#[derive(Clone, Default)]
struct MidiMessageBus {
    listener: Arc<Mutex<Option<Arc<KeyboardMessageListener>>>>,
}

impl MidiMessageBus {
    fn dispatch(&self, message: &[u8]) {
        let listener = self.listener.lock().ok().and_then(|guard| guard.clone());
        if let Some(listener) = listener {
            listener(message);
        }
    }

    fn set_listener(&self, listener: Option<Arc<KeyboardMessageListener>>) {
        if let Ok(mut slot) = self.listener.lock() {
            *slot = listener;
        }
    }
}

pub fn list_input_ports() -> Result<Vec<MidiInputPortInfo>, String> {
    let midi_in = MidiInput::new("OpenDAW MIDI Input")
        .map_err(|err| format!("failed to initialize MIDI input: {err}"))?;
    let ports = midi_in.ports();
    ports
        .iter()
        .map(|port| {
            let name = midi_in
                .port_name(port)
                .map_err(|err| format!("failed to read MIDI port name: {err}"))?;
            Ok(MidiInputPortInfo {
                is_virtual_loopback: is_virtual_loopback_port(&name),
                name,
            })
        })
        .collect()
}

pub fn connect_keyboard(
    port_index: usize,
    instrument: &MidiInstrument,
) -> Result<KeyboardSession, String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device available".to_string())?;
    let default_config = device
        .default_output_config()
        .map_err(|err| format!("failed to get default output config: {err}"))?;

    let sample_format = default_config.sample_format();
    let mut config: StreamConfig = default_config.config();
    config.buffer_size = cpal::BufferSize::Default;
    let sample_rate = config.sample_rate;
    let channels = config.channels;

    let backend = Arc::new(Mutex::new(create_backend(
        instrument,
        sample_rate,
        channels,
    )?));
    let message_bus = MidiMessageBus::default();
    let err_fn = |err| eprintln!("keyboard audio stream error: {err}");

    let stream = match sample_format {
        SampleFormat::F32 => build_output_stream::<f32>(&device, &config, backend.clone(), err_fn),
        SampleFormat::I16 => build_output_stream::<i16>(&device, &config, backend.clone(), err_fn),
        SampleFormat::U16 => build_output_stream::<u16>(&device, &config, backend.clone(), err_fn),
        other => Err(format!("unsupported output sample format `{other:?}`")),
    }?;

    let mut midi_in = MidiInput::new("OpenDAW MIDI Input")
        .map_err(|err| format!("failed to initialize MIDI input: {err}"))?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();
    let port = ports
        .get(port_index)
        .ok_or_else(|| format!("MIDI input port {} out of range", port_index))?;
    let port_name = midi_in
        .port_name(port)
        .map_err(|err| format!("failed to read MIDI port name: {err}"))?;

    let backend_for_midi = backend.clone();
    let message_bus_for_midi = message_bus.clone();
    let connection = midi_in
        .connect(
            port,
            "OpenDAW MIDI Keyboard",
            move |_stamp, message, _| {
                if let Ok(mut backend) = backend_for_midi.lock() {
                    backend.handle_midi(message);
                }
                message_bus_for_midi.dispatch(message);
            },
            (),
        )
        .map_err(|err| format!("failed to connect MIDI input: {err}"))?;

    stream
        .play()
        .map_err(|err| format!("failed to start keyboard audio stream: {err}"))?;

    Ok(KeyboardSession {
        _stream: stream,
        _connection: connection,
        port_name,
        message_bus,
    })
}

pub fn monitor_keyboard_input(port_index: usize) -> Result<KeyboardMonitorSession, String> {
    let mut midi_in = MidiInput::new("OpenDAW MIDI Monitor")
        .map_err(|err| format!("failed to initialize MIDI input: {err}"))?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let port = ports
        .get(port_index)
        .ok_or_else(|| format!("MIDI input port {} out of range", port_index))?;
    let port_name = midi_in
        .port_name(port)
        .map_err(|err| format!("failed to read MIDI port name: {err}"))?;

    let printed_port_name = port_name.clone();
    let connection = midi_in
        .connect(
            port,
            "OpenDAW MIDI Monitor",
            move |_stamp, message, _| {
                println!(
                    "midi {} {}",
                    printed_port_name,
                    describe_midi_message(message)
                );
            },
            (),
        )
        .map_err(|err| format!("failed to connect MIDI input monitor: {err}"))?;

    Ok(KeyboardMonitorSession {
        _connection: connection,
        port_name,
    })
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    backend: Arc<Mutex<KeyboardBackend>>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, String>
where
    T: Sample + FromSample<f32> + cpal::SizedSample,
{
    let channels = usize::from(config.channels);

    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| {
                let mut mixed = vec![0.0_f32; data.len()];
                if let Ok(mut backend) = backend.lock() {
                    backend.render(&mut mixed, channels);
                }

                for (dst, src) in data.iter_mut().zip(mixed.iter()) {
                    *dst = T::from_sample((*src).clamp(-1.0, 1.0));
                }
            },
            err_fn,
            None,
        )
        .map_err(|err| format!("failed to build keyboard output stream: {err}"))
}

enum KeyboardBackend {
    Basic(BasicKeyboardSynth),
    SoundFont(SoundFontKeyboardSynth),
}

impl KeyboardBackend {
    fn handle_midi(&mut self, message: &[u8]) {
        match self {
            Self::Basic(synth) => synth.handle_midi(message),
            Self::SoundFont(synth) => synth.handle_midi(message),
        }
    }

    fn render(&mut self, output: &mut [f32], channels: usize) {
        match self {
            Self::Basic(synth) => synth.render(output, channels),
            Self::SoundFont(synth) => synth.render(output, channels),
        }
    }
}

fn create_backend(
    instrument: &MidiInstrument,
    sample_rate: u32,
    channels: u16,
) -> Result<KeyboardBackend, String> {
    match instrument {
        MidiInstrument::Basic => Ok(KeyboardBackend::Basic(BasicKeyboardSynth::new(sample_rate))),
        MidiInstrument::SoundFont(path) => Ok(KeyboardBackend::SoundFont(
            SoundFontKeyboardSynth::new(path.clone(), sample_rate, channels)?,
        )),
    }
}

struct BasicKeyboardSynth {
    sample_rate: f32,
    voices: Vec<BasicVoice>,
}

impl BasicKeyboardSynth {
    fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32,
            voices: Vec::new(),
        }
    }

    fn handle_midi(&mut self, message: &[u8]) {
        if message.is_empty() {
            return;
        }
        let status = message[0];
        let command = status & 0xF0;
        let note = *message.get(1).unwrap_or(&0);
        let value = *message.get(2).unwrap_or(&0);

        match command {
            0x90 if value > 0 => self.note_on(note, value),
            0x90 | 0x80 => self.note_off(note),
            _ => {}
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        self.voices.retain(|voice| voice.note != note);
        self.voices.push(BasicVoice {
            note,
            velocity: velocity as f32 / 127.0,
            phase: 0.0,
            released: false,
            release_level: 1.0,
        });
    }

    fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note == note {
                voice.released = true;
            }
        }
    }

    fn render(&mut self, output: &mut [f32], channels: usize) {
        output.fill(0.0);
        for frame in 0..(output.len() / channels.max(1)) {
            let mut sample = 0.0_f32;
            for voice in &mut self.voices {
                let freq = midi_note_to_frequency(voice.note);
                let phase = voice.phase;
                let tone =
                    phase.sin() * 0.75 + (phase * 2.0).sin() * 0.18 + (phase * 3.0).sin() * 0.07;
                let env = if voice.released {
                    voice.release_level *= 0.9992;
                    voice.release_level
                } else {
                    1.0
                };
                sample += tone * voice.velocity * env * 0.22;
                voice.phase += 2.0 * PI * freq / self.sample_rate;
                if voice.phase > 2.0 * PI {
                    voice.phase -= 2.0 * PI;
                }
            }

            let base = frame * channels.max(1);
            match channels {
                0 => {}
                1 => output[base] = sample,
                _ => {
                    output[base] = sample;
                    output[base + 1] = sample;
                    for ch in 2..channels {
                        output[base + ch] = sample;
                    }
                }
            }
        }

        self.voices
            .retain(|voice| !voice.released || voice.release_level > 0.0005);
    }
}

struct BasicVoice {
    note: u8,
    velocity: f32,
    phase: f32,
    released: bool,
    release_level: f32,
}

struct SoundFontKeyboardSynth {
    synth: Synthesizer,
    left: Vec<f32>,
    right: Vec<f32>,
    channels: usize,
}

impl SoundFontKeyboardSynth {
    fn new(path: PathBuf, sample_rate: u32, channels: u16) -> Result<Self, String> {
        let data = fs::read(&path)
            .map_err(|err| format!("failed to read soundfont `{}`: {err}", path.display()))?;
        let sound_font = Arc::new(
            SoundFont::new(&mut Cursor::new(data))
                .map_err(|err| format!("failed to parse soundfont `{}`: {err}", path.display()))?,
        );

        let mut settings = SynthesizerSettings::new(sample_rate as i32);
        settings.block_size = 256;
        let synth = Synthesizer::new(&sound_font, &settings)
            .map_err(|err| format!("synth init failed: {err}"))?;

        Ok(Self {
            synth,
            left: Vec::new(),
            right: Vec::new(),
            channels: usize::from(channels),
        })
    }

    fn handle_midi(&mut self, message: &[u8]) {
        if message.is_empty() {
            return;
        }
        let status = message[0];
        let channel = (status & 0x0F) as i32;
        let command = (status & 0xF0) as i32;
        let data1 = i32::from(*message.get(1).unwrap_or(&0));
        let data2 = i32::from(*message.get(2).unwrap_or(&0));
        self.synth
            .process_midi_message(channel, command, data1, data2);
    }

    fn render(&mut self, output: &mut [f32], channels: usize) {
        let frames = output.len() / channels.max(1);
        if self.left.len() < frames {
            self.left.resize(frames, 0.0);
        }
        if self.right.len() < frames {
            self.right.resize(frames, 0.0);
        }

        let left = &mut self.left[..frames];
        let right = &mut self.right[..frames];
        left.fill(0.0);
        right.fill(0.0);
        self.synth.render(left, right);

        output.fill(0.0);
        for frame in 0..frames {
            let base = frame * channels.max(1);
            let l = left[frame];
            let r = right[frame];
            match channels {
                0 => {}
                1 => output[base] = (l + r) * 0.5,
                _ => {
                    output[base] = l;
                    output[base + 1] = r;
                    for ch in 2..channels {
                        output[base + ch] = (l + r) * 0.5;
                    }
                }
            }
        }
        let _ = self.channels;
    }
}

fn midi_note_to_frequency(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

fn is_virtual_loopback_port(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.contains("midi through") || lower.contains("through port")
}

fn describe_midi_message(message: &[u8]) -> String {
    if message.is_empty() {
        return "empty".to_string();
    }

    let status = message[0];
    let channel = (status & 0x0F) + 1;
    let command = status & 0xF0;
    let data1 = *message.get(1).unwrap_or(&0);
    let data2 = *message.get(2).unwrap_or(&0);

    match command {
        0x80 => format!("note-off ch{} note={} vel={}", channel, data1, data2),
        0x90 if data2 == 0 => format!("note-off ch{} note={} vel=0", channel, data1),
        0x90 => format!("note-on ch{} note={} vel={}", channel, data1, data2),
        0xA0 => format!(
            "poly-aftertouch ch{} note={} value={}",
            channel, data1, data2
        ),
        0xB0 => format!("cc ch{} ctrl={} value={}", channel, data1, data2),
        0xC0 => format!("program-change ch{} value={}", channel, data1),
        0xD0 => format!("channel-aftertouch ch{} value={}", channel, data1),
        0xE0 => {
            let bend = ((u16::from(data2) << 7) | u16::from(data1)) as i32 - 8192;
            format!("pitch-bend ch{} value={}", channel, bend)
        }
        _ => format!("raw {:?}", message),
    }
}

#[allow(dead_code)]
pub fn default_sample_rate() -> u32 {
    DEFAULT_SAMPLE_RATE
}
