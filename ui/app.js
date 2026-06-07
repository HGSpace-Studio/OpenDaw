const DEFAULT_PROJECT_PATH = "test/output/ui_project.json";
const DEFAULT_EXPORT_PATH = "test/output/ui_mix.wav";
const POLL_INTERVAL_MS = 150;
const ARRANGE_PPS = 120;
const PIANO_PPS = 180;
const PIANO_ROW_HEIGHT = 20;
const PIANO_KEY_WIDTH = 56;
const PIANO_DEFAULT_MIN = 60;
const PIANO_DEFAULT_MAX = 83;
const NOTE_MIN_DURATION = 0.0625;
const LAYOUT_STORAGE_KEY = "opendaw-layout-v1";
const DEFAULT_LAYOUT = {
  uiScale: 0.8,
  sidebarWidth: 252,
  bottomDockHeight: 248,
  mixerWidth: 286,
  trackHeaderWidth: 172,
};

const I18N = {
  en: {
    appTitle: "OpenDAW",
    arrangement: "Arrangement",
    play: "Play",
    pause: "Pause",
    resume: "Resume",
    stop: "Stop",
    record: "Record",
    recording: "Recording",
    import: "Import",
    newTrack: "New Track",
    liveMidi: "Live MIDI",
    duplicate: "Duplicate",
    exportMix: "Export Mix",
    tempo: "Tempo",
    save: "Save",
    load: "Load",
    library: "Library",
    soundfonts: "SoundFonts",
    inspector: "Inspector",
    name: "Name",
    gain: "Gain",
    pan: "Pan",
    color: "Color",
    speed: "Speed",
    pitch: "Pitch",
    offset: "Offset",
    keyboardPort: "Keyboard Port",
    length: "Length",
    notes: "Notes",
    pattern: "Pattern",
    connectKeyboard: "Connect Keyboard",
    disconnectKeyboard: "Disconnect Keyboard",
    basicSynth: "Basic Synth",
    setSoundfont: "Set SoundFont",
    tracks: "Tracks",
    pianoRoll: "Piano Roll",
    clearPattern: "Clear Pattern",
    fillDemo: "Fill Demo",
    stepsPerBar: "16 steps per bar",
    mixer: "Mixer",
    channelStrips: "Channel strips",
    ready: "Ready",
    noTracks: "No tracks loaded.",
    noMidiPorts: "No MIDI ports",
    loopback: "loopback",
    bar: "Bar",
    bars: "bars",
    track: "Track",
    active: "Active",
    muted: "Muted",
    solo: "Solo",
    mute: "Mute",
    unmute: "Unmute",
    unsolo: "Unsolo",
    defaultSoundfont: "Default SoundFont",
    basicInstrumentDesc: "Built-in synth",
    detectedInFolder: "Detected in test/src",
    noSoundfontsFound: "No .sf2 files found in test/src",
    doubleClickOpen: "Double-click a MIDI clip to open piano roll",
    doubleClickMidi: "Double-click the piano grid to create notes.",
    pianoRollAudioHint: "Audio tracks do not have a piano roll",
    pianoRollLiveHint: "Live MIDI: double-click clips or record from keyboard",
    pianoRollEditHint: "Drag notes to move, resize the right edge, marquee-select on empty grid",
    playbackPaused: "Playback paused",
    playbackStopped: "Playback stopped",
    playbackRunning: "Playback",
    latency: "latency",
    importPrompt: "Import WAV or MIDI path",
    newTrackPrompt: "New MIDI track name",
    liveTrackPrompt: "New live MIDI track name",
    exportPrompt: "Export mix path",
    savePrompt: "Save project path",
    openPrompt: "Open project path",
    soundfontPrompt: "SoundFont path",
    noTrackSelected: "No track selected",
    selectMidiTrack: "Select a MIDI track first",
    selectLiveTrackForRecord: "Recording requires a selected live MIDI track",
    audioNoPianoRoll: "Audio tracks cannot open piano roll",
    recordingStatus: "Recording",
    tracksLoaded: "tracks",
    patternEmpty: "Empty",
    patternReady: "Ready",
    sf2Applied: "Applied SoundFont",
    loopEnabled: "Loop enabled",
    loopDisabled: "Loop disabled",
    trimHint: "Drag clip edges to trim",
    rulerHint: "Left click sets playhead. Right-drag creates loop range.",
  },
  zh: {
    appTitle: "OpenDAW",
    arrangement: "编曲",
    play: "播放",
    pause: "暂停",
    resume: "继续",
    stop: "停止",
    record: "录音",
    recording: "录音中",
    import: "导入",
    newTrack: "新建轨道",
    liveMidi: "实时 MIDI",
    duplicate: "复制",
    exportMix: "导出混音",
    tempo: "速度",
    save: "保存",
    load: "加载",
    library: "音色库",
    soundfonts: "SoundFont",
    inspector: "检查器",
    name: "名称",
    gain: "音量",
    pan: "声像",
    color: "颜色",
    speed: "速度",
    pitch: "变调",
    offset: "偏移",
    keyboardPort: "键盘口",
    length: "长度",
    notes: "音符",
    pattern: "片段",
    connectKeyboard: "连接键盘",
    disconnectKeyboard: "断开键盘",
    basicSynth: "基础合成器",
    setSoundfont: "设置 SoundFont",
    tracks: "轨道",
    pianoRoll: "钢琴卷帘",
    clearPattern: "清空片段",
    fillDemo: "填充示例",
    stepsPerBar: "每小节 16 步",
    mixer: "混音台",
    channelStrips: "通道条",
    ready: "就绪",
    noTracks: "当前没有轨道。",
    noMidiPorts: "没有 MIDI 输入口",
    loopback: "回环",
    bar: "小节",
    bars: "小节",
    track: "轨道",
    active: "激活",
    muted: "静音",
    solo: "独奏",
    mute: "静音",
    unmute: "取消静音",
    unsolo: "取消独奏",
    defaultSoundfont: "默认 SoundFont",
    basicInstrumentDesc: "内置合成器",
    detectedInFolder: "自动探测自 test/src",
    noSoundfontsFound: "未在 test/src 中找到 .sf2 文件",
    doubleClickOpen: "双击 MIDI 片段打开钢琴卷帘",
    doubleClickMidi: "双击网格创建音符。",
    pianoRollAudioHint: "音频轨道没有钢琴卷帘",
    pianoRollLiveHint: "实时 MIDI：双击片段打开，或直接从键盘录制",
    pianoRollEditHint: "可拖动移动音符、拖右边缘改长度、空白处框选",
    playbackPaused: "播放已暂停",
    playbackStopped: "播放已停止",
    playbackRunning: "播放中",
    latency: "延迟",
    importPrompt: "导入 WAV 或 MIDI 路径",
    newTrackPrompt: "新的 MIDI 轨道名称",
    liveTrackPrompt: "新的实时 MIDI 轨道名称",
    exportPrompt: "导出混音路径",
    savePrompt: "保存工程路径",
    openPrompt: "打开工程路径",
    soundfontPrompt: "SoundFont 路径",
    noTrackSelected: "未选择轨道",
    selectMidiTrack: "请先选择 MIDI 轨道",
    selectLiveTrackForRecord: "录制需要选中 live MIDI 轨道",
    audioNoPianoRoll: "音频轨道不能打开钢琴卷帘",
    recordingStatus: "录音",
    tracksLoaded: "轨道",
    patternEmpty: "空",
    patternReady: "可编辑",
    sf2Applied: "已应用 SoundFont",
    loopEnabled: "循环已启用",
    loopDisabled: "循环已关闭",
    trimHint: "拖动片段两侧可裁剪",
    rulerHint: "左键设置播放头，右键拖出循环范围",
  },
};

