use std::error::Error;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

pub struct SimulatorConnection {
    xplane: Option<XPlaneConnection>,
    flight_data: Arc<Mutex<FlightData>>,
}

impl SimulatorConnection {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let xplane = XPlaneConnection::new(49000)?;
        let flight_data = xplane.flight_data.clone();
        
        // 启动数据接收线程
        xplane.start()?;
        
        // 发送数据订阅请求
        xplane.subscribe_data()?;
        
        // 等待初始数据
        thread::sleep(Duration::from_millis(500));
        
        Ok(SimulatorConnection {
            xplane: Some(xplane),
            flight_data,
        })
    }
    
    pub fn get_flight_data(&self) -> Result<FlightData, Box<dyn Error>> {
        let mut data = self.flight_data.lock().unwrap().clone();
        
        // 如果没有呼号，生成一个
        if data.callsign.is_empty() {
            data.callsign = format!("CCA{:03}", (data.altitude / 100.0) as u32 % 1000);
        }
        
        Ok(data)
    }
}

struct XPlaneConnection {
    socket: UdpSocket,
    xplane_addr: String,
    flight_data: Arc<Mutex<FlightData>>,
    running: Arc<Mutex<bool>>,
}

impl XPlaneConnection {
    fn new(port: u16) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(false)?;
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        
        println!("X-Plane UDP listener started on port {}", port);
        
        Ok(XPlaneConnection {
            socket,
            xplane_addr: "127.0.0.1:49000".to_string(),
            flight_data: Arc::new(Mutex::new(FlightData::default())),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    fn subscribe_data(&self) -> Result<(), Box<dyn Error>> {
        // 发送 RREF 请求订阅数据
        // 格式: RREF\0 + freq(4字节) + id(4字节) + dataref_path
        
        let datarefs = vec![
            (1, "sim/flightmodel/position/indicated_airspeed"),  // 空速
            (2, "sim/flightmodel/position/elevation"),           // 高度（英尺）
            (3, "sim/flightmodel/position/psi"),                 // 航向
            (4, "sim/flightmodel/position/vh_ind_fpm"),          // 垂直速度
            (5, "sim/flightmodel/position/latitude"),            // 纬度
            (6, "sim/flightmodel/position/longitude"),           // 经度
        ];
        
        for (id, dataref) in datarefs {
            let mut msg = b"RREF\0".to_vec();
            msg.extend_from_slice(&1i32.to_le_bytes());  // 频率：每秒1次
            msg.extend_from_slice(&id.to_le_bytes());    // ID
            msg.extend_from_slice(dataref.as_bytes());
            msg.push(0);  // null terminator
            
            // 填充到400字节
            while msg.len() < 413 {
                msg.push(0);
            }
            
            self.socket.send_to(&msg, &self.xplane_addr)?;
        }
        
        println!("Subscribed to X-Plane data");
        Ok(())
    }
    
    fn start(&self) -> Result<(), Box<dyn Error>> {
        *self.running.lock().unwrap() = true;
        
        let socket = self.socket.try_clone()?;
        let flight_data = self.flight_data.clone();
        let running = self.running.clone();
        
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            
            println!("X-Plane data receiver thread started");
            
            while *running.lock().unwrap() {
                match socket.recv(&mut buf) {
                    Ok(size) => {
                        if let Some(data) = parse_xplane_packet(&buf[..size]) {
                            let mut current = flight_data.lock().unwrap();
                            
                            // 更新数据
                            if data.altitude > 0.0 {
                                current.altitude = data.altitude;
                            }
                            if data.speed > 0.0 {
                                current.speed = data.speed;
                            }
                            if data.heading >= 0.0 {
                                current.heading = data.heading;
                            }
                            if data.vertical_speed != 0.0 {
                                current.vertical_speed = data.vertical_speed;
                            }
                            if data.latitude != 0.0 {
                                current.latitude = data.latitude;
                            }
                            if data.longitude != 0.0 {
                                current.longitude = data.longitude;
                            }
                            
                            // 调试输出
                            if current.altitude > 0.0 {
                                println!("Flight data: ALT={:.0}ft SPD={:.0}kts HDG={:.0}° VS={:.0}fpm", 
                                    current.altitude, current.speed, current.heading, current.vertical_speed);
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
            
            println!("X-Plane data receiver thread stopped");
        });
        
        Ok(())
    }
}

impl Drop for XPlaneConnection {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct FlightData {
    pub callsign: String,
    pub altitude: f64,      // 英尺
    pub speed: f64,         // 节
    pub heading: f64,       // 度
    pub vertical_speed: f64, // 英尺/分钟
    pub latitude: f64,      // 度
    pub longitude: f64,     // 度
}

impl Default for FlightData {
    fn default() -> Self {
        FlightData {
            callsign: String::new(),
            altitude: 0.0,
            speed: 0.0,
            heading: 0.0,
            vertical_speed: 0.0,
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

fn parse_xplane_packet(buf: &[u8]) -> Option<FlightData> {
    if buf.len() < 5 {
        return None;
    }
    
    let header = &buf[0..4];
    
    // 处理 RREF 响应
    if header == b"RREF" {
        return parse_rref_data(&buf[5..]);
    }
    
    // 处理 DATA 包
    if header == b"DATA" {
        return parse_data_packet(&buf[5..]);
    }
    
    None
}

fn parse_rref_data(buf: &[u8]) -> Option<FlightData> {
    if buf.len() < 8 {
        return None;
    }
    
    let mut data = FlightData::default();
    let mut offset = 0;
    
    while offset + 8 <= buf.len() {
        let id = i32::from_le_bytes([buf[offset], buf[offset+1], buf[offset+2], buf[offset+3]]);
        let value = f32::from_le_bytes([buf[offset+4], buf[offset+5], buf[offset+6], buf[offset+7]]);
        
        match id {
            1 => data.speed = value as f64,           // 空速
            2 => data.altitude = value as f64,        // 高度
            3 => data.heading = value as f64,         // 航向
            4 => data.vertical_speed = value as f64,  // 垂直速度
            5 => data.latitude = value as f64,        // 纬度
            6 => data.longitude = value as f64,       // 经度
            _ => {}
        }
        
        offset += 8;
    }
    
    Some(data)
}

fn parse_data_packet(buf: &[u8]) -> Option<FlightData> {
    let mut data = FlightData::default();
    let mut offset = 0;
    
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
                // Speeds - 空速（节）
                data.speed = values[0] as f64;
            }
            17 => {
                // Pitch, Roll, Heading - 航向（度）
                data.heading = values[2] as f64;
            }
            20 => {
                // Position - 高度（英尺）
                data.altitude = values[2] as f64;
            }
            _ => {}
        }
        
        offset += 32;
    }
    
    Some(data)
}
