# TTS（语音合成）集成指南

## 概述

Virtual ATC 使用 SiliconFlow 的 TTS API（基于 Fish Speech 1.4）实现 ATC 语音播放。

## 功能特性

- ✅ 中文语音（zh-CN-XiaoxiaoNeural - 温柔、专业的女声）
- ✅ 英文语音（en-US-JennyNeural - 清晰、专业的女声）
- ✅ 自动播放 ATC 回复
- ✅ 支持 MP3 格式
- ✅ 可调节语速（默认 1.0）

## 工作流程

1. 用户按住 PTT 说话
2. Whisper 转录语音为文字
3. LLM 生成 ATC 回复
4. **TTS 将回复转换为语音并播放**
5. 前端显示文字记录

## 技术实现

### 依赖库

- `rodio` - Rust 音频播放库
- `reqwest` - HTTP 客户端（调用 TTS API）

### API 端点

```
POST https://api.siliconflow.cn/v1/audio/speech
```

### 请求格式

```json
{
  "model": "fishaudio/fish-speech-1.4",
  "input": "国航123，北京塔台，可以起飞",
  "voice": "zh-CN-XiaoxiaoNeural",
  "response_format": "mp3",
  "speed": 1.0
}
```

### 可用语音

#### 中文
- `zh-CN-XiaoxiaoNeural` - 女声（推荐，温柔专业）
- `zh-CN-YunxiNeural` - 男声（沉稳）
- `zh-CN-YunyangNeural` - 男声（年轻）

#### 英文
- `en-US-JennyNeural` - 女声（推荐，清晰专业）
- `en-US-GuyNeural` - 男声（沉稳）
- `en-US-AriaNeural` - 女声（活泼）

## 使用方法

### 自动播放（默认）

当 LLM 生成 ATC 回复后，TTS 会自动播放语音：

```rust
let response = llm.get_atc_response(&message, &language, flight_data).await?;

// 自动播放 TTS
let tts = state.tts.lock().unwrap();
tts.speak(&response, &language).await?;
```

### 手动调用

```rust
use modules::tts::TTSEngine;

let tts = TTSEngine::new();

// 中文
tts.speak("国航123，北京塔台，可以起飞", "zh").await?;

// 英文
tts.speak("Air China 123, Beijing Tower, cleared for takeoff", "en").await?;
```

### 保存到文件（调试用）

```rust
tts.save_to_file(
    "国航123，北京塔台，可以起飞",
    "zh",
    "./output.mp3"
).await?;
```

## 配置选项

### 调整语速

修改 `tts.rs` 中的 `speed` 参数：

```rust
"speed": 1.2  // 1.2 倍速（更快）
"speed": 0.8  // 0.8 倍速（更慢）
```

### 更换语音

修改 `speak()` 函数中的 `voice` 变量：

```rust
let voice = if language == "zh" {
    "zh-CN-YunxiNeural"  // 改为男声
} else {
    "en-US-GuyNeural"    // 改为男声
};
```

## 性能优化

### 延迟优化

- TTS API 响应时间：约 500-1000ms
- 音频播放延迟：< 100ms
- 总延迟：约 1 秒

### 缓存策略（未来）

可以缓存常用短语：

```rust
// 缓存常用回复
let cache = HashMap::new();
cache.insert("收到", "cached_audio_1.mp3");
cache.insert("明白", "cached_audio_2.mp3");
```

## 故障排查

### TTS 无声音

1. 检查 API key 是否配置正确
2. 检查网络连接
3. 查看控制台错误日志

### 音频播放失败

1. 检查系统音频设备
2. 确认 `rodio` 依赖已安装
3. 尝试保存到文件测试

### API 调用失败

```bash
# 测试 API 连接
curl -X POST https://api.siliconflow.cn/v1/audio/speech \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "fishaudio/fish-speech-1.4",
    "input": "测试",
    "voice": "zh-CN-XiaoxiaoNeural"
  }' \
  --output test.mp3
```

## 未来改进

1. **离线 TTS** - 集成 Piper TTS（无需网络）
2. **语音缓存** - 缓存常用短语
3. **多语音选择** - 用户可选择不同的 ATC 声音
4. **情感控制** - 紧急情况使用更严肃的语气
5. **背景音效** - 添加无线电杂音效果

## 示例对话

**飞行员**："北京塔台，国航123，请求起飞"

**ATC（文字）**："国航123，跑道01，可以起飞，地面风270度5米"

**ATC（语音）**：🔊 *播放专业女声*

---

**Pilot**: "Beijing Tower, Air China 123, ready for departure"

**ATC (Text)**: "Air China 123, runway 01, cleared for takeoff, wind 270 at 5"

**ATC (Voice)**: 🔊 *Professional female voice*

## 参考资料

- [SiliconFlow TTS API 文档](https://docs.siliconflow.cn/api-reference/audio/speech)
- [Fish Speech 项目](https://github.com/fishaudio/fish-speech)
- [Rodio 音频库](https://github.com/RustAudio/rodio)
