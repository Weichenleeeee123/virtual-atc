use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Little Navmap 数据库连接器
pub struct LittleNavmapDB {
    conn: Connection,
    db_type: DatabaseType,
}

#[derive(Debug, Clone)]
pub enum DatabaseType {
    Navigraph,
    XPlane,
    MSFS,
}

impl LittleNavmapDB {
    /// 自动检测并连接 Little Navmap 数据库
    pub fn new() -> Result<Self> {
        Self::auto_detect()
    }
    
    /// 自动检测并连接 Little Navmap 数据库
    pub fn auto_detect() -> Result<Self> {
        let db_paths = Self::get_database_paths();
        
        // 优先使用 Navigraph 数据库
        for (db_type, path) in db_paths {
            if path.exists() {
                println!("检测到 Little Navmap 数据库: {:?} at {:?}", db_type, path);
                let conn = Connection::open_with_flags(
                    &path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
                )?;
                return Ok(LittleNavmapDB { conn, db_type });
            }
        }
        
        Err(rusqlite::Error::InvalidPath(PathBuf::from(
            "未找到 Little Navmap 数据库。请确保已安装 Little Navmap。"
        )))
    }
    
    /// 获取数据库路径（按优先级排序）
    fn get_database_paths() -> Vec<(DatabaseType, PathBuf)> {
        let mut paths = Vec::new();
        
        #[cfg(target_os = "windows")]
        {
            if let Some(appdata) = dirs::config_dir() {
                let base = appdata.join("ABarthel").join("little_navmap_db");
                paths.push((DatabaseType::Navigraph, base.join("little_navmap_navigraph.sqlite")));
                paths.push((DatabaseType::XPlane, base.join("little_navmap_xp12.sqlite")));
                paths.push((DatabaseType::MSFS, base.join("little_navmap_msfs.sqlite")));
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Some(home) = dirs::home_dir() {
                let base = home
                    .join("Library")
                    .join("Application Support")
                    .join("ABarthel")
                    .join("little_navmap_db");
                paths.push((DatabaseType::Navigraph, base.join("little_navmap_navigraph.sqlite")));
                paths.push((DatabaseType::XPlane, base.join("little_navmap_xp12.sqlite")));
                paths.push((DatabaseType::MSFS, base.join("little_navmap_msfs.sqlite")));
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Some(config) = dirs::config_dir() {
                let base = config
                    .join("ABarthel")
                    .join("little_navmap_db");
                paths.push((DatabaseType::Navigraph, base.join("little_navmap_navigraph.sqlite")));
                paths.push((DatabaseType::XPlane, base.join("little_navmap_xp12.sqlite")));
                paths.push((DatabaseType::MSFS, base.join("little_navmap_msfs.sqlite")));
            }
        }
        
        paths
    }
    
    /// 获取机场完整信息
    pub fn get_airport(&self, icao: &str) -> Result<AirportData> {
        let mut stmt = self.conn.prepare(
            "SELECT airport_id, ident, name, city, country, lat, lonx, altitude, 
                    tower_frequency, atis_frequency, awos_frequency, asos_frequency,
                    unicom_frequency
             FROM airport WHERE ident = ?1"
        )?;
        
        let airport = stmt.query_row([icao], |row| {
            Ok(AirportData {
                airport_id: row.get(0)?,
                icao: row.get(1)?,
                name: row.get(2)?,
                city: row.get(3).unwrap_or_default(),
                country: row.get(4).unwrap_or_default(),
                latitude: row.get(5)?,
                longitude: row.get(6)?,
                elevation: row.get(7)?,
                tower_frequency: row.get(8).ok(),
                atis_frequency: row.get(9).ok(),
                awos_frequency: row.get(10).ok(),
                asos_frequency: row.get(11).ok(),
                unicom_frequency: row.get(12).ok(),
                runways: Vec::new(),
                parking: Vec::new(),
                sids: Vec::new(),
                stars: Vec::new(),
            })
        })?;
        
        Ok(airport)
    }
    
    /// ��取跑道信息
    pub fn get_runways(&self, airport_id: i32) -> Result<Vec<RunwayData>> {
        let mut stmt = self.conn.prepare(
            "SELECT primary_name, secondary_name, heading, length, width, surface
             FROM runway WHERE airport_id = ?1"
        )?;
        
        let runways = stmt.query_map([airport_id], |row| {
            Ok(RunwayData {
                primary_name: row.get(0)?,
                secondary_name: row.get(1)?,
                heading: row.get(2)?,
                length: row.get(3)?,
                width: row.get(4)?,
                surface: row.get(5).unwrap_or_default(),
            })
        })?;
        
        Ok(runways.collect::<Result<Vec<_>>>()?)
    }
    
    /// 获取停机位信息
    pub fn get_parking(&self, airport_id: i32) -> Result<Vec<ParkingData>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, type, airline_codes, number, radius, heading, lat, lonx
             FROM parking WHERE airport_id = ?1 ORDER BY name"
        )?;
        
        let parking = stmt.query_map([airport_id], |row| {
            Ok(ParkingData {
                name: row.get(0)?,
                parking_type: row.get(1)?,
                airline_codes: row.get(2).unwrap_or_default(),
                number: row.get(3).unwrap_or_default(),
                radius: row.get(4).unwrap_or(0.0),
                heading: row.get(5).unwrap_or(0.0),
                latitude: row.get(6)?,
                longitude: row.get(7)?,
            })
        })?;
        
        Ok(parking.collect::<Result<Vec<_>>>()?)
    }
    
