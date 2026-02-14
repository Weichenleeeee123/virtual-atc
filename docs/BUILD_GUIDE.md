# Virtual ATC - 编译和运行指南

## 系统要求

### 最低配置
- **操作系统**：Windows 10/11, macOS 10.15+, Linux (Ubuntu 20.04+)
- **CPU**：Intel i5 或同等性能
- **内存**：8 GB RAM
- **存储**：5 GB 可用空间（包括模型文件）
- **麦克风**：任何标准麦克风

### 推荐配置
- **CPU**：Intel i7 或 AMD Ryzen 7
- **内存**：16 GB RAM
- **GPU**：NVIDIA GTX 1060 或更高（用于 GPU 加速）
- **存储**：10 GB SSD
- **麦克风**：降噪麦克风或耳机麦克风

## 前置依赖

### 1. Rust 工具链

**Windows:**
```bash
# 下载并运行 rustup-init.exe
https://rustup.rs/

# 或使用 winget
winget install Rustlang.Rustup
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

验证安装：
```bash
rustc --version
cargo --version
```

### 2. Node.js

**Windows:**
```bash
# 下载安装包
https://nodejs.org/

# 或使用 winget
winget install OpenJS.NodeJS
```

**macOS:**
```bash
brew install node
```

**Linux:**
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

验证安装：
```bash
node --version
npm --version
```

### 3. 系统依赖

**Windows:**
- Visual Studio 2019 或更高版本（需要 C++ 工具）
- 或者安装 [Build Tools for Visual Studio](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libasound2-dev
```

## 克隆项目

```bash
git clone https://github.com/Weichenleeeee123/virtual-atc.git
cd virtual-atc
```

## 安装依赖

### 1. 安装 Node.js 依赖

```bash
npm install
```

### 2. 下载 Whisper 模型

```bash
# 创建模型目录
mkdir -p models

# 下载 medium 模型（推荐）
curl -L -o models/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

**注意**：模型文件约 1.5 GB，下载可能需要几分钟。

### 3. 配置环境变量

创建 `.env` 文件：

```bash
# .env
SILICONFLOW_API_KEY=your_api_key_here
WHISPER_MODEL_PATH=./models/ggml-medium.bin
```

**获取 SiliconFlow API Key：**
1. 访问 [SiliconFlow](https://siliconflow.cn/)
2. 注册并登录
3. 进入控制台 → API Keys
4. 创建新的 API Key
5. 复制并粘贴到 `.env` 文件

## 开发模式运行

```bash
npm run tauri dev
```

首次运行会编译 Rust 代码，可能需要 5-10 分钟。

### 常见编译错误

**错误 1：找不到 Rust 编译器**
```
error: could not find `rustc`
```

**解决方案**：
```bash
# 重新安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**错误 2：缺少系统依赖（Linux）**
```
error: failed to run custom build command for `webkit2gtk-sys`
```

**解决方案**：
```bash
sudo apt install libwebkit2gtk-4.0-dev
```

**错误 3：Whisper 模型未找到**
```
Model file not found: ./models/ggml-medium.bin
```

**解决方案**：
```bash
# 检查模型文件
ls -lh models/ggml-medium.bin

# 如果不存在，重新下载
curl -L -o models/ggml-medium.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

## 生产构建

### 1. 构建应用

```bash
npm run tauri build
```

构建完成后，安装包位于：

- **Windows**: `src-tauri/target/release/bundle/msi/`
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/deb/` 或 `appimage/`

### 2. 优化构建大小

编辑 `src-tauri/Cargo.toml`：

```toml
[profile.release]
opt-level = "z"     # 优化大小
lto = true          # 链接时优化
codegen-units = 1   # 更好的优化
strip = true        # 移除调试符号
```

重新构建：
```bash
npm run tauri build
```

## 配置 X-Plane

### 1. 启动 X-Plane

确保 X-Plane 11 或 X-Plane 12 正在运行。

### 2. 无需手动配置

Virtual ATC 使用 RREF 机制自动订阅数据，无需在 X-Plane 中手动配置 Data Output。

### 3. 验证连接

1. 启动 Virtual ATC
2. 点击"连接模拟器"按钮
3. 如果连接成功，会显示飞行数据

详细配置请参考 [XPLANE_CONNECTION.md](./XPLANE_CONNECTION.md)。

## 使用指南

### 1. 启动应用

双击安装后的应用图标，或在开发模式下运行：

```bash
npm run tauri dev
```

### 2. 连接模拟器

1. 确保 X-Plane 正在运行
2. 点击"连接模拟器"按钮
3. 等待连接成功（状态变为绿色）

