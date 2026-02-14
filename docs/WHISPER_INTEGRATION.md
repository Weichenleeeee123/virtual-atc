# Whisper.cpp 集成指南

## 概述
将 whisper.cpp 集成到 Tauri 应用中，实现本地语音识别。

## 方案选择

### 方案 1：使用 whisper-rs（推荐）
whisper-rs 是 whisper.cpp 的 Rust 绑定，最适合 Tauri 项目。

**优点：**
- 纯 Rust 集成，类型安全
- 自动处理 FFI
- 支持 GPU 加速
- 活跃维护

**依赖：**
```toml
[dependencies]
whisper-rs = "0.10"
cpal = "0.15"  # 音频捕获
hound = "3.5"  # WAV 文件处理
```

### 方案 2：直接使用 whisper.cpp
通过 bindgen 生成 Rust 绑定。

**缺点：**
- 需要手动编译 whisper.cpp
- FFI 复杂度高
- 跨平台构建困难

## 实现步骤

### 1. 下载 Whisper 模型
```bash
# 下载 medium 模型（推荐）
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin -o models/ggml-medium.bin

# 或者 small 模型（更快但精度稍低）
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin -o models/ggml-small.bin
```

### 2. 音频捕获
使用 `cpal` 库捕获麦克风音频：

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioRecorder {
    samples: Vec<f32>,
    stream: Option<cpal::Stream>,
}

impl AudioRecorder {
    pub fn start_recording(&mut self) -> Result<(), Box<dyn Error>> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        
        let samples = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = samples.clone();
        
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                samples_clone.lock().unwrap().extend_from_slice(data);
            },
            |err| eprintln!("Error: {}", err),
            None,
        )?;
        
        stream.play()?;
        self.stream = Some(stream);
        
        Ok(())
    }
    
    pub fn stop_recording(&mut self) -> Vec<f32> {
        self.stream = None;
        std::mem::take(&mut self.samples)
    }
}
```

### 3. Whisper 转录
```rust
use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};

pub struct WhisperEngine {
    ctx: WhisperContext,
}

impl WhisperEngine {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn Error>> {
        let ctx = WhisperContext::new(model_path)?;
        Ok(WhisperEngine { ctx })
    }
    
    pub fn transcribe(&self, audio: &[f32]) -> Result<String, Box<dyn Error>> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        // 设置语言（中文或英文）
        params.set_language(Some("zh"));
        params.set_translate(false);
        params.set_print_progress(false);
        params.set_print_special(false);
        
        // 转录
        let mut state = self.ctx.create_state()?;
        state.full(params, audio)?;
        
        // 获取结果
        let num_segments = state.full_n_segments()?;
        let mut result = String::new();
        
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            result.push_str(&segment);
        }
        
        Ok(result.trim().to_string())
    }
}
```

### 4. GPU 加速

**CUDA（NVIDIA）：**
```toml
[dependencies]
whisper-rs = { version = "0.10", features = ["cuda"] }
```

**Metal（Apple）：**
```toml
[dependencies]
whisper-rs = { version = "0.10", features = ["metal"] }
```

### 5. 音频预处理
Whisper 需要 16kHz 单声道音频：

```rust
fn resample_audio(audio: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return audio.to_vec();
    }
    
    let ratio = from_rate as f32 / to_rate as f32;
    let output_len = (audio.len() as f32 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    
    for i in 0..output_len {
        let pos = i as f32 * ratio;
        let idx = pos as usize;
        output.push(audio[idx]);
    }
    
    output
}
```

## 性能优化

1. **使用 medium 模型**：平衡精度和速度
2. **启用 GPU 加速**：CUDA 或 Metal
3. **音频缓冲**：避免频繁的小块转录
4. **异步处理**：不阻塞 UI 线程

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_whisper_transcribe() {
        let engine = WhisperEngine::new("models/ggml-medium.bin").unwrap();
        
        // 加载测试音频
        let audio = load_test_audio("test.wav");
        
        let result = engine.transcribe(&audio).unwrap();
        assert!(!result.is_empty());
    }
}
```

## 注意事项

1. **模型文件大小**：medium 模型约 1.5GB
2. **首次加载慢**：需要加载模型到内存
3. **内存占用**：约 2-3GB（medium 模型）
4. **实时性**：medium 模型在 GPU 上约 1-2 秒延迟

## 替代方案

如果 whisper.cpp 集成困难，可以考虑：
1. **Vosk**：轻量级离线语音识别
2. **Coqui STT**：开源 STT 引擎
3. **云端 API**：Azure Speech、Google Speech（需要网络）
