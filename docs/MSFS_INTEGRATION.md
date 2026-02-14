# MSFS SimConnect 集成指南

## 概述

Virtual ATC 支持微软飞行模拟器（MSFS）通过 SimConnect API 连接。由于 SimConnect 是 Windows 专用的 COM 接口，我们使用 Python 桥接方案来实现跨平台兼容。

## 架构设计

```
MSFS (SimConnect) 
    ↓
Python Bridge (msfs_bridge.py)
    ↓ UDP (JSON)
Virtual ATC (Rust)
```

## 安装步骤

### 1. 安装 Python 依赖

```bash
# 进入项目目录
cd virtual-atc

# 安装 Python 依赖
pip install -r scripts/requirements.txt
```

或者手动安装：

```bash
pip install Python-SimConnect==0.4.26
```

### 2. 验证 SimConnect 安装

MSFS 自带 SimConnect，无需额外安装。但需要确认：

**Windows 10/11:**
- SimConnect 库位于：`C:\MSFS SDK\SimConnect SDK\lib`
- 如果没有，需要安装 [MSFS SDK](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/SimConnect_SDK.htm)

### 3. 测试 Python 桥接

```bash
# 启动 MSFS
# 然后运行测试脚本
python scripts/msfs_bridge.py
```

如果看到以下输出，说明连接成功：

```
Starting MSFS SimConnect bridge...
Connected to MSFS
UDP socket created, sending to ('127.0.0.1', 49001)
```

## 使用方法

### 1. 启动 MSFS

确保 MSFS 2020 或 MSFS 2024 正在运行。

### 2. 启动 Virtual ATC

```bash
npm run tauri dev
```

### 3. 连接 MSFS

在 Virtual ATC 界面中：
1. 选择"MSFS"作为模拟器类型
2. 点击"连接模拟器"按钮
3. 等待连接成功（状态变为绿色）

### 4. 开始对话

与 X-Plane 使用方法相同：
1. 按住 PTT 按钮
2. 说出你的请求
3. 松开按钮
4. 等待 AI 空管回复

## 数据映射

Virtual ATC 从 MSFS 读取以下数据：

| SimConnect 变量 | 说明 | 单位 |
|-----------------|------|------|
| `ATC_ID` | 飞机呼号 | 字符串 |
| `INDICATED_ALTITUDE` | 指示高度 | 英尺 |
| `AIRSPEED_INDICATED` | 指示空速 | 节 |
| `PLANE_HEADING_DEGREES_MAGNETIC` | 磁航向 | 度 |
| `VERTICAL_SPEED` | 垂直速度 | 英尺/分钟 |
| `PLANE_LATITUDE` | 纬度 | 度 |
| `PLANE_LONGITUDE` | 经度 | 度 |
| `SIM_ON_GROUND` | 是否在地面 | 布尔值 |

## 工作原理

### Python 桥接脚本

`scripts/msfs_bridge.py` 的工作流程：

1. **连接 SimConnect**
   ```python
   sm = SimConnect()
   aq = AircraftRequests(sm, _time=200)
   ```

2. **读取飞行数据**
   ```python
   data = {
       "callsign": aq.get("ATC_ID"),
       "altitude": aq.get("INDICATED_ALTITUDE"),
       "speed": aq.get("AIRSPEED_INDICATED"),
       # ...
   }
   ```

3. **通过 UDP 发送**
   ```python
   json_data = json.dumps(data).encode('utf-8')
   sock.sendto(json_data, ("127.0.0.1", 49001))
   ```

4. **每秒更新 5 次**
   ```python
   time.sleep(0.2)  # 200ms
   ```

### Rust 接收端

`src-tauri/src/modules/msfs.rs` 的工作流程：

1. **启动 Python 进程**
   ```rust
   let process = std::process::Command::new("python")
       .arg("scripts/msfs_bridge.py")
       .spawn()?;
   ```

2. **监听 UDP 数据**
   ```rust
   let socket = UdpSocket::bind("127.0.0.1:49001")?;
   ```

3. **解析 JSON**
   ```rust
   let data = serde_json::from_str::<MSFSData>(json_str)?;
   ```

4. **更新飞行数据**
   ```rust
   *flight_data.lock().unwrap() = data;
   ```