const state = {
  locale: "en",
  project: {
    tracks: [],
    latency_mode: "balanced",
    playback: { running: false, paused: false, position_seconds: 0, sample_rate: 0, channels: 0 },
    transport: {
      playhead_seconds: 0,
      project_duration_seconds: 8,
      loop_enabled: false,
      loop_start_seconds: 0,
      loop_end_seconds: 0,
    },
    keyboard_route: null,
    recording: {
      active: false,
      track_index: null,
      track_name: null,
      elapsed_seconds: 0,
      note_count: 0,
    },
  },
  selectedTrackIndex: 0,
  status: "Ready",
  projectPath: DEFAULT_PROJECT_PATH,
  exportPath: DEFAULT_EXPORT_PATH,
  keyboardPorts: [],
  soundfonts: [],
  pianoRollOpen: false,
  pollHandle: null,
  pollInFlight: false,
  lastPlaybackRunning: false,
  noteDraft: null,
  selectedNoteIndices: new Set(),
  marqueeRect: null,
  interaction: null,
  loopDraft: null,
  pianoScrollLeft: 0,
  pianoScrollTop: 0,
};

const elements = {
  arrangementViewport: document.getElementById("arrangementViewport"),
  arrangementCanvas: document.getElementById("arrangementCanvas"),
  arrangementGrid: document.getElementById("arrangementGrid"),
  timelineScale: document.getElementById("timelineScale"),
  playheadMarker: document.getElementById("playheadMarker"),
  loopOverlay: document.getElementById("loopOverlay"),
  rulerMeta: document.getElementById("rulerMeta"),
  soundfontLibrary: document.getElementById("soundfontLibrary"),
  mixerStrips: document.getElementById("mixerStrips"),
  pianoRoll: document.getElementById("pianoRoll"),
  pianoRollShell: document.getElementById("pianoRollShell"),
  pianoRollEmpty: document.getElementById("pianoRollEmpty"),
  pianoRollHint: document.getElementById("pianoRollHint"),
  statusText: document.getElementById("statusText"),
  playheadText: document.getElementById("playheadText"),
  projectSummary: document.getElementById("projectSummary"),
  selectedTrackMeta: document.getElementById("selectedTrackMeta"),
  trackNameInput: document.getElementById("trackNameInput"),
  gainInput: document.getElementById("gainInput"),
  panInput: document.getElementById("panInput"),
  colorInput: document.getElementById("colorInput"),
  speedInput: document.getElementById("speedInput"),
  pitchInput: document.getElementById("pitchInput"),
  offsetInput: document.getElementById("offsetInput"),
  keyboardPortSelect: document.getElementById("keyboardPortSelect"),
  trackLengthStat: document.getElementById("trackLengthStat"),
  trackNoteStat: document.getElementById("trackNoteStat"),
  trackPatternStat: document.getElementById("trackPatternStat"),
  tempoInput: document.getElementById("tempoInput"),
  sidebarResizer: document.getElementById("sidebarResizer"),
  dockResizer: document.getElementById("dockResizer"),
  mixerResizer: document.getElementById("mixerResizer"),
  trackHeaderResizer: document.getElementById("trackHeaderResizer"),
  uiScaleInput: document.getElementById("uiScaleInput"),
  uiScaleValue: document.getElementById("uiScaleValue"),
  resetLayoutButton: document.getElementById("resetLayoutButton"),
  playButton: document.getElementById("playButton"),
  stopButton: document.getElementById("stopButton"),
  recordButton: document.getElementById("recordButton"),
  importTrackButton: document.getElementById("importTrackButton"),
  addTrackButton: document.getElementById("addTrackButton"),
  addLiveMidiButton: document.getElementById("addLiveMidiButton"),
  duplicateTrackButton: document.getElementById("duplicateTrackButton"),
  exportMixButton: document.getElementById("exportMixButton"),
  saveProjectButton: document.getElementById("saveProjectButton"),
  loadProjectButton: document.getElementById("loadProjectButton"),
  clearPatternButton: document.getElementById("clearPatternButton"),
  fillPatternButton: document.getElementById("fillPatternButton"),
  connectKeyboardButton: document.getElementById("connectKeyboardButton"),
  disconnectKeyboardButton: document.getElementById("disconnectKeyboardButton"),
  setBasicInstrumentButton: document.getElementById("setBasicInstrumentButton"),
  setSoundfontButton: document.getElementById("setSoundfontButton"),
};

boot().catch((error) => setStatus(`Failed to boot UI: ${stringifyError(error)}`));

async function boot() {
  state.locale = await detectLocale();
  loadLayoutPrefs();
  applyI18n();
  bindUi();
  await refreshProjectState();
  await Promise.all([refreshKeyboardPorts(), refreshSoundfonts()]);
  setStatus(tr("ready"));
  startPolling();
}

