# Whisper 模型下载和配置

## 1. 下载 Whisper 模型

Virtual ATC 使用 `ggml-medium.bin` 模型（中等大小，平衡准确率和性能）。

### 方法 1：官方下载（推荐）

```bash
# 创建模型目录
mkdir -p models

# 下载 medium 模型（约 1.5GB）
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin -o models/ggml-medium.bin
```

### 方法 2：从 Hugging Face 镜像下载

```bash
# 使用国内镜像加速
curl -L https://hf-mirror.com/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin -o models/ggml-medium.bin
```

### 方法 3：手动下载

访问：https://huggingface.co/ggerganov/whisper.cpp/tree/main

下载 `ggml-medium.bin` 并放到 `models/` 目录。

## 2. 其他可用模型

| 模型 | 大小 | 内存占用 | 准确率 | 速度 |
|------|------|----------|--------|------|
| ggml-tiny.bin | 75 MB | ~390 MB | 低 | 最快 |
| ggml-base.bin | 142 MB | ~500 MB | 中低 | 快 |
| ggml-small.bin | 466 MB | ~1.0 GB | 中 | 中等 |
| **ggml-medium.bin** | 1.5 GB | ~2.6 GB | **高** | **推荐** |
| ggml-large-v3.bin | 2.9 GB | ~4.7 GB | 最高 | 慢 |

**推荐使用 medium 模型**：
- ✅ 中文识别准确率高
- ✅ 航空术语识别效果好
- ✅ 性能和准确率平衡
- ✅ 支持 GPU 加速

## 3. 配置环境变量

在 `.env` 文件中添加：

```bash
WHISPER_MODEL_PATH=./models/ggml-medium.bin
```

如果使用其他模型：

```bash
# 使用 small 模型（更快但准确率稍低）
WHISPER_MODEL_PATH=./models/ggml-small.bin

# 使用 large 模型（最高准确率但速度慢）
WHISPER_MODEL_PATH=./models/ggml-large-v3.bin
```

## 4. GPU 加速（可选）

### NVIDIA GPU (CUDA)

如果你有 NVIDIA 显卡，可以启用 CUDA 加速：

1. 安装 CUDA Toolkit（11.8 或更高版本）
2. 在 `Cargo.toml` 中启用 CUDA 特性：

```toml
whisper-rs = { version = "0.12", features = ["cuda"] }
```

3. 重新编译项目

### Apple Silicon (Metal)

在 macOS 上自动启用 Metal 加速，无需额外配置。

## 5. 验证安装

运行应用后，检查日志：

```
✅ Whisper model loaded: ./models/ggml-medium.bin
```

如果看到错误：

```
❌ Model file not found: ./models/ggml-medium.bin
```

请检查：
1. 模型文件是否存在
2. 路径是否正确
3. `.env` 文件是否配置正确

## 6. 性能优化建议

### 内存不足（< 4GB）
使用 `ggml-small.bin` 或 `ggml-base.bin`

### 追求速度
使用 `ggml-base.bin` + GPU 加速

### 追求准确率
使用 `ggml-large-v3.bin` + GPU 加速

### 平衡选择（推荐）
使用 `ggml-medium.bin`（默认配置）

## 7. 常见问题

**Q: 下载速度慢怎么办？**
A: 使用国内镜像 `hf-mirror.com` 或使用下载工具（如 aria2）。

**Q: 模型可以放在其他位置吗？**
A: 可以，修改 `.env` 中的 `WHISPER_MODEL_PATH` 为绝对路径。

**Q: 支持其他语言吗？**
A: 支持！Whisper 支持 99 种语言，修改 `whisper.rs` 中的 `set_language()` 参数。

**Q: 识别准确率不高怎么办？**
A: 
1. 升级到 medium 或 large 模型
2. 确保麦克风质量良好
3. 减少环境噪音
4. 说话清晰、语速适中
