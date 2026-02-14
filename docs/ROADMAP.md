# Virtual ATC 开发计划

## 项目概述
基于 Tauri + Whisper + LLM 的智能虚拟空管工具

## 已完成 ✅

### 1. 项目初始化
- [x] 使用 create-tauri-app 创建项目
- [x] 配置 TypeScript + Vite
- [x] 添加必要的依赖（reqwest, tokio, serde）

### 2. 文档和标准用语
- [x] 中国民航 ATC 标准用语文档
- [x] ICAO 标准用语文档
- [x] 项目 README

### 3. 前端界面
- [x] 主界面布局（状态面板、飞行信息、通话记录）
- [x] PTT 按钮（按住通话）
- [x] 连接模拟器按钮
- [x] 语言切换（中英文）
- [x] 通话记录显示
- [x] 响应式设计和动画效果

### 4. 后端架构
- [x] Tauri 命令系统
- [x] 模块化结构（simulator, whisper, llm）
- [x] 状态管理（AppState）
- [x] LLM 客户端（SiliconFlow API 集成）
- [x] 模拟器连接模块（框架）
- [x] Whisper 引擎模块（框架）

## 待实现 🚧

### 5. Whisper.cpp 集成
- [ ] 下载并编译 whisper.cpp
- [ ] 下载 medium 模型
- [ ] 实现音频捕获（使用 cpal 或 rodio）
- [ ] 实现 Rust FFI 调用 whisper.cpp
- [ ] 添加 GPU 加速支持（CUDA/Metal）
- [ ] 实现实时转录

### 6. X-Plane 集成
- [ ] 研究 X-Plane Plugin SDK
- [ ] 实现 UDP 数据读取（DataRef）
- [ ] 读取飞机位置、高度、速度、航向
- [ ] 读取呼号和机型
- [ ] 实现数据更新循环

### 7. MSFS 集成（SimConnect）
- [ ] 添加 SimConnect SDK
- [ ] 实现 SimConnect 连接
- [ ] 读取飞行数据
- [ ] 支持 Windows 平台

### 8. ATC 逻辑增强
- [ ] 实现飞行阶段检测（起飞、巡航、降落）
- [ ] 根据飞行阶段提供合适的回复
- [ ] 实现频率管理系统
- [ ] 实现 AI 主动通话（如"XX航空XXX，联系进近XXX.X"）
- [ ] 添加标准用语模板库
- [ ] 实现上下文记忆（对话历史）

### 9. 功能完善
- [ ] 添加设置面板（API key、模型选择、音频设备）
- [ ] 实现音频播放（TTS 或预录音）
- [ ] 添加快捷键支持（空格键 PTT）
- [ ] 实现通话记录导出
- [ ] 添加错误处理和用户提示
- [ ] 性能优化

### 10. 测试和发布
- [ ] 单元测试
- [ ] 集成测试
- [ ] 在 X-Plane 中测试
- [ ] 在 MSFS 中测试
- [ ] 打包 Windows 安装程序
- [ ] 编写用户文档
- [ ] 推送到 GitHub
- [ ] 发布 v1.0

## 技术难点

1. **Whisper.cpp 集成**：需要处理 C/C++ FFI，可能需要使用 bindgen
2. **X-Plane SDK**：需要编写 C++ 插件或使用 UDP 协议
3. **SimConnect**：Windows 专用，需要处理 COM 接口
4. **实时音频处理**：低延迟音频捕获和转录
5. **GPU 加速**：CUDA/Metal 配置和优化

## 下一步行动

1. 先实现 Whisper.cpp 集成（最核心功能）
2. 然后实现 X-Plane UDP 数据读取（相对简单）
3. 完善 ATC 逻辑和对话系统
4. 最后添加 SimConnect 支持

## 预计时间

- Whisper 集成：2-3 天
- X-Plane 集成：1-2 天
- ATC 逻辑：2-3 天
- MSFS 集成：1-2 天
- 测试和优化：2-3 天

**��计：约 1-2 周**
