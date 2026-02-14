# Virtual ATC 用户手册

## 快速开始

### 1. 系统要求
- **操作系统**: Windows 10/11
- **内存**: 8GB+ RAM
- **显卡**: 支持 CUDA 的 NVIDIA 显卡（推荐，用于 GPU 加速）
- **模拟器**: X-Plane 11/12 或 Microsoft Flight Simulator 2020/2024
- **麦克风**: 任何标准麦克风

### 2. 安装

#### 下载
从 [GitHub Releases](https://github.com/Weichenleeeee123/virtual-atc/releases) 下载最新版本。

#### 安装步骤
1. 运行 `virtual-atc-setup.exe`
2. 按照安装向导完成安装
3. 首次运行会自动下载 Whisper 模型（约 1.5GB）

### 3. 配置模拟器

#### X-Plane 配置
参见 [X-Plane 配置指南](XPLANE_SETUP.md)

**快速步骤：**
1. 打开 X-Plane
2. `Settings` → `Data Output`
3. 勾选数据组 3, 17, 20 的 UDP 列
4. 设置 IP: `127.0.0.1`, Port: `49000`

#### MSFS 配置
参见 [MSFS 配置指南](MSFS_SETUP.md)（即将推出）

### 4. 使用 Virtual ATC

#### 连接模拟器
1. 启动模拟器并开始飞行
2. 启动 Virtual ATC
3. 点击"连接模拟器"按钮
4. 等待连接状态变为"已连接"（绿色）

#### 进行通话
1. **按住** PTT 按钮（或按住空格键）
2. 对着麦克风说话（使用标准陆空用语）
3. **松开** 按钮停止录音
4. 等待 AI 空管回复

#### 切换语言
在界面右上角选择：
- **中文**: 使用中国民航标准用语
- **English**: 使用 ICAO 标准用语

## 标准用语示例

### 中文模式

#### 起飞阶段
```
飞行员: "北京塔台，国航123，停机位A12，请求开车"
ATC:    "国航123，北京塔台，可以开车"

飞行员: "北京塔台，国航123，请求推出"
ATC:    "国航123，可以推出，面向北"

飞行员: "北京塔台，国航123，请求滑行"
ATC:    "国航123，可以滑行至跑道01，地面风360度5米"

飞行员: "北京塔台，国航123，跑道01，请求起飞"
ATC:    "国航123，跑道01，可以起飞，地面风360度5米"
```

#### 巡航阶段
```
飞行员: "北京进近，国航123，高度8000米，请求进近"
ATC:    "国航123，北京进近，雷达识别，保持现高度"

飞行员: "北京进近，国航123，请求下降"
ATC:    "国航123，可以下降至6000米"
```

#### 降落阶段
```
飞行员: "北京塔台，国航123，五边，请求落地"
ATC:    "国航123，跑道01，可以落地，地面风360度5米"

飞行员: "北京塔台，国航123，跑道外"
ATC:    "国航123，脱离跑道后联系地面121.6"
```

### English Mode

#### Departure
```
Pilot: "Beijing Tower, Air China 123, stand A12, request startup"
ATC:   "Air China 123, Beijing Tower, startup approved"

Pilot: "Beijing Tower, Air China 123, request pushback"
ATC:   "Air China 123, pushback approved, face north"

Pilot: "Beijing Tower, Air China 123, request taxi"
ATC:   "Air China 123, taxi to runway 01, wind 360 at 5"

Pilot: "Beijing Tower, Air China 123, runway 01, ready for departure"
ATC:   "Air China 123, runway 01, cleared for takeoff, wind 360 at 5"
```

## 界面说明

### 状态面板
- **连接状态**: 显示与模拟器的连接状态
  - 红色"未连接": 未连接到模拟器
  - 绿色"已连接": 已成功连接
- **当前频率**: 显示当前 ATC 频率（未来版本支持切换）
- **语言模式**: 切换中文/英文模式

### 飞行信息
实时显示当前飞行数据：
- **呼号**: 飞机呼号（如 CCA123）
- **高度**: 海拔高度（英尺）
- **速度**: 指示空速（节��
- **航向**: 真航向（度）

### 通话记录
显示所有陆空对话历史：
- 蓝色气泡: 飞行员（你）
- 绿色气泡: ATC（AI）
- 每条消息显示时间戳

### 控制按钮
- **PTT 按钮**: 按住说话，松开停止（也可以用空格键）
- **连接模拟器**: 连接/断开模拟器

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| 空格键（按住） | PTT（按住通话） |
| Ctrl+L | 切换语言 |
| Ctrl+C | 清空通话记录 |
| Ctrl+Q | 退出程序 |

## 常见问题

### Q: 语音识别不准确怎么办？
A: 
1. 确保麦克风音量适中（不要太小或太大）
2. 在安静的环境中使用
3. 说话清晰，使用标准用语
4. 检查是否选择了正确的语言模式

### Q: 连接不上模拟器？
A: 
1. 确认模拟器正在运行
2. 检查 X-Plane Data Output 设置
3. 确认防火墙允许 UDP 49000 端口
4. 重启模拟器和 Virtual ATC

### Q: AI 回复不合理？
A: 
1. 确保使用标准陆空用语
2. 提供完整的信息（呼号、请求内容）
3. 根据飞行阶段使用合适的用语
4. 如果问题持续，请在 GitHub 提交 Issue

### Q: 程序运行卡顿？
A: 
1. 关闭其他占用 GPU 的程序
2. 降低模拟器画质设置
3. 确保有足够的内存（8GB+）
4. 检查 CPU 和 GPU 温度

### Q: 如何更新？
A: 
1. 下载最新版本安装包
2. 运行安装程序（会自动覆盖旧版本）
3. 设置和数据会自动保留

## 高级功能（即将推出）

- 🔄 频率切换
- 🎵 TTS 语音播放
- 📝 飞行计划支持
- 🌤️ 天气信息集成
- 📊 飞行统计
- 🎙️ 录音回放

## 反馈和支持

- **GitHub Issues**: https://github.com/Weichenleeeee123/virtual-atc/issues
- **讨论区**: https://github.com/Weichenleeeee123/virtual-atc/discussions
- **邮箱**: weicheng@kinolens.ai

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件

## 致谢

- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) - 语音识别引擎
- [Tauri](https://tauri.app/) - 桌面应用框架
- [SiliconFlow](https://siliconflow.cn/) - LLM API 服务
- X-Plane 和 MSFS 社区

---

祝飞行愉快！✈️🎙️

如有问题，欢迎在 GitHub 提交 Issue 或参与讨论。
