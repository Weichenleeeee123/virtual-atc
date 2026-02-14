use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightPhase {
    PreFlight,      // 飞行前（停机坪）
    Taxi,           // 滑行
    Takeoff,        // 起飞
    Climb,          // 爬升
    Cruise,         // 巡航
    Descent,        // 下降
    Approach,       // 进近
    Landing,        // 着陆
    GoAround,       // 复飞
}

impl FlightPhase {
    pub fn as_str(&self) -> &str {
        match self {
            FlightPhase::PreFlight => "pre_flight",
            FlightPhase::Taxi => "taxi",
            FlightPhase::Takeoff => "takeoff",
            FlightPhase::Climb => "climb",
            FlightPhase::Cruise => "cruise",
            FlightPhase::Descent => "descent",
            FlightPhase::Approach => "approach",
            FlightPhase::Landing => "landing",
            FlightPhase::GoAround => "go_around",
        }
    }
    
    pub fn display_name(&self) -> &str {
        match self {
            FlightPhase::PreFlight => "飞行前",
            FlightPhase::Taxi => "滑行",
            FlightPhase::Takeoff => "起飞",
            FlightPhase::Climb => "爬升",
            FlightPhase::Cruise => "巡航",
            FlightPhase::Descent => "下降",
            FlightPhase::Approach => "进近",
            FlightPhase::Landing => "着陆",
            FlightPhase::GoAround => "复飞",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlightData {
    pub altitude: f64,           // 英尺
    pub speed: f64,              // 节
    pub heading: f64,            // 度
    pub vertical_speed: f64,     // 英尺/分钟
    pub on_ground: bool,         // 是否在地面
}

pub struct FlightPhaseDetector {
    current_phase: FlightPhase,
    previous_altitude: f64,
    previous_speed: f64,
    phase_start_time: std::time::Instant,
}

impl FlightPhaseDetector {
    pub fn new() -> Self {
        FlightPhaseDetector {
            current_phase: FlightPhase::PreFlight,
            previous_altitude: 0.0,
            previous_speed: 0.0,
            phase_start_time: std::time::Instant::now(),
        }
    }
    
    pub fn update(&mut self, data: &FlightData) -> FlightPhase {
        let new_phase = self.detect_phase(data);
        
        if new_phase != self.current_phase {
            println!("Flight phase changed: {:?} -> {:?}", self.current_phase, new_phase);
            self.current_phase = new_phase;
            self.phase_start_time = std::time::Instant::now();
        }
        
        self.previous_altitude = data.altitude;
        self.previous_speed = data.speed;
        
        self.current_phase
    }
    
    pub fn get_current_phase(&self) -> FlightPhase {
        self.current_phase
    }
    
    pub fn get_phase_duration(&self) -> std::time::Duration {
        self.phase_start_time.elapsed()
    }
    
    fn detect_phase(&self, data: &FlightData) -> FlightPhase {
        let alt = data.altitude;
        let spd = data.speed;
        let vs = data.vertical_speed;
        let on_ground = data.on_ground;
        
        // 飞行前：在地面，速度很低
        if on_ground && spd < 5.0 {
            return FlightPhase::PreFlight;
        }
        
        // 滑行：在地面，速度较低
        if on_ground && spd >= 5.0 && spd < 60.0 {
            return FlightPhase::Taxi;
        }
        
        // 起飞：在地面或低空，速度快，正在加速
        if (on_ground || alt < 500.0) && spd >= 60.0 && vs > 500.0 {
            return FlightPhase::Takeoff;
        }
        
        // 爬升：低于巡航高度，垂直速度为正
        if !on_ground && alt < 20000.0 && vs > 300.0 {
            return FlightPhase::Climb;
        }
        
        // 巡航：高空，垂直速度接近零
        if !on_ground && alt >= 20000.0 && vs.abs() < 500.0 {
            return FlightPhase::Cruise;
        }
        
        // 下降：从高空下降，垂直速度为负
        if !on_ground && alt > 5000.0 && vs < -300.0 {
            return FlightPhase::Descent;
        }
        
        // 进近：低空，下降中
        if !on_ground && alt >= 500.0 && alt <= 5000.0 && vs < -100.0 {
            return FlightPhase::Approach;
        }
        
        // 着陆：极低空或刚接地
        if (on_ground && spd > 60.0) || (!on_ground && alt < 500.0 && vs < -100.0) {
            return FlightPhase::Landing;
        }
        
        // 复飞：低空突然爬升
        if !on_ground && alt < 3000.0 && vs > 1000.0 && self.current_phase == FlightPhase::Approach {
            return FlightPhase::GoAround;
        }
        
        // 默认保持当前阶段
        self.current_phase
    }
    
    /// 获取当前阶段的 ATC 提示词
    pub fn get_atc_context(&self, language: &str) -> String {
        match language {
            "zh" => self.get_chinese_context(),
            _ => self.get_english_context(),
        }
    }
    
    fn get_chinese_context(&self) -> String {
        match self.current_phase {
            FlightPhase::PreFlight => {
                "飞行员正在准备起飞。你应该提供：\n\
                - 天气信息\n\
                - 跑道信息\n\
                - 放行许可\n\
                - 滑行指令"
            }
            FlightPhase::Taxi => {
                "飞行员正在滑行。你应该提供：\n\
                - 滑行路线指引\n\
                - 等待指令\n\
                - 跑道穿越许可"
            }
            FlightPhase::Takeoff => {
                "飞行员正在起飞。你应该提供：\n\
                - 起飞许可\n\
                - 初始爬升指令\n\
                - 离场航向"
            }
            FlightPhase::Climb => {
                "飞行员正在爬升。你应该提供：\n\
                - 爬升高度指令\n\
                - 航向调整\n\
                - 频率切换"
            }
            FlightPhase::Cruise => {
                "飞行员正在巡航。你应该提供：\n\
                - 高度保持确认\n\
                - 航路调整\n\
                - 天气信息"
            }
            FlightPhase::Descent => {
                "飞行员正在下降。你应该提供：\n\
                - 下降许可\n\
                - 目标高度\n\
                - 进近准备"
            }
            FlightPhase::Approach => {
                "飞行员正在进近。你应该提供：\n\
                - 进近许可\n\
                - 最后进近指令\n\
                - 着陆许可"
            }
            FlightPhase::Landing => {
                "飞行员正在着陆。你应该提供：\n\
                - 着陆许可确认\n\
                - 脱离跑道指令\n\
                - 滑行指引"
            }
            FlightPhase::GoAround => {
                "飞行员正在复飞。你应该提供：\n\
                - 复飞指令确认\n\
                - 爬升高度\n\
                - 复飞航向"
            }
        }.to_string()
    }
    
    fn get_english_context(&self) -> String {
        match self.current_phase {
            FlightPhase::PreFlight => {
                "Pilot is preparing for departure. You should provide:\n\
                - Weather information\n\
                - Runway information\n\
                - Clearance delivery\n\
                - Taxi instructions"
            }
            FlightPhase::Taxi => {
                "Pilot is taxiing. You should provide:\n\
                - Taxi route guidance\n\
                - Hold short instructions\n\
                - Runway crossing clearance"
            }
            FlightPhase::Takeoff => {
                "Pilot is taking off. You should provide:\n\
                - Takeoff clearance\n\
                - Initial climb instructions\n\
                - Departure heading"
            }
            FlightPhase::Climb => {
                "Pilot is climbing. You should provide:\n\
                - Climb altitude instructions\n\
                - Heading adjustments\n\
                - Frequency changes"
            }
            FlightPhase::Cruise => {
                "Pilot is cruising. You should provide:\n\
                - Altitude maintenance confirmation\n\
                - Route adjustments\n\
                - Weather information"
            }
            FlightPhase::Descent => {
                "Pilot is descending. You should provide:\n\
                - Descent clearance\n\
                - Target altitude\n\
                - Approach preparation"
            }
            FlightPhase::Approach => {
                "Pilot is on approach. You should provide:\n\
                - Approach clearance\n\
                - Final approach instructions\n\
                - Landing clearance"
            }
            FlightPhase::Landing => {
                "Pilot is landing. You should provide:\n\
                - Landing clearance confirmation\n\
                - Runway exit instructions\n\
                - Taxi guidance"
            }
            FlightPhase::GoAround => {
                "Pilot is going around. You should provide:\n\
                - Go-around instruction confirmation\n\
                - Climb altitude\n\
                - Go-around heading"
            }
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preflight_detection() {
        let mut detector = FlightPhaseDetector::new();
        let data = FlightData {
            altitude: 0.0,
            speed: 0.0,
            heading: 0.0,
            vertical_speed: 0.0,
            on_ground: true,
        };
        
        let phase = detector.update(&data);
        assert_eq!(phase, FlightPhase::PreFlight);
    }
    
    #[test]
    fn test_taxi_detection() {
        let mut detector = FlightPhaseDetector::new();
        let data = FlightData {
            altitude: 0.0,
            speed: 20.0,
            heading: 90.0,
            vertical_speed: 0.0,
            on_ground: true,
        };
        
        let phase = detector.update(&data);
        assert_eq!(phase, FlightPhase::Taxi);
    }
    
    #[test]
    fn test_takeoff_detection() {
        let mut detector = FlightPhaseDetector::new();
        let data = FlightData {
            altitude: 100.0,
            speed: 120.0,
            heading: 90.0,
            vertical_speed: 1500.0,
            on_ground: false,
        };
        
        let phase = detector.update(&data);
        assert_eq!(phase, FlightPhase::Takeoff);
    }
    
    #[test]
    fn test_cruise_detection() {
        let mut detector = FlightPhaseDetector::new();
        let data = FlightData {
            altitude: 35000.0,
            speed: 450.0,
            heading: 270.0,
            vertical_speed: 0.0,
            on_ground: false,
        };
        
        let phase = detector.update(&data);
        assert_eq!(phase, FlightPhase::Cruise);
    }
}