function bindUi() {
  document.addEventListener("contextmenu", (event) => event.preventDefault());
  document.addEventListener("keydown", handleKeydown);
  bindLayoutControls();

  elements.playButton.addEventListener("click", async () => {
    try {
      if (state.project.playback.running && !state.project.playback.paused) {
        applyOperation(await call("pause"));
      } else if (state.project.playback.paused) {
        applyOperation(await call("resume"));
      } else {
        applyOperation(await call("play"));
      }
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.stopButton.addEventListener("click", stopPlayback);
  elements.recordButton.addEventListener("click", toggleRecording);

  elements.importTrackButton.addEventListener("click", async () => {
    const path = window.prompt(tr("importPrompt"), "test/src/test.wav");
    if (!path) return;
    try {
      const result = await call("load_track", { args: { path, gain: 1.0 } });
      applyOperation(result);
      state.selectedTrackIndex = result.state.tracks.length - 1;
      renderAll();
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.addTrackButton.addEventListener("click", async () => {
    const name = window.prompt(tr("newTrackPrompt"), `${tr("track")} ${state.project.tracks.length + 1}`);
    if (!name) return;
    try {
      const result = await call("create_midi_track", { args: { name } });
      applyOperation(result);
      state.selectedTrackIndex = result.state.tracks.length - 1;
      state.pianoRollOpen = true;
      state.selectedNoteIndices.clear();
      renderAll();
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.addLiveMidiButton.addEventListener("click", async () => {
    const name = window.prompt(tr("liveTrackPrompt"), `${tr("track")} ${state.project.tracks.length + 1}`);
    if (!name) return;
    try {
      const result = await call("create_live_midi_track", { args: { name } });
      applyOperation(result);
      state.selectedTrackIndex = result.state.tracks.length - 1;
      state.pianoRollOpen = true;
      renderAll();
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.duplicateTrackButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!track) return;
    try {
      const result = await call("duplicate_track", { args: { index: track.index } });
      applyOperation(result);
      state.selectedTrackIndex = result.state.tracks.length - 1;
      renderAll();
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.exportMixButton.addEventListener("click", async () => {
    const path = window.prompt(tr("exportPrompt"), state.exportPath);
    if (!path) return;
    state.exportPath = path;
    try {
      applyOperation(await call("export_mix", { args: { path } }));
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.saveProjectButton.addEventListener("click", async () => {
    const path = window.prompt(tr("savePrompt"), state.projectPath);
    if (!path) return;
    state.projectPath = path;
    try {
      applyOperation(await call("save_project", { args: { path } }));
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.loadProjectButton.addEventListener("click", async () => {
    const path = window.prompt(tr("openPrompt"), state.projectPath);
    if (!path) return;
    state.projectPath = path;
    try {
      const result = await call("open_project", { args: { path } });
      applyOperation(result);
      state.selectedTrackIndex = 0;
      state.pianoRollOpen = false;
      clearNoteEditingState();
      await Promise.all([refreshKeyboardPorts(), refreshSoundfonts()]);
      renderAll();
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.clearPatternButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!isMidiTrack(track)) {
      setStatus(tr("selectMidiTrack"));
      return;
    }
    await commitNotes([], 2.0);
  });

  elements.fillPatternButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!isMidiTrack(track)) {
      setStatus(tr("selectMidiTrack"));
      return;
    }
    await commitNotes(demoNotes(), 2.0);
  });

  elements.trackNameInput.addEventListener("change", async (event) => {
    const track = selectedTrack();
    if (!track) return;
    await patchTrack(track.index, { name: event.target.value || track.name });
  });

  elements.gainInput.addEventListener("input", async (event) => {
    const track = selectedTrack();
    if (!track) return;
    await patchTrack(track.index, { gain: Number(event.target.value) });
  });

  elements.speedInput.addEventListener("input", async (event) => {
    const track = selectedTrack();
    if (!track) return;
    await patchTrack(track.index, { speed: Number(event.target.value) });
  });

  elements.pitchInput.addEventListener("input", async (event) => {
    const track = selectedTrack();
    if (!track) return;
    await patchTrack(track.index, { pitch_semitones: Number(event.target.value) });
  });

  elements.offsetInput.addEventListener("input", async (event) => {
    const track = selectedTrack();
    if (!track) return;
    await patchTrack(track.index, { offset_seconds: Number(event.target.value) });
  });

  elements.panInput.addEventListener("input", () => {
    setStatus("Pan UI is present, but backend pan is not wired yet");
  });

  elements.colorInput.addEventListener("input", renderAll);

  elements.connectKeyboardButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!track) return;
    const portIndex = Number(elements.keyboardPortSelect.value);
    if (Number.isNaN(portIndex)) {
      setStatus(tr("noMidiPorts"));
      return;
    }

    try {
      applyOperation(
        await call("connect_keyboard", { args: { port_index: portIndex, track_index: track.index } }),
      );
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.disconnectKeyboardButton.addEventListener("click", async () => {
    try {
      applyOperation(await call("disconnect_keyboard"));
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.setBasicInstrumentButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!isMidiTrack(track)) {
      setStatus(tr("selectMidiTrack"));
      return;
    }
    try {
      applyOperation(
        await call("set_track_instrument", {
          args: { index: track.index, instrument_kind: "basic", soundfont_path: null },
        }),
      );
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.setSoundfontButton.addEventListener("click", async () => {
    const track = selectedTrack();
    if (!isMidiTrack(track)) {
      setStatus(tr("selectMidiTrack"));
      return;
    }
    const defaultPath = track.soundfont_path || state.soundfonts[0]?.path || "test/src/UprightPianoKW-small-20190703.sf2";
    const path = window.prompt(tr("soundfontPrompt"), defaultPath);
    if (!path) return;
    try {
      applyOperation(
        await call("set_track_instrument", {
          args: { index: track.index, instrument_kind: "soundfont", soundfont_path: path },
        }),
      );
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });

  elements.timelineScale.addEventListener("pointerdown", handleTimelinePointerDown);
}

function bindLayoutControls() {
  elements.uiScaleInput.addEventListener("input", () => {
    applyLayout({
      uiScale: clamp(0.7, 1.1, Number(elements.uiScaleInput.value) || DEFAULT_LAYOUT.uiScale),
    });
    saveLayoutPrefs();
  });

  elements.resetLayoutButton.addEventListener("click", () => {
    applyLayout(DEFAULT_LAYOUT);
    saveLayoutPrefs();
    setStatus("layout reset");
  });

  bindResizer(elements.sidebarResizer, "x", (deltaX) => {
    const current = layoutValue("--sidebar-width", DEFAULT_LAYOUT.sidebarWidth);
    applyLayout({ sidebarWidth: clamp(200, 420, current + deltaX) });
  });

  bindResizer(elements.mixerResizer, "x", (deltaX) => {
    const current = layoutValue("--mixer-width", DEFAULT_LAYOUT.mixerWidth);
    applyLayout({ mixerWidth: clamp(220, 520, current - deltaX) });
  });

  bindResizer(elements.trackHeaderResizer, "x", (deltaX) => {
    const current = layoutValue("--track-header-width", DEFAULT_LAYOUT.trackHeaderWidth);
    applyLayout({ trackHeaderWidth: clamp(140, 320, current + deltaX) });
  });

  bindResizer(elements.dockResizer, "y", (deltaY) => {
    const current = layoutValue("--bottom-dock-height", DEFAULT_LAYOUT.bottomDockHeight);
    applyLayout({ bottomDockHeight: clamp(160, 420, current - deltaY) });
  });
}

function bindResizer(handle, axis, onDelta) {
  handle.addEventListener("pointerdown", (event) => {
    event.preventDefault();
    const startX = event.clientX;
    const startY = event.clientY;

    const onMove = (moveEvent) => {
      const delta = axis === "x" ? moveEvent.clientX - startX : moveEvent.clientY - startY;
      onDelta(delta);
      saveLayoutPrefs();
    };

    const onUp = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  });
}

function loadLayoutPrefs() {
  let stored = {};
  try {
    stored = JSON.parse(window.localStorage.getItem(LAYOUT_STORAGE_KEY) || "{}");
  } catch {
    stored = {};
  }
  applyLayout({ ...DEFAULT_LAYOUT, ...stored });
}

function saveLayoutPrefs() {
  const payload = {
    uiScale: layoutScale(),
    sidebarWidth: layoutValue("--sidebar-width", DEFAULT_LAYOUT.sidebarWidth),
    bottomDockHeight: layoutValue("--bottom-dock-height", DEFAULT_LAYOUT.bottomDockHeight),
    mixerWidth: layoutValue("--mixer-width", DEFAULT_LAYOUT.mixerWidth),
    trackHeaderWidth: layoutValue("--track-header-width", DEFAULT_LAYOUT.trackHeaderWidth),
  };
  window.localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify(payload));
}

function applyLayout(layout) {
  if (layout.uiScale != null) {
    document.documentElement.style.setProperty("--ui-scale", String(layout.uiScale));
    elements.uiScaleInput.value = String(layout.uiScale);
    elements.uiScaleValue.textContent = `${Math.round(layout.uiScale * 100)}%`;
  }
  if (layout.sidebarWidth != null) {
    document.documentElement.style.setProperty("--sidebar-width", `${Math.round(layout.sidebarWidth)}px`);
  }
  if (layout.bottomDockHeight != null) {
    document.documentElement.style.setProperty("--bottom-dock-height", `${Math.round(layout.bottomDockHeight)}px`);
  }
  if (layout.mixerWidth != null) {
    document.documentElement.style.setProperty("--mixer-width", `${Math.round(layout.mixerWidth)}px`);
  }
  if (layout.trackHeaderWidth != null) {
    document.documentElement.style.setProperty("--track-header-width", `${Math.round(layout.trackHeaderWidth)}px`);
  }
}

function layoutValue(name, fallback) {
  const value = parseFloat(getComputedStyle(document.documentElement).getPropertyValue(name));
  return Number.isFinite(value) ? value : fallback;
}

function layoutScale() {
  const value = parseFloat(getComputedStyle(document.documentElement).getPropertyValue("--ui-scale"));
  return Number.isFinite(value) ? value : DEFAULT_LAYOUT.uiScale;
}

function handleKeydown(event) {
  const tag = document.activeElement?.tagName?.toLowerCase();
  const editable =
    tag === "input" || tag === "textarea" || document.activeElement?.isContentEditable;
  if (editable) return;

  if (event.code === "Space") {
    event.preventDefault();
    if (state.project.playback.running) {
      stopPlayback();
    } else {
      call("play")
        .then(applyOperation)
        .catch((error) => setStatus(stringifyError(error)));
    }
  }
}

async function stopPlayback() {
  try {
    applyOperation(await call("stop"));
  } catch (error) {
    setStatus(stringifyError(error));
  }
}

async function toggleRecording() {
  try {
    if (state.project.recording?.active) {
      applyOperation(await call("stop_recording"));
      return;
    }

    const track = selectedTrack();
    if (!track) {
      setStatus(tr("noTrackSelected"));
      return;
    }
    if (track.kind !== "midi-live") {
      setStatus(tr("selectLiveTrackForRecord"));
      return;
    }
    applyOperation(await call("start_recording", { args: { track_index: track.index } }));
  } catch (error) {
    setStatus(stringifyError(error));
  }
}

async function detectLocale() {
  try {
    return normalizeLocale(await call("system_locale"));
  } catch {
    return normalizeLocale(globalThis.navigator?.language || "en");
  }
}

async function refreshProjectState() {
  state.project = await call("project_state");
  state.lastPlaybackRunning = state.project.playback.running;
  clampSelectedTrack();
  if (!isMidiTrack(selectedTrack())) {
    state.pianoRollOpen = false;
  }
  renderAll();
}

async function refreshKeyboardPorts() {
  try {
    state.keyboardPorts = await call("list_keyboard_ports");
    renderInspector();
  } catch (error) {
    setStatus(`Keyboard list failed: ${stringifyError(error)}`);
  }
}

async function refreshSoundfonts() {
  try {
    state.soundfonts = await call("list_soundfonts");
    renderSoundfontLibrary();
  } catch (error) {
    setStatus(`SoundFont list failed: ${stringifyError(error)}`);
  }
}

function renderAll() {
  renderTransport();
  renderArrangement();
  renderMixer();
  renderPianoRoll();
  renderInspector();
  renderSoundfontLibrary();
  renderStatus();
}

function renderTransport() {
  const playback = state.project.playback;
  elements.playButton.textContent = playback.running
    ? playback.paused
      ? tr("resume")
      : tr("pause")
    : tr("play");
  elements.recordButton.textContent = state.project.recording?.active ? tr("recording") : tr("record");
  elements.recordButton.classList.toggle("active", Boolean(state.project.recording?.active));
}

function renderArrangement() {
  const totalSeconds = Math.max(8, state.project.transport.project_duration_seconds || 8);
  const timelineWidth = Math.max(960, totalSeconds * ARRANGE_PPS);
  const playheadLeft = state.project.transport.playhead_seconds * ARRANGE_PPS;

  elements.arrangementCanvas.style.width = `${timelineWidth}px`;
  elements.arrangementGrid.style.width = `${timelineWidth}px`;
  elements.timelineScale.style.width = `${timelineWidth}px`;
  elements.playheadMarker.style.left = `${playheadLeft}px`;
  elements.playheadMarker.style.display = "block";
  elements.rulerMeta.textContent = formatTime(state.project.transport.playhead_seconds);

  renderTimeline(totalSeconds, timelineWidth);
  renderLoopOverlay();

  elements.arrangementGrid.innerHTML = "";
  if (state.project.tracks.length === 0) {
    const empty = document.createElement("div");
    empty.className = "workspace-empty";
    empty.textContent = tr("noTracks");
    elements.arrangementGrid.appendChild(empty);
    return;
  }

  for (const track of state.project.tracks) {
    const row = document.createElement("div");
    row.className = "track-row";

    const header = document.createElement("button");
    header.type = "button";
    header.className = `track-header${track.index === state.selectedTrackIndex ? " active" : ""}`;
    header.innerHTML = `
      <div class="track-header-top">
        <span class="track-name">${escapeHtml(track.name)}</span>
        <span class="track-badge">${escapeHtml(track.kind)}</span>
      </div>
      <div class="track-controls">
        <span class="track-chip">${escapeHtml(track.instrument)}</span>
        <span class="track-chip">${tr("gain")} ${track.gain.toFixed(2)}</span>
        <span class="track-chip">${tr("speed")} ${track.speed.toFixed(2)}x</span>
      </div>
      <div class="track-summary">
        <span class="track-chip">${track.notes.length} ${tr("notes")}</span>
        <span class="track-chip">${tr("offset")} ${track.offset_seconds.toFixed(2)}s</span>
        <span class="track-chip">${trackStateLabel(track)}</span>
      </div>
    `;
    header.addEventListener("click", () => {
      state.selectedTrackIndex = track.index;
      if (!isMidiTrack(track)) state.pianoRollOpen = false;
      renderAll();
    });

    const lane = document.createElement("div");
    lane.className = "clip-lane";
    lane.style.width = `${timelineWidth}px`;
    lane.style.backgroundSize = `${barSeconds() * ARRANGE_PPS}px 100%, 100% 31px`;

    const clip = document.createElement("button");
    clip.type = "button";
    clip.className = "clip-block";
    clip.style.left = `${track.offset_seconds * ARRANGE_PPS}px`;
    clip.style.width = `${Math.max(16, track.duration_seconds * ARRANGE_PPS)}px`;
    clip.style.background = buildClipGradient(track.index);
    clip.innerHTML = `
      <div class="clip-content">
        <strong>${escapeHtml(track.name)}</strong>
        <span>${escapeHtml(track.kind)} / ${escapeHtml(track.instrument)}</span>
      </div>
      <span class="clip-trim-handle left"></span>
      <span class="clip-trim-handle right"></span>
    `;
    clip.addEventListener("click", () => {
      state.selectedTrackIndex = track.index;
      renderAll();
    });
    clip.addEventListener("dblclick", () => openPianoRollForTrack(track));
    clip.addEventListener("pointerdown", (event) => handleClipPointerDown(event, track, "move"));
    clip.querySelector(".clip-trim-handle.left").addEventListener("pointerdown", (event) => {
      event.stopPropagation();
      handleClipPointerDown(event, track, "trim-left");
    });
    clip.querySelector(".clip-trim-handle.right").addEventListener("pointerdown", (event) => {
      event.stopPropagation();
      handleClipPointerDown(event, track, "trim-right");
    });

    lane.appendChild(clip);
    row.appendChild(header);
    row.appendChild(lane);
    elements.arrangementGrid.appendChild(row);
  }

  followArrangementPlayhead();
}

function renderTimeline(totalSeconds, timelineWidth) {
  elements.timelineScale.innerHTML = "";
  const barLen = barSeconds();

  for (let second = 0; second <= Math.ceil(totalSeconds); second += 1) {
    const marker = document.createElement("div");
    marker.className = "timeline-marker";
    marker.style.left = `${second * ARRANGE_PPS}px`;
    marker.textContent = second % Math.max(1, Math.round(barLen)) === 0 ? `${tr("bar")} ${Math.floor(second / barLen) + 1}` : formatShortTime(second);
    elements.timelineScale.appendChild(marker);
  }

  elements.timelineScale.style.backgroundSize = `${barLen * ARRANGE_PPS}px 100%`;
  elements.timelineScale.style.minWidth = `${timelineWidth}px`;
}

function renderLoopOverlay() {
  const loop = state.loopDraft || state.project.transport;
  const enabled = loop.loop_enabled && loop.loop_end_seconds > loop.loop_start_seconds;
  if (!enabled) {
    elements.loopOverlay.style.display = "none";
    return;
  }
  elements.loopOverlay.style.display = "block";
  elements.loopOverlay.style.left = `${loop.loop_start_seconds * ARRANGE_PPS}px`;
  elements.loopOverlay.style.width = `${Math.max(2, (loop.loop_end_seconds - loop.loop_start_seconds) * ARRANGE_PPS)}px`;
}

function renderMixer() {
  elements.mixerStrips.innerHTML = "";
  if (state.project.tracks.length === 0) {
    const empty = document.createElement("div");
    empty.className = "workspace-empty";
    empty.textContent = tr("noTracks");
    elements.mixerStrips.appendChild(empty);
    return;
  }

  for (const track of state.project.tracks) {
    const strip = document.createElement("div");
    strip.className = `mixer-strip${track.index === state.selectedTrackIndex ? " active" : ""}`;
    const meterLevel = Math.max(8, Math.min(100, Math.round(track.gain * 50 + track.notes.length)));
    strip.innerHTML = `
      <div class="mixer-strip-title">${escapeHtml(track.name)}</div>
      <div class="strip-mini">${escapeHtml(track.instrument)}</div>
      <div class="mixer-fader-zone">
        <div class="meter"><div class="meter-fill" style="height:${meterLevel}%"></div></div>
        <div class="mixer-fader-wrap">
          <input class="mixer-fader" type="range" min="0" max="1.5" step="0.01" value="${track.gain}" />
        </div>
      </div>
      <div class="strip-mini">${track.gain.toFixed(2)}</div>
      <div class="mixer-strip-buttons">
        <button class="transport-btn ghost small">${track.mute ? tr("unmute") : tr("mute")}</button>
        <button class="transport-btn ghost small">${track.solo ? tr("unsolo") : tr("solo")}</button>
      </div>
    `;
    strip.addEventListener("click", () => {
      state.selectedTrackIndex = track.index;
      renderAll();
    });
    strip.querySelector(".mixer-fader").addEventListener("input", async (event) => {
      await patchTrack(track.index, { gain: Number(event.target.value) });
    });
    const buttons = strip.querySelectorAll("button");
    buttons[0].addEventListener("click", async (event) => {
      event.stopPropagation();
      await patchTrack(track.index, { mute: !track.mute });
    });
    buttons[1].addEventListener("click", async (event) => {
      event.stopPropagation();
      await patchTrack(track.index, { solo: !track.solo });
    });
    elements.mixerStrips.appendChild(strip);
  }
}

function renderPianoRoll() {
  const track = selectedTrack();
  const editable = Boolean(track) && isMidiTrack(track) && state.pianoRollOpen;
  elements.pianoRollShell.classList.toggle("collapsed", !editable);
  elements.pianoRoll.innerHTML = "";

  if (!track) {
    elements.pianoRollHint.textContent = tr("doubleClickOpen");
    elements.pianoRollEmpty.textContent = tr("doubleClickMidi");
    return;
  }

  if (!isMidiTrack(track)) {
    elements.pianoRollHint.textContent = tr("pianoRollAudioHint");
    elements.pianoRollEmpty.textContent = tr("audioNoPianoRoll");
    return;
  }

  elements.pianoRollHint.textContent = track.kind === "midi-live" ? tr("pianoRollLiveHint") : tr("pianoRollEditHint");
  elements.pianoRollEmpty.textContent = tr("doubleClickMidi");

  if (!editable) return;

  const notes = editingNotes(track);
  const range = pianoNoteRange(notes);
  const duration = pianoDuration(track, notes);
  const width = Math.max(960, duration * PIANO_PPS + PIANO_KEY_WIDTH);
  const rowCount = range.max - range.min + 1;
  const height = rowCount * PIANO_ROW_HEIGHT;

  const editor = document.createElement("div");
  editor.className = "piano-roll-editor";

  const scroller = document.createElement("div");
  scroller.className = "piano-roll-scroller";
  const inner = document.createElement("div");
  inner.className = "piano-roll-inner";
  inner.style.width = `${width}px`;
  inner.style.height = `${height}px`;

  const keyColumn = document.createElement("div");
  keyColumn.className = "piano-key-column";
  keyColumn.style.height = `${height}px`;
  for (let note = range.max; note >= range.min; note -= 1) {
    const row = document.createElement("div");
    row.className = "note-row-label";
    row.style.top = `${(range.max - note) * PIANO_ROW_HEIGHT}px`;
    row.style.height = `${PIANO_ROW_HEIGHT}px`;
    row.textContent = midiName(note);
    keyColumn.appendChild(row);
  }

  const grid = document.createElement("div");
  grid.className = "piano-grid-surface";
  grid.style.width = `${width - PIANO_KEY_WIDTH}px`;
  grid.style.height = `${height}px`;
  grid.style.backgroundSize = `${(barSeconds() / 4) * PIANO_PPS}px ${PIANO_ROW_HEIGHT}px`;
  grid.addEventListener("pointerdown", (event) => handlePianoGridPointerDown(event, track, notes, range, width, height));
  grid.addEventListener("dblclick", (event) => handlePianoGridDoubleClick(event, track, notes, range));

  inner.appendChild(keyColumn);
  inner.appendChild(grid);

  notes.forEach((note, index) => {
    const block = document.createElement("div");
    block.className = `piano-note${state.selectedNoteIndices.has(index) ? " selected" : ""}`;
    block.style.left = `${PIANO_KEY_WIDTH + note.start_seconds * PIANO_PPS}px`;
    block.style.top = `${(range.max - note.note) * PIANO_ROW_HEIGHT + 1}px`;
    block.style.width = `${Math.max(10, (note.end_seconds - note.start_seconds) * PIANO_PPS)}px`;
    block.innerHTML = `
      <span class="piano-note-label">${midiName(note.note)}</span>
      <span class="piano-note-resize"></span>
    `;
    block.addEventListener("pointerdown", (event) => {
      const mode = event.target.classList.contains("piano-note-resize") ? "resize" : "move";
      handleNotePointerDown(event, track, index, mode, notes, range);
    });
    inner.appendChild(block);
  });

  if (state.marqueeRect) {
    const box = document.createElement("div");
    box.className = "selection-box";
    box.style.left = `${state.marqueeRect.x}px`;
    box.style.top = `${state.marqueeRect.y}px`;
    box.style.width = `${state.marqueeRect.width}px`;
    box.style.height = `${state.marqueeRect.height}px`;
    inner.appendChild(box);
  }

  scroller.appendChild(inner);

  const velocityLane = document.createElement("div");
  velocityLane.className = "velocity-lane";
  const velocityInner = document.createElement("div");
  velocityInner.className = "velocity-inner";
  velocityInner.style.width = `${width}px`;
  velocityInner.style.height = "96px";
  const velocityGrid = document.createElement("div");
  velocityGrid.className = "velocity-grid";
  velocityGrid.style.backgroundSize = `${(barSeconds() / 4) * PIANO_PPS}px 24px`;
  velocityInner.appendChild(velocityGrid);

  notes.forEach((note, index) => {
    const bar = document.createElement("div");
    bar.className = `velocity-bar${state.selectedNoteIndices.has(index) ? " selected" : ""}`;
    bar.style.left = `${PIANO_KEY_WIDTH + note.start_seconds * PIANO_PPS}px`;
    bar.style.height = `${Math.max(8, (note.velocity / 127) * 92)}px`;
    bar.addEventListener("pointerdown", (event) => handleVelocityPointerDown(event, track, index, notes));
    velocityInner.appendChild(bar);
  });
  velocityLane.appendChild(velocityInner);

  scroller.addEventListener("scroll", () => {
    state.pianoScrollLeft = scroller.scrollLeft;
    state.pianoScrollTop = scroller.scrollTop;
    velocityLane.scrollLeft = scroller.scrollLeft;
  });
  velocityLane.addEventListener("scroll", () => {
    scroller.scrollLeft = velocityLane.scrollLeft;
    state.pianoScrollLeft = velocityLane.scrollLeft;
  });

  editor.appendChild(scroller);
  editor.appendChild(velocityLane);
  elements.pianoRoll.appendChild(editor);
  scroller.scrollLeft = state.pianoScrollLeft;
  scroller.scrollTop = state.pianoScrollTop;
  velocityLane.scrollLeft = state.pianoScrollLeft;
  followPianoPlayhead(scroller);
}

function renderInspector() {
  const track = selectedTrack();
  if (!track) {
    elements.selectedTrackMeta.textContent = tr("noTrackSelected");
    elements.projectSummary.textContent = `0 ${tr("tracksLoaded")}`;
    return;
  }

  elements.selectedTrackMeta.textContent = `${track.name} · ${track.kind}`;
  elements.trackNameInput.value = track.name;
  elements.gainInput.value = String(track.gain);
  elements.panInput.value = "0";
  elements.colorInput.value = colorForTrack(track.index);
  elements.speedInput.value = String(track.speed);
  elements.pitchInput.value = String(track.pitch_semitones);
  elements.offsetInput.value = String(track.offset_seconds);
  elements.trackLengthStat.textContent = `${track.duration_seconds.toFixed(2)} s`;
  elements.trackNoteStat.textContent = String(track.notes.length);
  elements.trackPatternStat.textContent = track.notes.length > 0 ? tr("patternReady") : tr("patternEmpty");
  elements.projectSummary.textContent = `${state.project.tracks.length} ${tr("tracksLoaded")} · ${tr("latency")} ${state.project.latency_mode}`;

  elements.keyboardPortSelect.innerHTML = "";
  if (state.keyboardPorts.length === 0) {
    const option = document.createElement("option");
    option.textContent = tr("noMidiPorts");
    option.value = "";
    elements.keyboardPortSelect.appendChild(option);
  } else {
    state.keyboardPorts.forEach((port, index) => {
      const option = document.createElement("option");
      option.value = String(index);
      option.textContent = port.is_virtual_loopback ? `${port.name} (${tr("loopback")})` : port.name;
      elements.keyboardPortSelect.appendChild(option);
    });
    if (state.project.keyboard_route) {
      elements.keyboardPortSelect.value = String(state.project.keyboard_route.port_index);
    }
  }
}

function renderSoundfontLibrary() {
  elements.soundfontLibrary.innerHTML = "";
  const track = selectedTrack();
  const selectedPath = track?.soundfont_path || null;
  const selectedIsMidi = isMidiTrack(track);

  const basic = document.createElement("button");
  basic.type = "button";
  basic.className = `patch-card${track?.instrument === "basic" ? " active" : ""}`;
  basic.innerHTML = `
    <span class="patch-name">${escapeHtml(tr("defaultSoundfont"))}</span>
    <span class="patch-meta">${escapeHtml(tr("basicInstrumentDesc"))}</span>
  `;
  basic.addEventListener("click", async () => {
    if (!selectedIsMidi) {
      setStatus(tr("selectMidiTrack"));
      return;
    }
    try {
      applyOperation(
        await call("set_track_instrument", {
          args: { index: track.index, instrument_kind: "basic", soundfont_path: null },
        }),
      );
    } catch (error) {
      setStatus(stringifyError(error));
    }
  });
  elements.soundfontLibrary.appendChild(basic);

  if (state.soundfonts.length === 0) {
    const empty = document.createElement("div");
    empty.className = "library-empty";
    empty.textContent = tr("noSoundfontsFound");
    elements.soundfontLibrary.appendChild(empty);
    return;
  }

  for (const sf2 of state.soundfonts) {
    const card = document.createElement("button");
    card.type = "button";
    card.className = `patch-card${selectedPath === sf2.path ? " selected-sf2" : ""}`;
    card.innerHTML = `
      <span class="patch-name">${escapeHtml(sf2.name)}</span>
      <span class="patch-meta">${escapeHtml(tr("detectedInFolder"))}</span>
    `;
    card.addEventListener("click", async () => {
      if (!selectedIsMidi) {
        setStatus(tr("selectMidiTrack"));
        return;
      }
      try {
        applyOperation(
          await call("set_track_instrument", {
            args: { index: track.index, instrument_kind: "soundfont", soundfont_path: sf2.path },
          }),
        );
        setStatus(`${tr("sf2Applied")}: ${sf2.name}`);
      } catch (error) {
        setStatus(stringifyError(error));
      }
    });
    elements.soundfontLibrary.appendChild(card);
  }
}

function renderStatus() {
  elements.statusText.textContent = state.status;

  if (state.project.recording?.active) {
    elements.playheadText.textContent =
      `${tr("recordingStatus")} · ${state.project.recording.elapsed_seconds.toFixed(1)}s · ${state.project.recording.note_count} ${tr("notes")}`;
    return;
  }

  if (state.project.playback.running) {
    elements.playheadText.textContent =
      `${tr("playbackRunning")} ${formatTime(state.project.playback.position_seconds)} · ${state.project.playback.sample_rate} Hz / ${state.project.playback.channels} ch`;
  } else if (state.project.playback.paused) {
    elements.playheadText.textContent = tr("playbackPaused");
  } else {
    elements.playheadText.textContent = `${tr("playbackStopped")} · ${formatTime(state.project.transport.playhead_seconds)}`;
  }
}

function handleTimelinePointerDown(event) {
  const rect = elements.timelineScale.getBoundingClientRect();
  const seconds = xToSeconds(event.clientX - rect.left + elements.arrangementViewport.scrollLeft);

  if (event.button === 2) {
    const startSeconds = clampSeconds(seconds);
    state.loopDraft = {
      loop_enabled: true,
      loop_start_seconds: startSeconds,
      loop_end_seconds: startSeconds,
    };
    renderLoopOverlay();

    const onMove = (moveEvent) => {
      const moveSeconds = xToSeconds(
        moveEvent.clientX - rect.left + elements.arrangementViewport.scrollLeft,
      );
      state.loopDraft = normalizeLoopDraft(startSeconds, moveSeconds);
      renderLoopOverlay();
    };

    const onUp = async () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      const draft = state.loopDraft;
      state.loopDraft = null;
      try {
        if (!draft || draft.loop_end_seconds - draft.loop_start_seconds < 0.05) {
          applyOperation(
            await call("set_loop", { args: { enabled: false, start_seconds: 0, end_seconds: 0 } }),
          );
          setStatus(tr("loopDisabled"));
          return;
        }
        applyOperation(
          await call("set_loop", {
            args: {
              enabled: true,
              start_seconds: draft.loop_start_seconds,
              end_seconds: draft.loop_end_seconds,
            },
          }),
        );
        setStatus(tr("loopEnabled"));
      } catch (error) {
        setStatus(stringifyError(error));
      }
    };

    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return;
  }

  call("set_playhead", { args: { seconds: clampSeconds(seconds) } })
    .then(applyOperation)
    .catch((error) => setStatus(stringifyError(error)));
}

function handleClipPointerDown(event, track, mode) {
  if (event.button !== 0) return;
  event.preventDefault();
  const originalOffset = track.offset_seconds;
  const originalTrimStart = track.trim_start_seconds;
  const originalTrimEnd = track.trim_end_seconds;
  const element = mode === "move" ? event.currentTarget : event.currentTarget.parentElement;
  let pendingLeft = parseFloat(element.style.left || "0");
  let pendingWidth = parseFloat(element.style.width || "0");
  let rafId = 0;
  element.classList.add("dragging");

  const flushVisual = () => {
    rafId = 0;
    element.style.left = `${pendingLeft}px`;
    element.style.width = `${pendingWidth}px`;
  };

  const onMove = (moveEvent) => {
    const deltaSeconds = (moveEvent.clientX - event.clientX) / ARRANGE_PPS;
    if (mode === "move") {
      const next = Math.max(0, originalOffset + deltaSeconds);
      pendingLeft = next * ARRANGE_PPS;
    } else if (mode === "trim-left") {
      const nextStart = Math.max(0, Math.min(originalTrimEnd - NOTE_MIN_DURATION, originalTrimStart + deltaSeconds));
      const nextOffset = Math.max(0, originalOffset + (nextStart - originalTrimStart));
      pendingLeft = nextOffset * ARRANGE_PPS;
      pendingWidth = Math.max(16, (track.duration_seconds - (nextStart - originalTrimStart)) * ARRANGE_PPS);
    } else if (mode === "trim-right") {
      const nextEnd = Math.max(originalTrimStart + NOTE_MIN_DURATION, originalTrimEnd + deltaSeconds);
      pendingWidth = Math.max(16, (track.duration_seconds + (nextEnd - originalTrimEnd)) * ARRANGE_PPS);
    }

    if (!rafId) {
      rafId = window.requestAnimationFrame(flushVisual);
    }
  };

  const onUp = async (upEvent) => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    element.classList.remove("dragging");
    if (rafId) {
      window.cancelAnimationFrame(rafId);
      flushVisual();
    }
    const deltaSeconds = (upEvent.clientX - event.clientX) / ARRANGE_PPS;
    try {
      if (mode === "move") {
        await patchTrack(track.index, { offset_seconds: Math.max(0, originalOffset + deltaSeconds) });
      } else if (mode === "trim-left") {
        const nextStart = Math.max(0, Math.min(originalTrimEnd - NOTE_MIN_DURATION, originalTrimStart + deltaSeconds));
        const nextOffset = Math.max(0, originalOffset + (nextStart - originalTrimStart));
        await patchTrack(track.index, {
          offset_seconds: nextOffset,
          trim_start_seconds: nextStart,
        });
      } else if (mode === "trim-right") {
        const nextEnd = Math.max(originalTrimStart + NOTE_MIN_DURATION, originalTrimEnd + deltaSeconds);
        await patchTrack(track.index, { trim_end_seconds: nextEnd });
      }
    } catch (error) {
      setStatus(stringifyError(error));
    }
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
}

function handlePianoGridPointerDown(event, track, notes, range, width, height) {
  if (event.button !== 0) return;
  const innerRect = event.currentTarget.parentElement.getBoundingClientRect();
  const startX = event.clientX - innerRect.left;
  const startY = event.clientY - innerRect.top;
  state.marqueeRect = { x: startX, y: startY, width: 0, height: 0 };
  renderPianoRoll();

  const onMove = (moveEvent) => {
    const x = moveEvent.clientX - innerRect.left;
    const y = moveEvent.clientY - innerRect.top;
    state.marqueeRect = {
      x: Math.min(startX, x),
      y: Math.min(startY, y),
      width: Math.abs(x - startX),
      height: Math.abs(y - startY),
    };
    renderPianoRoll();
  };

  const onUp = () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    const rect = state.marqueeRect;
    state.marqueeRect = null;
    if (!rect || rect.width < 4 || rect.height < 4) {
      state.selectedNoteIndices.clear();
      renderPianoRoll();
      return;
    }
    state.selectedNoteIndices = new Set(
      notes
        .map((note, index) => ({ index, left: PIANO_KEY_WIDTH + note.start_seconds * PIANO_PPS, top: (range.max - note.note) * PIANO_ROW_HEIGHT, width: Math.max(10, (note.end_seconds - note.start_seconds) * PIANO_PPS) }))
        .filter((item) => rectIntersects(rect, item))
        .map((item) => item.index),
    );
    renderPianoRoll();
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
}

async function handlePianoGridDoubleClick(event, track, notes, range) {
  const rect = event.currentTarget.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  const note = range.max - Math.floor(y / PIANO_ROW_HEIGHT);
  const start = quantizeSeconds(x / PIANO_PPS);
  const next = cloneNotes(notes);
  next.push({
    note: clampMidi(note),
    velocity: 104,
    channel: 0,
    start_seconds: start,
    end_seconds: start + 0.25,
  });
  sortNotes(next);
  await commitNotes(next, noteDuration(next));
}

function handleNotePointerDown(event, track, index, mode, notes, range) {
  if (event.button !== 0) return;
  event.preventDefault();
  event.stopPropagation();

  if (!event.shiftKey && !state.selectedNoteIndices.has(index)) {
    state.selectedNoteIndices = new Set([index]);
  } else if (event.shiftKey) {
    if (state.selectedNoteIndices.has(index)) {
      state.selectedNoteIndices.delete(index);
    } else {
      state.selectedNoteIndices.add(index);
    }
  } else if (state.selectedNoteIndices.size === 0) {
    state.selectedNoteIndices = new Set([index]);
  }

  const activeIndices = state.selectedNoteIndices.has(index) ? [...state.selectedNoteIndices] : [index];
  const baseNotes = cloneNotes(notes);
  const originX = event.clientX;
  const originY = event.clientY;

  const onMove = (moveEvent) => {
    const deltaSeconds = quantizeSeconds((moveEvent.clientX - originX) / PIANO_PPS);
    const deltaNotes = Math.round((originY - moveEvent.clientY) / PIANO_ROW_HEIGHT);
    const draft = cloneNotes(baseNotes);

    for (const noteIndex of activeIndices) {
      const note = draft[noteIndex];
      if (!note) continue;
      if (mode === "move") {
        const duration = note.end_seconds - note.start_seconds;
        note.start_seconds = Math.max(0, quantizeSeconds(baseNotes[noteIndex].start_seconds + deltaSeconds));
        note.end_seconds = note.start_seconds + duration;
        note.note = clampMidi(baseNotes[noteIndex].note + deltaNotes);
      } else {
        note.end_seconds = Math.max(
          note.start_seconds + NOTE_MIN_DURATION,
          quantizeSeconds(baseNotes[noteIndex].end_seconds + deltaSeconds),
        );
      }
    }
    state.noteDraft = { trackIndex: track.index, notes: draft };
    renderPianoRoll();
  };

  const onUp = async () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    const next = editingNotes(track);
    await commitNotes(next, noteDuration(next));
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
  renderPianoRoll();
}

function handleVelocityPointerDown(event, track, index, notes) {
  if (event.button !== 0) return;
  event.preventDefault();
  const laneRect = event.currentTarget.parentElement.getBoundingClientRect();
  const baseNotes = cloneNotes(notes);

  const applyVelocity = (clientY) => {
    const value = 127 - Math.round(((clientY - laneRect.top) / laneRect.height) * 127);
    const velocity = clamp(1, 127, value);
    const next = cloneNotes(baseNotes);
    next[index].velocity = velocity;
    state.noteDraft = { trackIndex: track.index, notes: next };
    renderPianoRoll();
  };

  applyVelocity(event.clientY);

  const onMove = (moveEvent) => applyVelocity(moveEvent.clientY);
  const onUp = async () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    await commitNotes(editingNotes(track), noteDuration(editingNotes(track)));
  };

  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onUp);
}

async function patchTrack(index, patch) {
  try {
    applyOperation(await call("patch_track", { args: { index, patch } }));
  } catch (error) {
    setStatus(stringifyError(error));
  }
}

async function commitNotes(notes, durationSeconds) {
  const track = selectedTrack();
  if (!track || !isMidiTrack(track)) {
    setStatus(tr("selectMidiTrack"));
    return;
  }
  try {
    const result = await call("set_track_notes", {
      args: {
        index: track.index,
        notes: sortNotes(cloneNotes(notes)),
        duration_seconds: Math.max(0.25, durationSeconds),
      },
    });
    state.noteDraft = null;
    state.pianoRollOpen = true;
    applyOperation(result);
  } catch (error) {
    setStatus(stringifyError(error));
  }
}

function applyOperation(result) {
  state.project = result.state;
  state.lastPlaybackRunning = state.project.playback.running;
  clampSelectedTrack();
  if (!isMidiTrack(selectedTrack())) {
    state.pianoRollOpen = false;
    clearNoteEditingState();
  }
  setStatus(result.message);
  renderAll();
}

function selectedTrack() {
  return state.project.tracks[state.selectedTrackIndex] || state.project.tracks[0] || null;
}

function clampSelectedTrack() {
  if (state.project.tracks.length === 0) {
    state.selectedTrackIndex = 0;
  } else if (state.selectedTrackIndex >= state.project.tracks.length) {
    state.selectedTrackIndex = state.project.tracks.length - 1;
  }
}

function openPianoRollForTrack(track) {
  state.selectedTrackIndex = track.index;
  if (!isMidiTrack(track)) {
    state.pianoRollOpen = false;
    setStatus(tr("audioNoPianoRoll"));
    renderAll();
    return;
  }
  state.pianoRollOpen = true;
  setStatus(tr("pianoRollEditHint"));
  renderAll();
}

function isMidiTrack(track) {
  return Boolean(track) && (track.kind === "midi" || track.kind === "midi-live");
}

function trackStateLabel(track) {
  if (track.mute) return tr("muted");
  if (track.solo) return tr("solo");
  return tr("active");
}

function followArrangementPlayhead() {
  if (!state.project.playback.running) return;
  const viewport = elements.arrangementViewport;
  const playheadX = state.project.transport.playhead_seconds * ARRANGE_PPS;
  const viewLeft = viewport.scrollLeft;
  const viewRight = viewLeft + viewport.clientWidth;
  const safeLeft = viewLeft + viewport.clientWidth * 0.22;
  const safeRight = viewLeft + viewport.clientWidth * 0.72;

  if (playheadX < safeLeft || playheadX > safeRight) {
    const target = Math.max(0, playheadX - viewport.clientWidth * 0.36);
    viewport.scrollLeft += (target - viewport.scrollLeft) * 0.28;
  }
}

function followPianoPlayhead(scroller) {
  if (!state.project.playback.running || !state.pianoRollOpen) return;
  const playheadX = state.project.transport.playhead_seconds * PIANO_PPS;
  const safeLeft = scroller.scrollLeft + scroller.clientWidth * 0.2;
  const safeRight = scroller.scrollLeft + scroller.clientWidth * 0.7;
  if (playheadX < safeLeft || playheadX > safeRight) {
    const target = Math.max(0, playheadX - scroller.clientWidth * 0.34);
    scroller.scrollLeft += (target - scroller.scrollLeft) * 0.28;
    state.pianoScrollLeft = scroller.scrollLeft;
  }
}

function startPolling() {
  if (state.pollHandle) return;
  state.pollHandle = window.setInterval(async () => {
    if (state.pollInFlight) return;
    if (!state.project.playback.running && !state.project.recording?.active) return;
    state.pollInFlight = true;
    try {
      const latest = await call("project_state");
      const wasRunning = state.lastPlaybackRunning;
      state.project = latest;
      clampSelectedTrack();
      renderAll();

      if (
        wasRunning &&
        !latest.playback.running &&
        latest.transport.loop_enabled &&
        latest.transport.loop_end_seconds > latest.transport.loop_start_seconds
      ) {
        const start_seconds = latest.transport.loop_start_seconds;
        const end_seconds = latest.transport.loop_end_seconds;
        applyOperation(await call("set_playhead", { args: { seconds: start_seconds } }));
        applyOperation(
          await call("set_loop", { args: { enabled: true, start_seconds, end_seconds } }),
        );
        applyOperation(await call("play"));
      }
      state.lastPlaybackRunning = latest.playback.running;
    } catch {
      // Ignore transient polling failures.
    } finally {
      state.pollInFlight = false;
    }
  }, POLL_INTERVAL_MS);
}

function editingNotes(track) {
  if (state.noteDraft && state.noteDraft.trackIndex === track.index) {
    return state.noteDraft.notes;
  }
  return track.notes;
}

function clearNoteEditingState() {
  state.noteDraft = null;
  state.selectedNoteIndices.clear();
  state.marqueeRect = null;
}

function pianoNoteRange(notes) {
  if (notes.length === 0) {
    return { min: PIANO_DEFAULT_MIN, max: PIANO_DEFAULT_MAX };
  }
  const noteMin = Math.min(...notes.map((note) => note.note), PIANO_DEFAULT_MIN);
  const noteMax = Math.max(...notes.map((note) => note.note), PIANO_DEFAULT_MAX);
  return { min: Math.max(24, noteMin - 1), max: Math.min(108, noteMax + 1) };
}

function pianoDuration(track, notes) {
  return Math.max(track.duration_seconds, noteDuration(notes), 2.0);
}

function noteDuration(notes) {
  return notes.reduce((max, note) => Math.max(max, note.end_seconds), 2.0);
}

function cloneNotes(notes) {
  return notes.map((note) => ({ ...note }));
}

function sortNotes(notes) {
  notes.sort(
    (left, right) =>
      left.start_seconds - right.start_seconds || left.note - right.note || left.end_seconds - right.end_seconds,
  );
  return notes;
}

function normalizeLoopDraft(a, b) {
  return {
    loop_enabled: true,
    loop_start_seconds: clampSeconds(Math.min(a, b)),
    loop_end_seconds: clampSeconds(Math.max(a, b)),
  };
}

function rectIntersects(a, b) {
  return a.x < b.left + b.width && a.x + a.width > b.left && a.y < b.top + PIANO_ROW_HEIGHT && a.y + a.height > b.top;
}

function barSeconds() {
  const bpm = Math.max(1, Number(elements.tempoInput.value) || 120);
  return (60 / bpm) * 4;
}

function xToSeconds(x) {
  return x / ARRANGE_PPS;
}

function clampSeconds(seconds) {
  return clamp(0, Math.max(2, state.project.transport.project_duration_seconds || 2), seconds);
}

function quantizeSeconds(seconds) {
  const step = barSeconds() / 16;
  return Math.round(seconds / step) * step;
}

function clamp(min, max, value) {
  return Math.min(max, Math.max(min, value));
}

function clampMidi(value) {
  return clamp(12, 108, value);
}

function midiName(note) {
  const names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
  const octave = Math.floor(note / 12) - 1;
  return `${names[note % 12]}${octave}`;
}

function formatTime(seconds) {
  const totalMs = Math.max(0, Math.round(seconds * 1000));
  const minutes = String(Math.floor(totalMs / 60000)).padStart(2, "0");
  const secs = String(Math.floor((totalMs % 60000) / 1000)).padStart(2, "0");
  const ms = String(totalMs % 1000).padStart(3, "0");
  return `${minutes}:${secs}.${ms}`;
}

function formatShortTime(seconds) {
  return `${seconds.toFixed(0)}s`;
}

function buildClipGradient(index) {
  const color = colorForTrack(index);
  return `linear-gradient(180deg, ${hexToRgba(color, 0.95)} 0%, ${hexToRgba(color, 0.56)} 100%)`;
}

function colorForTrack(index) {
  return ["#7ca6ff", "#8fd58a", "#e0a7ff", "#ffb86c", "#69d2c9", "#ff8ba7"][index % 6];
}

function hexToRgba(hex, alpha) {
  const safe = hex.replace("#", "");
  const num = Number.parseInt(safe, 16);
  const r = (num >> 16) & 255;
  const g = (num >> 8) & 255;
  const b = num & 255;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function setStatus(message) {
  state.status = message;
  renderStatus();
}

function applyI18n() {
  document.documentElement.lang = state.locale;
  document.title = tr("appTitle");
  document.querySelectorAll("[data-i18n]").forEach((node) => {
    node.textContent = tr(node.getAttribute("data-i18n"));
  });
}

function normalizeLocale(raw) {
  const safe = String(raw || "en").toLowerCase();
  return safe.startsWith("zh") ? "zh" : "en";
}

function tr(key) {
  return I18N[state.locale]?.[key] ?? I18N.en[key] ?? key;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function stringifyError(error) {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) return String(error.message);
  return String(error);
}

async function call(command, payload = {}) {
  const invoke = window.__TAURI__?.core?.invoke ?? window.__TAURI_INTERNALS__?.invoke;
  if (!invoke) {
    throw new Error("Tauri invoke bridge is unavailable");
  }
  return invoke(command, payload);
}
