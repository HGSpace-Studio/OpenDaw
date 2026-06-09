import { invoke } from '@tauri-apps/api/core'

export interface MidiNote {
  note: number
  velocity: number
  channel: number
  start_seconds: number
  end_seconds: number
}

export interface UiTrack {
  index: number
  name: string
  path: string
  kind: string
  instrument: string
  soundfont_path: string | null
  gain: number
  pan: number
  mute: boolean
  solo: boolean
  offset_seconds: number
  trim_start_seconds: number
  trim_end_seconds: number
  speed: number
  pitch_semitones: number
  duration_seconds: number
  sample_rate: number
  channels: number
  notes: MidiNote[]
}

export interface UiPlaybackState {
  running: boolean
  paused: boolean
  position_seconds: number
  sample_rate: number
  channels: number
}

export interface UiTransportState {
  playhead_seconds: number
  project_duration_seconds: number
  loop_enabled: boolean
  loop_start_seconds: number
  loop_end_seconds: number
}

export interface UiRecordingState {
  active: boolean
  track_index: number | null
  track_name: string | null
  elapsed_seconds: number
  note_count: number
}

export interface UiProjectState {
  tracks: UiTrack[]
  latency_mode: string
  playback: UiPlaybackState
  transport: UiTransportState
  keyboard_route: {
    port_index: number
    port_name: string
    track_index: number
    track_name: string
  } | null
  recording: UiRecordingState
}

export interface UiOperationResult {
  message: string
  state: UiProjectState
}

export interface TrackPatch {
  name?: string
  gain?: number
  pan?: number
  mute?: boolean
  solo?: boolean
  offset_seconds?: number
  trim_start_seconds?: number
  trim_end_seconds?: number
  speed?: number
  pitch_semitones?: number
}

export interface MidiInputPortInfo {
  index: number
  name: string
}

export interface UiSoundFontEntry {
  name: string
  path: string
}

export async function getProjectState(): Promise<UiProjectState> {
  return invoke('project_state')
}

export async function loadTrack(path: string, gain?: number): Promise<UiOperationResult> {
  return invoke('load_track', { path, gain })
}

export async function createMidiTrack(name: string): Promise<UiOperationResult> {
  return invoke('create_midi_track', { name })
}

export async function createLiveMidiTrack(name: string): Promise<UiOperationResult> {
  return invoke('create_live_midi_track', { name })
}

export async function duplicateTrack(index: number): Promise<UiOperationResult> {
  return invoke('duplicate_track', { index })
}

export async function removeTrack(index: number): Promise<UiOperationResult> {
  return invoke('remove_track', { index })
}

export async function clearProject(): Promise<UiOperationResult> {
  return invoke('clear_project')
}

export async function patchTrack(index: number, patch: TrackPatch): Promise<UiOperationResult> {
  return invoke('patch_track', { index, patch })
}

export async function setTrackInstrument(
  index: number,
  instrument_kind: string,
  soundfont_path?: string
): Promise<UiOperationResult> {
  return invoke('set_track_instrument', { index, instrument_kind, soundfont_path })
}

export async function setTrackNotes(
  index: number,
  notes: MidiNote[],
  duration_seconds: number
): Promise<UiOperationResult> {
  return invoke('set_track_notes', { index, notes, duration_seconds })
}

export async function saveProject(path: string): Promise<UiOperationResult> {
  return invoke('save_project', { path })
}

export async function openProject(path: string): Promise<UiOperationResult> {
  return invoke('open_project', { path })
}

export async function exportMix(path: string): Promise<UiOperationResult> {
  return invoke('export_mix', { path })
}

export async function play(): Promise<UiOperationResult> {
  return invoke('play')
}

export async function pause(): Promise<UiOperationResult> {
  return invoke('pause')
}

export async function resume(): Promise<UiOperationResult> {
  return invoke('resume')
}

export async function stop(): Promise<UiOperationResult> {
  return invoke('stop')
}

export async function setLatencyMode(mode: string): Promise<UiOperationResult> {
  return invoke('set_latency_mode', { mode })
}

export async function setPlayhead(seconds: number): Promise<UiOperationResult> {
  return invoke('set_playhead', { seconds })
}

export async function setLoop(enabled: boolean, start_seconds: number, end_seconds: number): Promise<UiOperationResult> {
  return invoke('set_loop', { enabled, start_seconds, end_seconds })
}

export async function listKeyboardPorts(): Promise<MidiInputPortInfo[]> {
  return invoke('list_keyboard_ports')
}

export async function listSoundfonts(): Promise<UiSoundFontEntry[]> {
  return invoke('list_soundfonts')
}

export async function connectKeyboard(port_index: number, track_index: number): Promise<UiOperationResult> {
  return invoke('connect_keyboard', { port_index, track_index })
}

export async function disconnectKeyboard(): Promise<UiOperationResult> {
  return invoke('disconnect_keyboard')
}

export async function startRecording(track_index: number): Promise<UiOperationResult> {
  return invoke('start_recording', { track_index })
}

export async function stopRecording(): Promise<UiOperationResult> {
  return invoke('stop_recording')
}

export async function getSystemLocale(): Promise<string> {
  return invoke('system_locale')
}
