# X-Plane 连接配置指南

## 概述

Virtual ATC 通过 UDP 协议与 X-Plane 通信，使用 RREF（Request DataRef）机制订阅飞行数据。

## 配置步骤

### 1. 启动 X-Plane

确保 X-Plane 11 或 X-Plane 12 正在运行。

### 2. 启用 UDP 数据输出（可选）

如果使用传统的 DATA 包方式，需要在 X-Plane 中配置：

1. 打开 X-Plane
2. 进入 **Settings → Data Output**
3. 勾选以下数据组：
   - **3: Speeds** (空速、地速)
   - **17: Pitch, Roll, Heading** (航向)
   - **20: Latitude, Longitude, Altitude** (位置、高度)

4. 设置 UDP 端口：
   - 进入 **Settings → Network**
   - 设置 **UDP Port** 为 `49000`（默认）
   - 勾选 **Send network data output**

### 3. 使用 RREF 方式（推荐）

Virtual ATC 默认使用 RREF 方式，**无需手动配置 X-Plane**。

程序会自动订阅以下 DataRef：

| ID | DataRef | 说明 |
|----|---------|------|
| 1 | `sim/flightmodel/position/indicated_airspeed` | 指示空速（节） |
| 2 | `sim/flightmodel/position/elevation` | 高度（英尺） |
| 3 | `sim/flightmodel/position/psi` | 航向（度） |
| 4 | `sim/flightmodel/position/vh_ind_fpm` | 垂直速度（英尺/分钟） |
| 5 | `sim/flightmodel/position/latitude` | 纬度（度） |
| 6 | `sim/flightmodel/position/longitude` | 经度（度） |

## 工作原理

### RREF 订阅流程

```
Virtual ATC                    X-Plane
    |                              |
    |--- RREF 请求 (订阅数据) ---->|
    |                              |
    |<--- RREF 响应 (每秒1次) -----|
    |                              |
    |<--- RREF 响应 (每秒1次) -----|
    |                              |
```

### 数据包格式

**RREF 请求**：
```
"RREF\0" + freq(4字节) + id(4字节) + dataref_path + "\0"
```

**RREF 响应**：
```
"RREF\0" + id(4字节) + value(4字节float)
```

## 测试连接

### 1. 启动 Virtual ATC

```bash
npm run tauri dev
```

### 2. 点击"连接模拟器"按钮

如果连接成功，控制台会显示：

```
X-Plane UDP listener started on port 49000
Subscribed to X-Plane data
X-Plane data receiver thread started
Flight data: ALT=5000ft SPD=250kts HDG=090° VS=0fpm
```

### 3. 检查飞行数据

前端界面应该显示实时更新的飞行数据：

- **呼号**：CCA123（自动生成）
- **高度**：5000 英尺
- **速度**：250 节
- **航向**：090°
- **垂直速度**：0 英尺/分钟

## 故障排查

### 无法连接

**问题**：点击"连接模拟器"后无响应

**解决方案**：
1. 确认 X-Plane 正在运行
2. 检查防火墙是否阻止 UDP 49000 端口
3. 确认 X-Plane 和 Virtual ATC 在同一台电脑上

### 无数据更新

**问题**：连接成功但数据不更新

**解决方案**：
1. 检查 X-Plane 是否在飞行中（不是主菜单）
2. 查看控制台是否有错误日志
3. 尝试重启 X-Plane 和 Virtual ATC

### 数据不准确

**问题**：显示的数据与 X-Plane 不一致

**解决方案**：
1. 确认使用的是 RREF 方式（推荐）
2. 检查 DataRef 路径是否正确
3. 查看控制台调试输出

## 高级配置

### 修改 UDP 端口

如果 49000 端口被占用，可以修改：

1. 编辑 `src-tauri/src/modules/simulator.rs`
2. 修改 `SimulatorConnection::new()` 中的端口号：

```rust
let xplane = XPlaneConnection::new(49001)?;  // 改为 49001
```

3. 在 X-Plane 中设置相同的端口

### 添加更多 DataRef

如果需要更多飞行数据，编辑 `subscribe_data()` 函数：

```rust
let datarefs = vec![
    (1, "sim/flightmodel/position/indicated_airspeed"),
    (2, "sim/flightmodel/position/elevation"),
    (3, "sim/flightmodel/position/psi"),
    (4, "sim/flightmodel/position/vh_ind_fpm"),
    (5, "sim/flightmodel/position/latitude"),
    (6, "sim/flightmodel/position/longitude"),
    (7, "sim/flightmodel/position/groundspeed"),  // 新增：地速
    (8, "sim/cockpit2/gauges/indicators/radio_altimeter_height_ft_pilot"),  // 新增：雷达高度
];
```

### 调整更新频率

修改 RREF 请求中的频率参数：

```rust
msg.extend_from_slice(&5i32.to_le_bytes());  // 改为每秒5次
```

## 常用 DataRef 列表

| DataRef | 说明 | 单位 |
|---------|------|------|
| `sim/flightmodel/position/indicated_airspeed` | 指示空速 | 节 |
| `sim/flightmodel/position/groundspeed` | 地速 | 米/秒 |
| `sim/flightmodel/position/elevation` | 海拔高度 | 英尺 |
| `sim/flightmodel/position/psi` | 航向 | 度 |
| `sim/flightmodel/position/vh_ind_fpm` | 垂直速度 | 英尺/分钟 |
| `sim/flightmodel/position/latitude` | 纬度 | 度 |
| `sim/flightmodel/position/longitude` | 经度 | 度 |
| `sim/cockpit2/gauges/indicators/radio_altimeter_height_ft_pilot` | 雷达高度 | 英尺 |
| `sim/flightmodel/position/mag_psi` | 磁航向 | 度 |
| `sim/flightmodel/position/true_psi` | 真航向 | 度 |

完整 DataRef 列表：https://developer.x-plane.com/datarefs/

## 参考资料

- [X-Plane UDP 协议文档](https://developer.x-plane.com/article/sending-data-to-x-plane/)
- [X-Plane DataRef 文档](https://developer.x-plane.com/datarefs/)
- [X-Plane SDK](https://developer.x-plane.com/sdk/)
