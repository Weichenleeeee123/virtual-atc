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
        xplane.start()?;
        
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
    flight_data: Arc<Mutex<FlightData>>,
    running: Arc<Mutex<bool>>,
}

impl XPlaneConnection {
    fn new(port: u16) -> Result<Self, Box<dyn Error>> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(false)?;
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        
        Ok(XPlaneConnection {
            socket,
            flight_data: Arc::new(Mutex::new(FlightData::default())),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    fn start(&self) -> Result<(), Box<dyn Error>> {
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
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock 
                            && e.kind() != std::io::ErrorKind::TimedOut {
                            eprintln!("UDP receive error: {}", e);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct FlightData {
    pub callsign: String,
    pub altitude: f64,
    pub speed: f64,
    pub heading: f64,
}

impl Default for FlightData {
    fn default() -> Self {
        FlightData {
            callsign: String::new(),
            altitude: 0.0,
            speed: 0.0,
            heading: 0.0,
        }
    }
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
