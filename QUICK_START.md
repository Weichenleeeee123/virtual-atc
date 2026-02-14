# Virtual ATC - 快速开始指南

## 📋 系统要求

- **操作系统**：Windows 10/11, macOS 10.15+, 或 Linux
- **内存**：至少 8GB RAM（推荐 16GB）
- **存储**：至少 5GB 可用空间（包含 Whisper 模型）
- **显卡**：支持 CUDA 的 NVIDIA 显卡（可选，用于 GPU 加速）
- **模拟器**：X-Plane 11/12 或 Microsoft Flight Simulator 2020

## 🔧 前置依赖

### 1. 安装 Rust

**Windows:**
```bash
# 下载并运行 rustup-init.exe
https://rustup.rs/

# 安装后重启终端，验证安装
rustc --version
cargo --version
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version
```

### 2. 安装 Node.js

**下载并安装 Node.js 18+ LTS:**
https://nodejs.org/

验证安装：
```bash
node --version  # 应该显示 v18.x.x 或更高
npm --version
```

### 3. 安装 Tauri CLI

```bash
cargo install tauri-cli
```

### 4. 安装系统依赖

**Windows:**
- 安装 Visual Studio 2022 Build Tools
- 安装 WebView2（Windows 11 已内置）

**macOS:**
```bash
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

## 📦 安装 Virtual ATC

### 1. 克隆仓库

```bash
git clone https://github.com/Weichenleeeee123/virtual-atc.git
cd virtual-atc
```

### 2. 安装前端依赖

```bash
npm install
```

### 3. 配置环境变量

创建 `.env` 文件：
```bash
cp .env.example .env
```

编辑 `.env` 文件，填入你的 API key：
```env
SILICONFLOW_API_KEY=your_api_key_here
WHISPER_MODEL_PATH=./models/ggml-medium.bin
```

### 4. 下载 Whisper 模型

**方法 1：使用内置模型管理器（推荐）**
- 启动应用后，点击"模型管理"标签
- 选择 `medium` 模型（推荐）
- 点击"下载"按钮

**方法 2：手动下载**
```bash
# 创建模型目录
mkdir -p models

# 下载 medium 模型（1.5GB）
cd models
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin

# 或使用 curl
curl -L -o ggml-medium.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
```

**模型选择建议：**
- `tiny` (75MB) - 最快，准确率较低
- `base` (142MB) - 快速，准确率一般
- `small` (466MB) - 平衡，推荐测试用
- **`medium` (1.5GB) - 推荐，准确率高**
- `large-v3` (3.1GB) - 最准确，但速度慢

## 🚀 运行应用

### 开发模式

```bash
# 启动开发服务器（支持热重载）
npm run tauri dev
```

### 生产构建

```bash
# 构建生产版本
npm run tauri build

# 构建产物位置：
# Windows: src-tauri/target/release/virtual-atc.exe
# macOS: src-tauri/target/release/bundle/macos/Virtual ATC.app
# Linux: src-tauri/target/release/virtual-atc
```

## 🎮 配置模拟器

### X-Plane 配置

**无需手动配置！** Virtual ATC 会自动通过 RREF 协议订阅飞行数据。

只需确保：
1. X-Plane 正在运行
2. 飞机已加载
3. 点击 Virtual ATC 的"连接模拟器"按钮

### MSFS 配置

1. **安装 Python 依赖：**
```bash
pip install SimConnect-Python
```

2. **启动 MSFS**

3. **在 Virtual ATC 中点击"连接 MSFS"**

应用会自动启动 Python 桥接脚本。

## 🎯 使用流程

### 1. 启动应用

```bash
npm run tauri dev
```

### 2. 下载模型（首次使用）

- 点击"模型管理"标签
- 下载 `medium` 模型
- 等待下载完成（约 1.5GB）

### 3. 连接模拟器

- 启动 X-Plane 或 MSFS
- 加载飞机
- 在 Virtual ATC 中点击"连接模拟器"
- 等待连接成功提示

### 4. 开始对话

1. **按住 PTT 按钮**（或按住空格键）
2. **说话**："北京塔台，国航123，请求起飞"
3. **松开按钮**
4. **等待 AI 回复**（约 3-5 秒）
5. **听到语音回复**

### 5. 查看飞行信息

右侧面板实时显示：
- 当前飞行阶段
- 高度、速度、航向
- 垂直速度
- 经纬度坐标

## 🔍 故障排查

### 问题 1：Whisper 模型加载失败

**症状：** 提示"Model file not found"

**解决：**
```bash
# 检查模型文件是否存在
ls -lh models/ggml-medium.bin

# 检查 .env 配置
cat .env | grep WHISPER_MODEL_PATH

# 确保路径正确
WHISPER_MODEL_PATH=./models/ggml-medium.bin
```

### 问题 2：无法连接 X-Plane

**症状：** 点击"连接模拟器"无响应

**解决：**
1. 确保 X-Plane 正在运行
2. 确保飞机已加载（不能在主菜单）
3. 检查防火墙是否阻止 UDP 端口 49000
4. 查看控制台日志

### 问题 3：语音识别不准确

**症状：** 转录结果错误

**解决：**
1. 使用更大的模型（medium 或 large-v3）
2. 确保麦克风音质良好
3. 在安静环境中使用
4. 说话清晰、语速适中

### 问题 4：LLM 回复延迟

**症状：** 等待时间超过 10 秒

**解决：**
1. 检查网络连接
2. 验证 API key 是否有效
3. 查看 SiliconFlow 服务状态
4. 考虑使用本地 LLM（需要修改代码）

### 问题 5：Little Navmap 数据库未找到

**症状：** 提示"未找到 Little Navmap 数据库"

**解决：**
1. 安装 Little Navmap：https://albar965.github.io/littlenavmap.html
2. 启动 Little Navmap 并加载场景库
3. 确保数据库文件存在：
   - Windows: `%APPDATA%\ABarthel\little_navmap_db\`
   - macOS: `~/Library/Application Support/ABarthel/little_navmap_db/`
   - Linux: `~/.config/ABarthel/little_navmap_db/`

## 📚 更多文档

- [用户手册](docs/USER_GUIDE.md) - 详细使用说明
- [X-Plane 配置](docs/XPLANE_SETUP.md) - X-Plane 详细配置
- [MSFS 集成](docs/MSFS_INTEGRATION.md) - MSFS 配置指南
- [Whisper 设置](docs/WHISPER_SETUP.md) - 语音识别配置
- [开发路线图](docs/ROADMAP.md) - 未来计划

## 🆘 获取帮助

- **GitHub Issues**: https://github.com/Weichenleeeee123/virtual-atc/issues
- **讨论区**: https://github.com/Weichenleeeee123/virtual-atc/discussions

## 🎉 开始使用

现在你已经准备好了！启动模拟器，连接 Virtual ATC，开始你的虚拟 ATC 训练之旅！

祝飞行愉快！✈️