## 故障排查

### 无法连接 MSFS

**错误**：`Failed to connect to MSFS`

**解决方案**：
1. 确认 MSFS 正在运行
2. 确认 SimConnect 已启用（默认启用）
3. 检查防火墙是否阻止 Python
4. 尝试以管理员身份运行

### Python 依赖缺失

**错误**：`ModuleNotFoundError: No module named 'SimConnect'`

**解决方案**：
```bash
pip install Python-SimConnect
```

### UDP 端口被占用

**错误**：`Address already in use`

**解决方案**：
1. 检查是否有其他程序占用 49001 端口
2. 修改端口号（需要同时修改 Python 和 Rust 代码）

### 数据不更新

**问题**：连接成功但飞行数据不变化

**解决方案**：
1. 确认 MSFS 中飞机正在飞行（不是主菜单）
2. 检查 Python 脚本是否正常运行
3. 查看控制台日志

### Python 脚本崩溃

**问题**：Python 桥接进程意外退出

**解决方案**：
1. 手动运行 `python scripts/msfs_bridge.py` 查看错误
2. 检查 SimConnect 版本兼容性
3. 重启 MSFS

## 高级配置

### 修改更新频率

编辑 `scripts/msfs_bridge.py`：

```python
# 每秒更新 10 次（更流畅）
time.sleep(0.1)

# 每秒更新 2 次（省资源）
time.sleep(0.5)
```

### 添加更多数据字段

编辑 `scripts/msfs_bridge.py`：

```python
data = {
    "callsign": aq.get("ATC_ID"),
    "altitude": aq.get("INDICATED_ALTITUDE"),
    # 新增字段
    "fuel": aq.get("FUEL_TOTAL_QUANTITY"),
    "flaps": aq.get("FLAPS_HANDLE_PERCENT"),
    "gear": aq.get("GEAR_POSITION"),
}
```

同时更新 `src-tauri/src/modules/msfs.rs` 中的 `MSFSData` 结构体。

### 修改 UDP 端口

如果 49001 端口被占用，可以修改：

**Python 端**（`scripts/msfs_bridge.py`）：
```python
target = ("127.0.0.1", 49002)  # 改为 49002
```

**Rust 端**（`src-tauri/src/modules/msfs.rs`）：
```rust
let socket = UdpSocket::bind("127.0.0.1:49002")?;  // 改为 49002
```

## 性能对比

| 特性 | X-Plane (UDP) | MSFS (SimConnect) |
|------|---------------|-------------------|
| 连接方式 | 直接 UDP | Python 桥接 |
| 延迟 | <50ms | ~100ms |
| CPU 占用 | 低 | 中等 |
| 内存占用 | 低 | 中等（Python 进程） |
| 稳定性 | 高 | 高 |
| 跨平台 | 是 | 仅 Windows |

## 常见问题

### Q: 为什么不直接使用 Rust 调用 SimConnect？

A: SimConnect 是 Windows COM 接口，Rust 的 COM 绑定复杂且不稳定。Python 的 `Python-SimConnect` 库更成熟可靠。

### Q: 可以同时连接 X-Plane 和 MSFS 吗？

A: 不可以。Virtual ATC 一次只能连接一个模拟器。需要先断开当前连接，再连接另一个。

### Q: MSFS 2024 支持吗？

A: 支持。MSFS 2024 使用相同的 SimConnect API。

### Q: 可以在 Linux/macOS 上使用 MSFS 连接吗？

A: 不可以。MSFS 和 SimConnect 都是 Windows 专用。但可以通过网络连接远程 Windows 机器上的 MSFS。

### Q: 如何调试 Python 桥接？

A: 手动运行脚本并查看输出：

```bash
python scripts/msfs_bridge.py
```

所有日志会输出到控制台。

## 参考资料

- [MSFS SimConnect SDK](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/SimConnect_SDK.htm)
- [Python-SimConnect](https://github.com/odwdinc/Python-SimConnect)
- [SimConnect 变量列表](https://docs.flightsimulator.com/html/Programming_Tools/SimVars/Simulation_Variables.htm)

## 贡献

如果你发现 MSFS 集成的问题或有改进建议，欢迎提交 Issue 或 Pull Request！