### 3. 开始对话

1. 按住"PTT"按钮（或按住空格键）
2. 说出你的请求，例如：
   - 中文："国航123，请求起飞"
   - 英文："Air China 123, request takeoff"
3. 松开按钮
4. 等待 AI 空管回复（自动播放语音）

### 4. 切换语言

点击右上角的语言切换按钮：
- 🇨🇳 中文模式
- 🇺🇸 英文模式

## 性能优化

### 1. 使用更小的模型

如果转录速度慢，可以使用 `small` 模型：

```bash
# 下载 small 模型
curl -L -o models/ggml-small.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin

# 更新 .env
WHISPER_MODEL_PATH=./models/ggml-small.bin
```

### 2. 启用 GPU 加速（NVIDIA GPU）

**Windows:**

1. 安装 [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads)
2. 编辑 `src-tauri/Cargo.toml`：

```toml
[dependencies]
whisper-rs = { version = "0.12", features = ["cuda"] }
```

3. 重新编译：

```bash
npm run tauri build
```

**Linux:**

```bash
# 安装 CUDA
sudo apt install nvidia-cuda-toolkit

# 重新编译
npm run tauri build
```

### 3. 调整音频缓冲

编辑 `src-tauri/src/modules/whisper.rs`，调整采样率：

```rust
self.sample_rate = 16000;  // 降低采样率可以提高速度
```

## 故障排查

### 应用无法启动

**问题**：双击应用后没有反应

**解决方案**：
1. 检查是否有错误日志（Windows: `%APPDATA%/virtual-atc/logs/`）
2. 确认所有依赖已安装
3. 尝试在终端中运行查看错误信息

### 无法连接 X-Plane

**问题**：点击"连接模拟器"后显示错误

**解决方案**：
1. 确认 X-Plane 正在运行
2. 检查防火墙是否阻止 UDP 49000 端口
3. 确认 X-Plane 和 Virtual ATC 在同一台电脑上
4. 查看控制台日志

### 语音识别不准确

**问题**：转录的文字错误很多

**解决方案**：
1. 使用更大的模型（medium 或 large）
2. 确保麦克风音质良好
3. 在安静的环境中录音
4. 说话清晰、语速适中
5. 检查语言设置是否正确

### TTS 无声音

**问题**：AI 回复没有语音

**解决方案**：
1. 检查系统音量
2. 确认 SiliconFlow API Key 正确
3. 查看控制台是否有 TTS 错误
4. 检查网络连接

### 内存占用过高

**问题**：应用占用大量内存

**解决方案**：
1. 使用更小的 Whisper 模型
2. 关闭其他占用内存的程序
3. 增加系统虚拟内存

## 开发调试

### 查看日志

**开发模式：**
```bash
npm run tauri dev
```

日志会直接输出到终端。

**生产模式：**

- **Windows**: `%APPDATA%/virtual-atc/logs/`
- **macOS**: `~/Library/Application Support/virtual-atc/logs/`
- **Linux**: `~/.local/share/virtual-atc/logs/`

### 调试 Rust 代码

在 `src-tauri/src/` 中添加 `println!` 或 `eprintln!`：

```rust
println!("Debug: flight data = {:?}", flight_data);
```

### 调试前端代码

打开浏览器开发者工具：

- **Windows/Linux**: `Ctrl + Shift + I`
- **macOS**: `Cmd + Option + I`

## 更新应用

### 拉取最新代码

```bash
git pull origin master
```

### 重新安装依赖

```bash
npm install
```

### 重新构建

```bash
npm run tauri build
```

## 卸载

### Windows

1. 打开"设置" → "应用"
2. 找到"Virtual ATC"
3. 点击"卸载"

### macOS

1. 打开"应用程序"文件夹
2. 将"Virtual ATC"拖到废纸篓

### Linux

```bash
# Debian/Ubuntu
sudo apt remove virtual-atc

# 或手动删除
rm -rf ~/.local/share/virtual-atc
```

## 贡献指南

欢迎贡献代码！请参考 [CONTRIBUTING.md](./CONTRIBUTING.md)。

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件。

## 支持

- **GitHub Issues**: https://github.com/Weichenleeeee123/virtual-atc/issues
- **文档**: https://github.com/Weichenleeeee123/virtual-atc/tree/master/docs
- **讨论**: https://github.com/Weichenleeeee123/virtual-atc/discussions

## 相关文档

- [X-Plane 连接配置](./XPLANE_CONNECTION.md)
- [Whisper 模型设置](./WHISPER_SETUP.md)
- [用户手册](./USER_GUIDE.md)
- [开发路线图](./ROADMAP.md)
