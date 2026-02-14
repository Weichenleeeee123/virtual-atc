# Virtual ATC 开发日志

## 2026-02-14

### 项目启动 🚀
- 创建 Tauri 桌面应用项目
- 技术栈：Rust + TypeScript + Whisper.cpp + SiliconFlow API
- 目标：为模拟飞行爱好者提供 AI 虚拟空管工具

### 已完成功能

#### 1. 前端界面 ✅
- 状态面板（连接状态、频率、语言切换）
- 飞行信息显示（呼号、高度、速度、航向）
- 通话记录面板（飞行员/ATC 消息）
- PTT 按钮（按住通话）
- 响应式设计 + 动画效果

#### 2. 后端架构 ✅
- 模块化设计：
  - `simulator.rs` - 模拟器连接
  - `whisper.rs` - 语音识别
  - `llm.rs` - ATC 对话生成
- Tauri 命令系统
- 状态管理（AppState）

#### 3. LLM 集成 ✅
- SiliconFlow API 集成
- 中国民航标准用语系统提示词
- ICAO 标准用语系统提示词
- 根据飞行数据生成上下文
- 支持中英文模式

#### 4. 文档 ✅
- 中国民航 ATC 标准用语文档
- ICAO 标准用语文档
- Whisper.cpp 集成指南
- X-Plane UDP 集成指南
- MSFS SimConnect 集成指南
- 项目 README 和 ROADMAP

### 技术亮点

1. **本地语音识别**
   - 使用 Whisper.cpp medium 模型
   - 支持 GPU 加速（CUDA/Metal）
   - 隐私保护，无需联网

2. **双模拟器支持**
   - X-Plane：通过 UDP 协议读取数据
   - MSFS：通过 SimConnect API
   - 自动检测模拟器类型

3. **标准陆空用语**
   - 中国民航标准
   - ICAO 国际标准
   - 专业、准确的 ATC 对话

4. **中英文双语**
   - 可切换语言模式
   - 适应不同地区用户

### 下一步计划

#### 短期（1周内）
1. 实现 Whisper.cpp 集成
   - 音频捕获（cpal）
   - 实时转录
   - GPU 加速配置

2. 实现 X-Plane UDP 连接
   - 接收飞行数据
   - 解析 DataRef
   - 实时更新 UI

3. 完善 ATC 对话逻辑
   - 飞行阶段检测
   - 上下文记忆
   - 标准用语模板

#### 中期（2周内）
4. 添加 MSFS 支持
   - SimConnect 集成
   - 或使用 Python 桥接

5. 功能增强
   - 频率管理
   - AI 主动通话
   - 音频播放（TTS）

6. 测试和优化
   - 性能优化
   - 错误处理
   - 用户体验改进

#### 长期
7. 高级功能
   - 多机场数据库
   - 天气信息集成
   - 飞行计划支持
   - 录音回放

### 技术挑战

1. **Whisper.cpp FFI**
   - Rust 绑定复杂度
   - 跨平台编译
   - GPU 驱动兼容性

2. **实时性能**
   - 语音识别延迟
   - UI 响应速度
   - 内存占用优化

3. **模拟器兼容性**
   - X-Plane 版本差异
   - MSFS SimConnect 配置
   - 数据格式变化

### 项目信息

- **GitHub**: https://github.com/Weichenleeeee123/virtual-atc
- **开发者**: 围城
- **开始日期**: 2026-02-14
- **预计完成**: 2026-02-28（基础版本）

### 开发心得

这是一个很有意思的项目，结合了：
- AI 技术（语音识别 + LLM）
- 飞行模拟（X-Plane + MSFS）
- 专业领域知识（ATC 标准用语）

对于模拟飞行爱好者来说，这个工具可以：
- 练习陆空通话
- 提升飞行真实感
- 学习标准用语
- 单人也能体验完整的 ATC 流程

技术上也很有挑战性：
- 本地语音识别（Whisper.cpp）
- 实时数据处理（UDP/SimConnect）
- 跨平台桌面应用（Tauri）
- AI 对话生成（LLM）

期待看到它逐步完善！✈️