    /// 获取 SID 程序
    pub fn get_sids(&self, airport_id: i32) -> Result<Vec<ProcedureData>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT fix_ident, runway_name 
             FROM transition 
             WHERE airport_id = ?1 AND type = 'SID'
             ORDER BY fix_ident"
        )?;
        
        let sids = stmt.query_map([airport_id], |row| {
            Ok(ProcedureData {
                name: row.get(0)?,
                runway: row.get(1).unwrap_or_default(),
                procedure_type: "SID".to_string(),
            })
        })?;
        
        Ok(sids.collect::<Result<Vec<_>>>()?)
    }
    
    /// 获取 STAR 程序
    pub fn get_stars(&self, airport_id: i32) -> Result<Vec<ProcedureData>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT fix_ident, runway_name 
             FROM transition 
             WHERE airport_id = ?1 AND type = 'STAR'
             ORDER BY fix_ident"
        )?;
        
        let stars = stmt.query_map([airport_id], |row| {
            Ok(ProcedureData {
                name: row.get(0)?,
                runway: row.get(1).unwrap_or_default(),
                procedure_type: "STAR".to_string(),
            })
        })?;
        
        Ok(stars.collect::<Result<Vec<_>>>()?)
    }
    
    /// 获取机场完整数据（包含所有子数据）
    pub fn get_airport_full(&self, icao: &str) -> Result<AirportData> {
        let mut airport = self.get_airport(icao)?;
        
        airport.runways = self.get_runways(airport.airport_id).unwrap_or_default();
        airport.parking = self.get_parking(airport.airport_id).unwrap_or_default();
        airport.sids = self.get_sids(airport.airport_id).unwrap_or_default();
        airport.stars = self.get_stars(airport.airport_id).unwrap_or_default();
        
        Ok(airport)
    }
    
    /// 格式化为 LLM 上下文
    pub fn format_for_llm(&self, airport: &AirportData, language: &str) -> String {
        if language == "zh" {
            format!(
                r#"## 机场信息：{} ({})

**基本信息：**
- ICAO 代码：{}
- 城市：{}
- 国家：{}
- 海拔：{} 英尺
- 坐标：{:.4}°N, {:.4}°E

**跑道信息：**
{}

**停机位：**
{}

**通信频率：**
{}

**标准离场程序（SID）：**
{}

**标准进场程序（STAR）：**
{}

**重要提示：**
- 必须使用正确的跑道编号
- 滑行指令必须包含完整的滑行道序列
- 停机位指令必须使用真实存在的停机位编号
- 必须在适当时机提供频率切换
"#,
                airport.name,
                airport.icao,
                airport.icao,
                airport.city,
                airport.country,
                airport.elevation,
                airport.latitude,
                airport.longitude,
                Self::format_runways(&airport.runways, language),
                Self::format_parking(&airport.parking, language),
                Self::format_frequencies(airport, language),
                Self::format_procedures(&airport.sids, language),
                Self::format_procedures(&airport.stars, language),
            )
        } else {
            format!(
                r#"## Airport Information: {} ({})

**Basic Info:**
- ICAO Code: {}
- City: {}
- Country: {}
- Elevation: {} ft
- Coordinates: {:.4}°N, {:.4}°E

**Runways:**
{}

**Parking:**
{}

**Frequencies:**
{}

**SID (Standard Instrument Departure):**
{}

**STAR (Standard Terminal Arrival Route):**
{}

**Important Notes:**
- MUST use correct runway number
- Taxi instructions MUST include complete taxiway sequence
- Gate assignments MUST use real gate numbers
- MUST provide frequency change at appropriate time
"#,
                airport.name,
                airport.icao,
                airport.icao,
                airport.city,
                airport.country,
                airport.elevation,
                airport.latitude,
                airport.longitude,
                Self::format_runways(&airport.runways, language),
                Self::format_parking(&airport.parking, language),
                Self::format_frequencies(airport, language),
                Self::format_procedures(&airport.sids, language),
                Self::format_procedures(&airport.stars, language),
            )
        }
    }
    
    fn format_runways(runways: &[RunwayData], language: &str) -> String {
        if runways.is_empty() {
            return if language == "zh" { "（暂无数据）".to_string() } else { "(No data)".to_string() };
        }
        
        runways.iter().map(|r| {
            if language == "zh" {
                format!("- 跑道 {}/{}：长 {} 米，宽 {} 米，航向 {}°，表面：{}", 
                    r.primary_name, r.secondary_name, r.length, r.width, r.heading, r.surface)
            } else {
                format!("- Runway {}/{}: {} m x {} m, heading {}°, surface: {}", 
                    r.primary_name, r.secondary_name, r.length, r.width, r.heading, r.surface)
            }
        }).collect::<Vec<_>>().join("\n")
    }
    
    fn format_parking(parking: &[ParkingData], language: &str) -> String {
        if parking.is_empty() {
            return if language == "zh" { "（暂无数据）".to_string() } else { "(No data)".to_string() };
        }
        
        let count = parking.len();
        let sample: Vec<_> = parking.iter().take(20).collect();
        
        let list = sample.iter().map(|p| {
            if language == "zh" {
                format!("- {} ({})", p.name, p.parking_type)
            } else {
                format!("- {} ({})", p.name, p.parking_type)
            }
        }).collect::<Vec<_>>().join("\n");
        
        if count > 20 {
            if language == "zh" {
                format!("{}\n... 共 {} 个停机位", list, count)
            } else {
                format!("{}\n... Total {} parking positions", list, count)
            }
        } else {
            list
        }
    }
    
    fn format_frequencies(airport: &AirportData, language: &str) -> String {
        let mut freqs = Vec::new();
        
        if let Some(f) = airport.tower_frequency {
            freqs.push(if language == "zh" {
                format!("- 塔台：{:.3} MHz", f / 1000.0)
            } else {
                format!("- Tower: {:.3} MHz", f / 1000.0)
            });
        }
        
        if let Some(f) = airport.atis_frequency {
            freqs.push(if language == "zh" {
                format!("- ATIS：{:.3} MHz", f / 1000.0)
            } else {
                format!("- ATIS: {:.3} MHz", f / 1000.0)
            });
        }
        
        if let Some(f) = airport.unicom_frequency {
            freqs.push(if language == "zh" {
                format!("- UNICOM：{:.3} MHz", f / 1000.0)
            } else {
                format!("- UNICOM: {:.3} MHz", f / 1000.0)
            });
        }
        
        if freqs.is_empty() {
            if language == "zh" { "（暂无数据）".to_string() } else { "(No data)".to_string() }
        } else {
            freqs.join("\n")
        }
    }
    
    fn format_procedures(procedures: &[ProcedureData], language: &str) -> String {
        if procedures.is_empty() {
            return if language == "zh" { "（暂无数据）".to_string() } else { "(No data)".to_string() };
        }
        
        procedures.iter().map(|p| {
            if p.runway.is_empty() {
                p.name.clone()
            } else {
                format!("{} (RWY {})", p.name, p.runway)
            }
        }).collect::<Vec<_>>().join(", ")
    }
}

/// 机场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirportData {
    pub airport_id: i32,
    pub icao: String,
    pub name: String,
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: i32,
    pub tower_frequency: Option<i32>,
    pub atis_frequency: Option<i32>,
    pub awos_frequency: Option<i32>,
    pub asos_frequency: Option<i32>,
    pub unicom_frequency: Option<i32>,
    pub runways: Vec<RunwayData>,
    pub parking: Vec<ParkingData>,
    pub sids: Vec<ProcedureData>,
    pub stars: Vec<ProcedureData>,
}

/// 跑道数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunwayData {
    pub primary_name: String,
    pub secondary_name: String,
    pub heading: f64,
    pub length: i32,
    pub width: i32,
    pub surface: String,
}

/// 停机位数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingData {
    pub name: String,
    pub parking_type: String,
    pub airline_codes: String,
    pub number: i32,
    pub radius: f64,
    pub heading: f64,
    pub latitude: f64,
    pub longitude: f64,
}

/// 程序数据（SID/STAR）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureData {
    pub name: String,
    pub runway: String,
    pub procedure_type: String,
}
