import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getProjectState,
  loadTrack,
  createMidiTrack,
  createLiveMidiTrack,
  duplicateTrack,
  removeTrack,
  clearProject,
  patchTrack,
  setTrackInstrument,
  setTrackNotes,
  saveProject,
  openProject,
  exportMix,
  play,
  pause,
  resume,
  stop,
  setLatencyMode,
  setPlayhead,
  setLoop,
  type UiTrack,
  type UiProjectState,
  type TrackPatch,
  type MidiNote
} from '@/api/backend'

export const useProjectStore = defineStore('project', () => {
  const tracks = ref<UiTrack[]>([])
  const latencyMode = ref('balanced')
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const projectDuration = ref(0)
  const loopEnabled = ref(false)
  const loopStart = ref(0)
  const loopEnd = ref(0)
  const zoom = ref(100)
  const activeTrackId = ref<number | null>(null)

  const playheadPosition = computed(() => currentTime.value * zoom.value)
  const pixelsPerSecond = computed(() => zoom.value)

  async function loadState() {
    const state = await getProjectState()
    syncState(state)
  }

  function syncState(state: UiProjectState) {
    tracks.value = state.tracks
    latencyMode.value = state.latency_mode
    isPlaying.value = state.playback.running && !state.playback.paused
    currentTime.value = state.transport.playhead_seconds
    projectDuration.value = state.transport.project_duration_seconds
    loopEnabled.value = state.transport.loop_enabled
    loopStart.value = state.transport.loop_start_seconds
    loopEnd.value = state.transport.loop_end_seconds
  }

  async function loadTrackFile(path: string, gain?: number) {
    const result = await loadTrack(path, gain)
    syncState(result.state)
    return result
  }

  async function createMidi(name: string) {
    const result = await createMidiTrack(name)
    syncState(result.state)
    return result
  }

  async function createLiveMidi(name: string) {
    const result = await createLiveMidiTrack(name)
    syncState(result.state)
    return result
  }

  async function duplicate(index: number) {
    const result = await duplicateTrack(index)
    syncState(result.state)
    return result
  }

  async function remove(index: number) {
    const result = await removeTrack(index)
    syncState(result.state)
    return result
  }

  async function clear() {
    const result = await clearProject()
    syncState(result.state)
    return result
  }

  async function updateTrack(index: number, patch: TrackPatch) {
    const result = await patchTrack(index, patch)
    syncState(result.state)
    return result
  }

  async function updateTrackInstrument(index: number, instrumentKind: string, soundfontPath?: string) {
    const result = await setTrackInstrument(index, instrumentKind, soundfontPath)
    syncState(result.state)
    return result
  }

  async function updateTrackNotes(index: number, notes: MidiNote[], durationSeconds: number) {
    const result = await setTrackNotes(index, notes, durationSeconds)
    syncState(result.state)
    return result
  }

  async function save(path: string) {
    const result = await saveProject(path)
    syncState(result.state)
    return result
  }

  async function open(path: string) {
    const result = await openProject(path)
    syncState(result.state)
    return result
  }

  async function exportToFile(path: string) {
    const result = await exportMix(path)
    syncState(result.state)
    return result
  }

  async function startPlay() {
    const result = await play()
    syncState(result.state)
    return result
  }

  async function pausePlay() {
    const result = await pause()
    syncState(result.state)
    return result
  }

  async function resumePlay() {
    const result = await resume()
    syncState(result.state)
    return result
  }

  async function stopPlay() {
    const result = await stop()
    syncState(result.state)
    return result
  }

  async function updateLatencyMode(mode: string) {
    const result = await setLatencyMode(mode)
    syncState(result.state)
    return result
  }

  async function updatePlayhead(seconds: number) {
    const result = await setPlayhead(seconds)
    syncState(result.state)
    return result
  }

  async function updateLoop(enabled: boolean, start: number, end: number) {
    const result = await setLoop(enabled, start, end)
    syncState(result.state)
    return result
  }

  function setZoom(newZoom: number) {
    zoom.value = Math.max(25, Math.min(400, newZoom))
  }

  function setActiveTrack(index: number | null) {
    activeTrackId.value = index
  }

  function setCurrentTime(time: number) {
    currentTime.value = Math.max(0, time)
  }

  return {
    tracks,
    latencyMode,
    isPlaying,
    currentTime,
    projectDuration,
    loopEnabled,
    loopStart,
    loopEnd,
    zoom,
    activeTrackId,
    playheadPosition,
    pixelsPerSecond,
    loadState,
    syncState,
    loadTrackFile,
    createMidi,
    createLiveMidi,
    duplicate,
    remove,
    clear,
    updateTrack,
    updateTrackInstrument,
    updateTrackNotes,
    save,
    open,
    exportToFile,
    startPlay,
    pausePlay,
    resumePlay,
    stopPlay,
    updateLatencyMode,
    updatePlayhead,
    updateLoop,
    setZoom,
    setActiveTrack,
    setCurrentTime
  }
})
