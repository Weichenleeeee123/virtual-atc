# MSFS SimConnect 集成指南

## 概述
通过 SimConnect API 从 Microsoft Flight Simulator 读取飞行数据。

## SimConnect 简介

SimConnect 是 MSFS 的官方 API，支持：
- 读取飞行数据（位置、速度、高度等）
- 发送命令（设置频率、自动驾驶等）
- 订阅事件（起飞、降落等）

## 前置要求

### 1. SimConnect SDK
MSFS 安装时会自动安装 SimConnect SDK：
```
C:\MSFS SDK\SimConnect SDK
```

### 2. Rust 绑定
使用 `simconnect` crate（社区维护）：

```toml
[dependencies]
simconnect = "0.4"
```

或者使用 `windows` crate 直接调用 COM 接口：

```toml
[dependencies.windows]
version = "0.51"
features = [
    "Win32_System_Com",
    "Win32_Foundation",
]
```

## 实现步骤

### 1. 基础连接

```rust
use simconnect;

pub struct MSFSConnection {
    conn: simconnect::SimConnector,
}

impl MSFSConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let conn = simconnect::SimConnector::new();
        
        // 连接到 MSFS
        conn.open("Virtual ATC")?;
        
        Ok(MSFSConnection { conn })
    }
    
    pub fn is_connected(&self) -> bool {
        self.conn.is_connected()
    }
}
```

### 2. 定义数据结构

```rust
use simconnect::*;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AircraftData {
    pub title: [u8; 256],           // 飞机型号
    pub atc_id: [u8; 32],           // ATC 呼号
    pub altitude: f64,              // 高度（英尺）
    pub indicated_airspeed: f64,    // 指示空速（节）
    pub heading: f64,               // 航向（度）
    pub vertical_speed: f64,        // 垂直速度（英尺/分钟）
    pub latitude: f64,              // 纬度
    pub longitude: f64,             // 经度
    pub on_ground: i32,             // 是否在地面
}

impl Default for AircraftData {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
```

### 3. 注册数据定义

```rust
impl MSFSConnection {
    pub fn register_data_definitions(&mut self) -> Result<(), Box<dyn Error>> {
        // 定义数据请求
        self.conn.add_data_definition(
            0, // 定义 ID
            "TITLE",
            "",
            SIMCONNECT_DATATYPE_STRING256,
        )?;
        
        self.conn.add_data_definition(
            0,
            "ATC ID",
            "",
            SIMCONNECT_DATATYPE_STRING32,
        )?;
        
        self.conn.add_data_definition(
            0,
            "PLANE ALTITUDE",
            "feet",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "AIRSPEED INDICATED",
            "knots",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "PLANE HEADING DEGREES TRUE",
            "degrees",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "VERTICAL SPEED",
            "feet per minute",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "PLANE LATITUDE",
            "degrees",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "PLANE LONGITUDE",
            "degrees",
            SIMCONNECT_DATATYPE_FLOAT64,
        )?;
        
        self.conn.add_data_definition(
            0,
            "SIM ON GROUND",
            "bool",
            SIMCONNECT_DATATYPE_INT32,
        )?;
        
        Ok(())
    }
}
```

### 4. 请求数据

```rust
impl MSFSConnection {
    pub fn request_data(&mut self) -> Result<(), Box<dyn Error>> {
        self.conn.request_data_on_sim_object(
            0,  // 请求 ID
            0,  // 定义 ID
            SIMCONNECT_OBJECT_ID_USER,  // 用户飞机
            SIMCONNECT_PERIOD_SECOND,   // 每秒更新
            SIMCONNECT_DATA_REQUEST_FLAG_DEFAULT,
        )?;
        
        Ok(())
    }
    
    pub fn get_flight_data(&mut self) -> Result<FlightData, Box<dyn Error>> {
        // 处理消息
        self.conn.call_dispatch()?;
        
        // 获取数据
        let data = self.conn.get_next_message::<AircraftData>()?;
        
        Ok(FlightData {
            callsign: String::from_utf8_lossy(&data.atc_id)
                .trim_end_matches('\0')
                .to_string(),
            altitude: data.altitude as f32,
            speed: data.indicated_airspeed as f32,
            heading: data.heading as f32,
            vertical_speed: data.vertical_speed as f32,
            latitude: data.latitude as f32,
            longitude: data.longitude as f32,
        })
    }
}
```

### 5. 简化版实现（推荐）

如果 `simconnect` crate 不可用，使用更简单的方式：

