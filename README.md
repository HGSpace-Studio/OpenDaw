# OpenDaw

一个用 Rust 编写的 DAW 原型，目前包含两个部分：

- 基于 `vizia` 的深色 GUI 壳
- 基于成熟库 `hound` 读取和写出 WAV
- 基于 `rayon` 并行读取轨道与并行分块混音
- 读取多个 `16-bit PCM WAV` 或 `32-bit float WAV`
- 每个输入轨道可单独设置音量
- 自动按最长轨道补静音混音
- 支持单声道和双声道轨道混合
- 输出新的 `16-bit PCM WAV`

## 运行 GUI

```bash
cargo run --release --
```

## 命令行混音

```text
mix <输出文件.wav> <输入1.wav:音量> <输入2.wav:音量> ...
```

例如：

```bash
cargo run --release -- mix out/song_mix.wav kick.wav:0.9 pad.wav:0.45 lead.wav:0.7
```

更快的运行方式：

```bash
cargo build --release
./target/release/opendaw
./target/release/opendaw mix out/song_mix.wav kick.wav:0.9 pad.wav:0.45 lead.wav:0.7
```

## 当前限制

- GUI 当前是深色静态工作台外观，先把视觉框架搭起来，还没接入真实时间线交互
- 所有输入必须具有相同的采样率
- 当前重点支持 `mono/stereo` 混音：
  - `mono -> stereo` 会自动复制到左右声道
  - `stereo -> mono` 会自动平均下混
- 输入当前支持 `16-bit PCM WAV` 和 `32-bit float WAV`
- 输出当前固定为 `16-bit PCM WAV`
- 暂不包含 GUI、时间线、效果器、实时播放
