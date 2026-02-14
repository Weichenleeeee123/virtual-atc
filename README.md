# Virtual ATC - AI 虚拟空管系统

基于 Tauri + Whisper + LLM 的智能虚拟空管工具，支持 X-Plane 和 MSFS。

## 功能特性

- 🎙️ **本地语音识别**：使用 Whisper.cpp (medium 模型)
- 🤖 **AI 空管对话**：基于 SiliconFlow API
- 🔊 **TTS 语音播放**：ATC 回复自动播放语音
- ✈️ **飞行模拟器集成**：支持 X-Plane 和 MSFS (SimConnect)
- 📻 **频率切换**：模拟真实 ATC 频率管理
- 🌏 **中英文模式**：可选中文或英文 ATC 用语
- 🔄 **双向通话**：AI 主动发起通话 + 响应飞行员

## 技术栈

- **前端**：TypeScript + Vite
- **后端**：Rust (Tauri)
- **语音识别**：whisper.cpp
- **LLM**：SiliconFlow API
- **飞行模拟器**：X-Plane SDK + SimConnect

## 系统要求

- Windows 10/11 (推荐有独显用于 GPU 加速)
- Node.js 18+
- Rust 1.70+
- 8GB+ RAM

## 开发计划

- [ ] 基础 UI 界面
- [ ] Whisper.cpp 集成
- [ ] SiliconFlow API 集成
- [ ] 中国民航 ATC 标准用语库
- [ ] X-Plane 插件开发
- [ ] SimConnect 集成
- [ ] 频率管理系统
- [ ] 双向通话逻辑

## 安装

```bash
npm install
npm run tauri dev
```

## 配置

创建 `.env` 文件：

```
SILICONFLOW_API_KEY=your_api_key_here
```