```rust
use std::process::Command;
use serde_json::Value;

pub struct MSFSConnection {
    // 使用外部工具或 WebSocket
}

impl MSFSConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // 方案 1：使用 FSUIPC（第三方工具）
        // 方案 2：使用 SimConnect WebSocket 桥接
        // 方案 3：使用 Python 脚本 + IPC
        
        Ok(MSFSConnection {})
    }
    
    pub fn get_flight_data(&self) -> Result<FlightData, Box<dyn Error>> {
        // 通过外部工具获取数据
        // 例如：调用 Python 脚本读取 SimConnect
        
        let output = Command::new("python")
            .arg("scripts/read_simconnect.py")
            .output()?;
        
        let json: Value = serde_json::from_slice(&output.stdout)?;
        
        Ok(FlightData {
            callsign: json["callsign"].as_str().unwrap_or("").to_string(),
            altitude: json["altitude"].as_f64().unwrap_or(0.0) as f32,
            speed: json["speed"].as_f64().unwrap_or(0.0) as f32,
            heading: json["heading"].as_f64().unwrap_or(0.0) as f32,
            vertical_speed: 0.0,
            latitude: 0.0,
            longitude: 0.0,
        })
    }
}
```

### 6. Python SimConnect 脚本

```python
# scripts/read_simconnect.py
from SimConnect import SimConnect
import json

def main():
    sm = SimConnect()
    aq = sm.new_request()
    
    # 请求数据
    aq.add("ATC_ID", "string")
    aq.add("PLANE_ALTITUDE", "feet")
    aq.add("AIRSPEED_INDICATED", "knots")
    aq.add("PLANE_HEADING_DEGREES_TRUE", "degrees")
    
    data = aq.get()
    
    # 输出 JSON
    result = {
        "callsign": data["ATC_ID"],
        "altitude": data["PLANE_ALTITUDE"],
        "speed": data["AIRSPEED_INDICATED"],
        "heading": data["PLANE_HEADING_DEGREES_TRUE"]
    }
    
    print(json.dumps(result))

if __name__ == "__main__":
    main()
```

安装 Python SimConnect：
```bash
pip install SimConnect
```

## 集成到项目

更新 `simulator.rs`：

```rust
pub struct SimulatorConnection {
    xplane: Option<XPlaneConnection>,
    msfs: Option<MSFSConnection>,
    sim_type: SimulatorType,
}

pub enum SimulatorType {
    XPlane,
    MSFS,
    Auto,
}

impl SimulatorConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // 自动检测模拟器
        let sim_type = Self::detect_simulator();
        
        let (xplane, msfs) = match sim_type {
            SimulatorType::XPlane => {
                let xp = XPlaneConnection::new(49000)?;
                xp.start()?;
                (Some(xp), None)
            }
            SimulatorType::MSFS => {
                let ms = MSFSConnection::new()?;
                (None, Some(ms))
            }
            SimulatorType::Auto => {
                // 尝试两者
                if let Ok(xp) = XPlaneConnection::new(49000) {
                    xp.start()?;
                    (Some(xp), None)
                } else if let Ok(ms) = MSFSConnection::new() {
                    (None, Some(ms))
                } else {
                    return Err("No simulator detected".into());
                }
            }
        };
        
        Ok(SimulatorConnection {
            xplane,
            msfs,
            sim_type,
        })
    }
    
    fn detect_simulator() -> SimulatorType {
        // 检测 MSFS 进程
        if Self::is_process_running("FlightSimulator.exe") {
            return SimulatorType::MSFS;
        }
        
        // 检测 X-Plane 进程
        if Self::is_process_running("X-Plane.exe") {
            return SimulatorType::XPlane;
        }
        
        SimulatorType::Auto
    }
    
    fn is_process_running(name: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let output = Command::new("tasklist")
                .output()
                .ok();
            
            if let Some(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains(name);
            }
        }
        
        false
    }
    
    pub fn get_flight_data(&mut self) -> Result<FlightData, Box<dyn Error>> {
        if let Some(xp) = &self.xplane {
            xp.get_flight_data()
        } else if let Some(ms) = &mut self.msfs {
            ms.get_flight_data()
        } else {
            Err("No simulator connected".into())
        }
    }
}
```

## 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_msfs_connection() {
        let mut conn = MSFSConnection::new().unwrap();
        conn.register_data_definitions().unwrap();
        conn.request_data().unwrap();
        
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        let data = conn.get_flight_data().unwrap();
        println!("Callsign: {}", data.callsign);
        println!("Altitude: {} ft", data.altitude);
    }
}
```

## 故障排查

### 问题 1：无法连接
- 确认 MSFS 正在运行
- 检查 SimConnect.cfg 配置
- 确认防火墙允许连接

### 问题 2：数据为空
- 确认飞机已加载
- 检查数据定义是否正确
- 验证变量名称（区分大小写）

### 问题 3：性能问题
- 降低数据请求频率
- 只请求必要的数据
- 使用异步处理

## 参考资料

- [SimConnect SDK 文档](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/SimConnect_SDK.htm)
- [Python SimConnect](https://github.com/odwdinc/Python-SimConnect)
- [FSUIPC](http://www.fsuipc.com/)
