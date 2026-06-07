# OpenDaw

一个用 Rust 编写的 DAW 原型，目前先提供命令行混音和交互式音轨 shell：

- `-shell` 交互式音轨管理
- 基于 `cpal` 的跨平台实时播放
- 基于成熟库 `hound` 读取和写出 WAV
- 基于 `midly` 读取 MIDI
- 基于 `rayon` 并行读取轨道与并行分块混音
- 读取多个 `16-bit PCM WAV` 或 `32-bit float WAV`
- 支持加载 `.mid/.midi` 并使用内建合成器播放/导出
- 支持给 MIDI 轨道绑定 `SoundFont (.sf2)`
- 每个输入轨道可单独设置音量
- 自动按最长轨道补静音混音
- 支持单声道和双声道轨道混合
- 输出新的 `16-bit PCM WAV`

## 交互式 Shell

```bash
cargo run --release -- -shell
```

可用命令：

```text
help
load <path.wav> [gain]
midi-live <name>
keyboard list
keyboard connect <port_index> <track_index>
keyboard disconnect
keyboard status
keyboard monitor <port_index>
keyboard monitor stop
keyboard monitor status
record start <track_index>
record stop
record status
list
gain <index> <gain>
mute <index> [on|off|toggle]
solo <index> [on|off|toggle]
rename <index> <new_name>
offset <index> <seconds>
speed <index> <ratio>
pitch <index> <semitones>
notes <index> <seconds>
soundfont <index> <path.sf2|basic>
play
pause
resume
stop
latency <low|balanced|safe>
remove <index>
clear
mix <output.wav>
save <project.json>
open <project.json>
quit
```

例如：

```text
opendaw> load test/src/test.wav 0.8
opendaw> load test/src/suzume_no_tojimari.wav 0.6
opendaw> list
opendaw> solo 1 on
opendaw> offset 1 2.5
opendaw> speed 1 1.15
opendaw> pitch 1 3
opendaw> notes 1 10
opendaw> latency low
opendaw> play
opendaw> pause
opendaw> resume
opendaw> stop
opendaw> gain 1 0.7
opendaw> save test/project.json
opendaw> mix test/output/output.wav
```

说明：

- `latency low`：优先更低输出缓冲，适合追求低延迟
- `latency balanced`：默认模式，兼顾延迟和稳定性
- `latency safe`：更大的输出缓冲，更适合避免爆音/掉帧
- `speed`：改变轨道速度，`1.0` 为原速，`2.0` 更快，`0.5` 更慢
- `pitch`：按半音调整音高，`12` 为升高一个八度，`-12` 为降低一个八度
- `notes`：查询某个 MIDI 轨道在指定时间点的活动音符
- `soundfont`：给 MIDI 轨道切换音源；`basic` 使用内建合成器，`path.sf2` 使用 SoundFont
- `midi-live`：新建一个实时演奏用 MIDI 轨道
- `keyboard connect`：把外部 MIDI 键盘路由到指定 MIDI 轨道，并使用该轨道音色实时发声
- `keyboard monitor`：监视某个 MIDI 输入端口的实时事件，用来排查键盘有没有真的发消息进来
- `keyboard list` 会把 `Midi Through` 这类回环口标出来，它不是实体键盘
- `record start`：对已连接键盘的 `midi-live` 轨开始录制，保留力度（velocity）
- `record stop`：停止录制，并把当前 `midi-live` 轨转换成可 `notes` / `mix` / `save` 的普通 MIDI 轨
- `play` 会在后台分段混音，避免在音频回调线程里做重计算
- `midi-live` 轨道只用于实时演奏，不会参与 `play` / `mix`

MIDI 示例：

```text
opendaw> load test/src/Minecraft.mid 0.7
opendaw> soundfont 0 /path/to/piano.sf2
opendaw> notes 0 5
opendaw> speed 0 1.2
opendaw> pitch 0 2
opendaw> mix test/output/minecraft.wav
```

实时 MIDI 键盘示例：

```text
opendaw> midi-live Piano
opendaw> soundfont 0 /path/to/piano.sf2
opendaw> keyboard list
opendaw> keyboard connect 0 0
opendaw> keyboard monitor 0
opendaw> record start 0
opendaw> record stop
opendaw> notes 0 1.5
```

性能说明：

- `mix` 现在改为分段导出，边混音边写文件，不再整首先拼到内存
- `play` 现在改为后台分段混音，音频回调线程只负责取块播放
- 这些优化不会改变混音质量，主要减少启动等待和峰值内存

## 直接命令行混音

```text
<输出文件.wav> <输入1.wav:音量> <输入2.wav:音量> ...
```

例如：

```bash
cargo run --release -- out/song_mix.wav kick.wav:0.9 pad.wav:0.45 lead.wav:0.7
```

更快的运行方式：

```bash
cargo build --release
./target/release/opendaw -shell
./target/release/opendaw out/song_mix.wav kick.wav:0.9 pad.wav:0.45 lead.wav:0.7
```

## 当前限制

- 所有输入必须具有相同的采样率
- 当前重点支持 `mono/stereo` 混音：
  - `mono -> stereo` 会自动复制到左右声道
  - `stereo -> mono` 会自动平均下混
- 输入当前支持 `16-bit PCM WAV` 和 `32-bit float WAV`
- MIDI 当前使用内建简单合成器渲染为音频，不依赖外部 soundfont
- 如果给 MIDI 轨道绑定 `.sf2`，会改用 SoundFont 合成
- 输出当前固定为 `16-bit PCM WAV`
- 播放会尽量使用较小输出缓冲，但实际延迟仍取决于系统设备和驱动
- 当前实时播放依赖系统默认输出设备；如果设备不可用，`play` 会直接返回错误
- 当前 `pitch/speed` 使用基础离线算法，已经可用，但音质还不是专业级 timestretch/pitch-shift
