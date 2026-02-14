use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// SimConnect 数据结构
#[derive(Debug, Clone)]
pub struct MSFSData {
    pub callsign: String,
    pub altitude: f64,      // 英尺
    pub speed: f64,         // 节
    pub heading: f64,       // 度
    pub vertical_speed: f64, // 英尺/分钟
    pub latitude: f64,      // 度
    pub longitude: f64,     // 度
    pub on_ground: bool,    // 是否在地面
}

impl Default for MSFSData {
    fn default() -> Self {
        MSFSData {
            callsign: String::new(),
            altitude: 0.0,
            speed: 0.0,
            heading: 0.0,
            vertical_speed: 0.0,
            latitude: 0.0,
            longitude: 0.0,
            on_ground: true,
        }
    }
}

pub struct MSFSConnection {
    flight_data: Arc<Mutex<MSFSData>>,
    running: Arc<Mutex<bool>>,
    python_process: Option<std::process::Child>,
}

impl MSFSConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let flight_data = Arc::new(Mutex::new(MSFSData::default()));
        let running = Arc::new(Mutex::new(false));
        
        Ok(MSFSConnection {
            flight_data,
            running,
            python_process: None,
        })
    }
    
    pub fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        // 检查 Python 脚本是否存在
        let script_path = "scripts/msfs_bridge.py";
        if !std::path::Path::new(script_path).exists() {
            return Err("MSFS bridge script not found. Please run setup first.".into());
        }
        
        // 启动 Python 桥接进程
        let process = std::process::Command::new("python")
            .arg(script_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        
        self.python_process = Some(process);
        *self.running.lock().unwrap() = true;
        
        // 启动数据接收线程
        self.start_receiver()?;
        
        println!("MSFS SimConnect bridge started");
        Ok(())
    }
    
    fn start_receiver(&self) -> Result<(), Box<dyn Error>> {
        let flight_data = self.flight_data.clone();
        let running = self.running.clone();
        
        thread::spawn(move || {
            use std::net::UdpSocket;
            
            // 监听 Python 桥接发送的数据
            let socket = match UdpSocket::bind("127.0.0.1:49001") {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to bind UDP socket: {}", e);
                    return;
                }
            };
            
            socket.set_read_timeout(Some(Duration::from_secs(1))).ok();
            
            let mut buf = [0u8; 1024];
            
            while *running.lock().unwrap() {
                match socket.recv(&mut buf) {
                    Ok(size) => {
                        if let Ok(json_str) = std::str::from_utf8(&buf[..size]) {
                            if let Ok(data) = serde_json::from_str::<MSFSData>(json_str) {
                                *flight_data.lock().unwrap() = data;
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock 
                            && e.kind() != std::io::ErrorKind::TimedOut {
                            eprintln!("UDP receive error: {}", e);
                        }
                    }
                }
            }
            
            println!("MSFS data receiver stopped");
        });
        
        Ok(())
    }
    
    pub fn get_flight_data(&self) -> Result<MSFSData, Box<dyn Error>> {
        let mut data = self.flight_data.lock().unwrap().clone();
        
        // 如果没有呼号，生成一个
        if data.callsign.is_empty() {
            data.callsign = format!("MSFS{:03}", (data.altitude / 100.0) as u32 % 1000);
        }
        
        Ok(data)
    }
    
    pub fn disconnect(&mut self) {
        *self.running.lock().unwrap() = false;
        
        // 终止 Python 进程
        if let Some(mut process) = self.python_process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        
        println!("MSFS SimConnect disconnected");
    }
}

impl Drop for MSFSConnection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// 为了兼容性，实现 Serialize
impl serde::Serialize for MSFSData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MSFSData", 8)?;
        state.serialize_field("callsign", &self.callsign)?;
        state.serialize_field("altitude", &self.altitude)?;
        state.serialize_field("speed", &self.speed)?;
        state.serialize_field("heading", &self.heading)?;
        state.serialize_field("vertical_speed", &self.vertical_speed)?;
        state.serialize_field("latitude", &self.latitude)?;
        state.serialize_field("longitude", &self.longitude)?;
        state.serialize_field("on_ground", &self.on_ground)?;
        state.end()
    }
}

// 为了兼容性，实现 Deserialize
impl<'de> serde::Deserialize<'de> for MSFSData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Helper {
            callsign: String,
            altitude: f64,
            speed: f64,
            heading: f64,
            vertical_speed: f64,
            latitude: f64,
            longitude: f64,
            on_ground: bool,
        }
        
        let helper = Helper::deserialize(deserializer)?;
        Ok(MSFSData {
            callsign: helper.callsign,
            altitude: helper.altitude,
            speed: helper.speed,
            heading: helper.heading,
            vertical_speed: helper.vertical_speed,
            latitude: helper.latitude,
            longitude: helper.longitude,
            on_ground: helper.on_ground,
        })
    }
}
