# X-Plane 集成指南

## 概述
通过 UDP 协议从 X-Plane 读取飞行数据，无需编写插件。

## X-Plane UDP 协议

X-Plane 支持通过 UDP 发送飞行数据（DataRef），这是最简单的集成方式。

### 数据格式
X-Plane 发送的 UDP 数据包格式：
```
Header: "DATA" (4 bytes)
Index: 数据组索引 (4 bytes)
Data: 8个 float32 值 (32 bytes)
```

### 常用数据组索引

| Index | 数据组 | 包含内容 |
|-------|--------|----------|
| 3 | Speeds | 空速、地速、真空速等 |
| 17 | Pitch, Roll, Heading | 俯仰、滚转、航向 |
| 20 | Lat, Lon, Alt | 纬度、经度、高度 |

## 实现步骤

### 1. 配置 X-Plane

在 X-Plane 中启用 UDP 输出：
1. 打开 Settings → Data Output
2. 勾选需要的数据组（3, 17, 20）
3. 选择 "UDP" 列
4. 设置目标 IP 和端口（默认 49000）

### 2. Rust UDP 接收器

```rust
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct XPlaneConnection {
    socket: UdpSocket,
    flight_data: Arc<Mutex<FlightData>>,
    running: Arc<Mutex<bool>>,
}

impl XPlaneConnection {
    pub fn new(port: u16) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(false)?;
        
        Ok(XPlaneConnection {
            socket,
            flight_data: Arc::new(Mutex::new(FlightData::default())),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    pub fn start(&self) -> Result<(), Box<dyn Error>> {
        *self.running.lock().unwrap() = true;
        
        let socket = self.socket.try_clone()?;
        let flight_data = self.flight_data.clone();
        let running = self.running.clone();
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            
            while *running.lock().unwrap() {
                match socket.recv(&mut buf) {
                    Ok(size) => {
                        if let Some(data) = parse_xplane_data(&buf[..size]) {
                            *flight_data.lock().unwrap() = data;
                        }
                    }
                    Err(e) => eprintln!("UDP receive error: {}", e),
                }
            }
        });
        
        Ok(())
    }
    
    pub fn get_flight_data(&self) -> FlightData {
        self.flight_data.lock().unwrap().clone()
    }
    
    pub fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

#[derive(Clone, Default)]
pub struct FlightData {
    pub callsign: String,
    pub altitude: f32,      // 英尺
    pub speed: f32,         // 节
    pub heading: f32,       // 度
    pub latitude: f32,
    pub longitude: f32,
    pub vertical_speed: f32,
}

fn parse_xplane_data(buf: &[u8]) -> Option<FlightData> {
    if buf.len() < 5 || &buf[0..4] != b"DATA" {
        return None;
    }
    
    let mut data = FlightData::default();
    let mut offset = 5;
    
    while offset + 36 <= buf.len() {
        let index = buf[offset] as usize;
        offset += 4;
        
        let values: Vec<f32> = (0..8)
            .map(|i| {
                let bytes = &buf[offset + i * 4..offset + (i + 1) * 4];
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            })
            .collect();
        
        match index {
            3 => {
                // Speeds
                data.speed = values[0]; // 空速（节）
            }
            17 => {
                // Pitch, Roll, Heading
                data.heading = values[2]; // 航向（度）
            }
            20 => {
                // Position
                data.latitude = values[0];
                data.longitude = values[1];
                data.altitude = values[2]; // 高度（英尺）
            }
            _ => {}
        }
        
        offset += 32;
    }
    
    Some(data)
}
```

### 3. 获取呼号

呼号需要通过 DataRef 读取，有两种方式：

#### 方式 A：UDP DataRef 请求
```rust
pub fn request_dataref(socket: &UdpSocket, dataref: &str) -> Result<(), Box<dyn Error>> {
    let mut packet = Vec::new();
    packet.extend_from_slice(b"RREF");
    packet.extend_from_slice(&5u32.to_le_bytes()); // 频率（Hz）
    packet.extend_from_slice(&0u32.to_le_bytes()); // 索引
    packet.extend_from_slice(dataref.as_bytes());
    packet.push(0); // null terminator
    
    socket.send_to(&packet, "127.0.0.1:49000")?;
    Ok(())
}

// 使用
request_dataref(&socket, "sim/aircraft/view/acf_tailnum")?;
```

#### 方式 B：使用默认呼号
```rust
impl FlightData {
    pub fn generate_callsign(&self) -> String {
        // 根据位置生成呼号（简化版）
        format!("CCA{:03}", (self.altitude / 100.0) as u32 % 1000)
    }
}
```

### 4. 集成到 Tauri

更新 `simulator.rs`：

```rust
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct SimulatorConnection {
    xplane: Option<XPlaneConnection>,
}

impl SimulatorConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let xplane = XPlaneConnection::new(49000)?;
        xplane.start()?;
        
        // 等待数据
        thread::sleep(Duration::from_millis(500));
        
        Ok(SimulatorConnection {
            xplane: Some(xplane),
        })
    }
    
    pub fn get_flight_data(&self) -> Result<FlightData, Box<dyn Error>> {
        match &self.xplane {
            Some(conn) => {
                let mut data = conn.get_flight_data();
                
                // 如果没有呼号，生成一个
                if data.callsign.is_empty() {
                    data.callsign = format!("CCA{:03}", (data.altitude / 100.0) as u32 % 1000);
                }
                
                Ok(data)
            }
            None => Err("Not connected".into()),
        }
    }
}
```

## 测试

### 1. 启动 X-Plane
确保 X-Plane 正在运行并已配置 UDP 输出。

### 2. 测试 UDP 接收
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xplane_connection() {
        let conn = XPlaneConnection::new(49000).unwrap();
        conn.start().unwrap();
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let data = conn.get_flight_data();
        println!("Altitude: {} ft", data.altitude);
        println!("Speed: {} kts", data.speed);
        println!("Heading: {}°", data.heading);
    }
}
```

### 3. 使用 netcat 测试
```bash
# 监听 UDP 端口
nc -ul 49000
```

## 高级功能

### 1. 发送命令到 X-Plane
```rust
pub fn send_command(socket: &UdpSocket, command: &str) -> Result<(), Box<dyn Error>> {
    let mut packet = Vec::new();
    packet.extend_from_slice(b"CMND");
    packet.extend_from_slice(command.as_bytes());
    packet.push(0);
    
    socket.send_to(&packet, "127.0.0.1:49000")?;
    Ok(())
}

// 示例：设置频率
send_command(&socket, "sim/radios/com1_standy_flip")?;
```

### 2. 读取更多 DataRef
```rust
// 请求多个 DataRef
request_dataref(&socket, "sim/aircraft/view/acf_tailnum")?;
request_dataref(&socket, "sim/flightmodel/position/indicated_airspeed")?;
request_dataref(&socket, "sim/flightmodel/position/vh_ind")?; // 垂直速度
```

## 故障排查

### 问题 1：收不到数据
- 检查 X-Plane Data Output 设置
- 确认端口号正确（默认 49000）
- 检查防火墙设置

### 问题 2：数据不更新
- 确认 X-Plane 正在飞行（不是暂停状态）
- 检查 UDP 频率设置（建议 5-10 Hz）

### 问题 3：数据不准确
- 验证数据组索引是否正确
- 检查字节序（little-endian）

## 参考资料

- [X-Plane UDP 文档](https://developer.x-plane.com/article/sending-data-to-x-plane/)
- [DataRef 列表](https://developer.x-plane.com/datarefs/)
- [X-Plane SDK](https://developer.x-plane.com/sdk/)
