// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;

use modules::simulator::SimulatorConnection;
use modules::whisper::WhisperEngine;
use modules::llm::LLMClient;
use modules::tts::TTSEngine;
use modules::msfs::MSFSConnection;
use modules::flight_phase::{FlightPhaseDetector, FlightPhase};
use modules::atc_database::ATCDatabase;
use modules::little_navmap::LittleNavmapDB;
use modules::model_manager::{ModelManager, WhisperModel};
use std::sync::Mutex;
use tauri::State;

struct AppState {
    simulator: Mutex<Option<SimulatorConnection>>,
    msfs: Mutex<Option<MSFSConnection>>,
    whisper: Mutex<Option<WhisperEngine>>,
    llm: Mutex<LLMClient>,
    tts: Mutex<TTSEngine>,
    current_sim: Mutex<String>, // "xplane" or "msfs"
    phase_detector: Mutex<FlightPhaseDetector>,
    atc_database: Mutex<ATCDatabase>,
    little_navmap: Mutex<Option<LittleNavmapDB>>,
    model_manager: Mutex<ModelManager>,
}

#[tauri::command]
async fn connect_simulator(sim_type: String, state: State<'_, AppState>) -> Result<String, String> {
    let mut current_sim = state.current_sim.lock().unwrap();
    
    match sim_type.as_str() {
        "xplane" => {
            let mut sim = state.simulator.lock().unwrap();
            match SimulatorConnection::new() {
                Ok(connection) => {
                    *sim = Some(connection);
                    *current_sim = "xplane".to_string();
                    Ok("✓ 已连接到 X-Plane\n正在接收飞行数据...".to_string())
                }
                Err(e) => {
                    Err(format!("❌ 无法连接到 X-Plane\n\n可能的原因：\n• X-Plane 未运行\n• 飞机未加载\n• 防火墙阻止 UDP 端口 49000\n\n详细错误：{}", e))
                }
            }
        }
        "msfs" => {
            let mut msfs = state.msfs.lock().unwrap();
            match MSFSConnection::new() {
                Ok(mut connection) => {
                    match connection.connect() {
                        Ok(_) => {
                            *msfs = Some(connection);
                            *current_sim = "msfs".to_string();
                            Ok("✓ 已连接到 MSFS\n正在接收飞行数据...".to_string())
                        }
                        Err(e) => {
                            Err(format!("❌ 无法连接到 MSFS\n\n可能的原因：\n• MSFS 未运行\n• SimConnect 未安装\n• Python 桥接脚本启动失败\n\n详细错误：{}", e))
                        }
                    }
                }
                Err(e) => {
                    Err(format!("❌ 初始化 MSFS 连接失败\n\n请确保：\n• 已安装 Python\n• 已安装 SimConnect-Python (pip install SimConnect-Python)\n\n详细错误：{}", e))
                }
            }
        }
        _ => Err("❌ 无效的模拟器类型\n\n请使用 'xplane' 或 'msfs'".to_string())
    }
}

#[tauri::command]
async fn disconnect_simulator(state: State<'_, AppState>) -> Result<String, String> {
    let mut current_sim = state.current_sim.lock().unwrap();
    
    match current_sim.as_str() {
        "xplane" => {
            let mut sim = state.simulator.lock().unwrap();
            *sim = None;
        }
        "msfs" => {
            let mut msfs = state.msfs.lock().unwrap();
            if let Some(connection) = msfs.as_mut() {
                connection.disconnect();
            }
            *msfs = None;
        }
        _ => {}
    }
    
    *current_sim = String::new();
    Ok("Disconnected".to_string())
}

#[tauri::command]
async fn get_flight_data(state: State<'_, AppState>) -> Result<FlightDataResponse, String> {
    let current_sim = state.current_sim.lock().unwrap();
    
    // 获取飞行数据
    let (callsign, altitude, speed, heading, vertical_speed, latitude, longitude, on_ground) = match current_sim.as_str() {
        "xplane" => {
            let sim = state.simulator.lock().unwrap();
            match &*sim {
                Some(connection) => {
                    let data = connection.get_flight_data().map_err(|e| e.to_string())?;
                    (
                        data.callsign,
                        data.altitude,
                        data.speed,
                        data.heading,
                        data.vertical_speed,
                        data.latitude,
                        data.longitude,
                        data.altitude < 10.0 && data.vertical_speed.abs() < 100.0,
                    )
                }
                None => return Err("Not connected to X-Plane".to_string()),
            }
        }
        "msfs" => {
            let msfs = state.msfs.lock().unwrap();
            match &*msfs {
                Some(connection) => {
                    let data = connection.get_flight_data().map_err(|e| e.to_string())?;
                    (
                        data.callsign,
                        data.altitude,
                        data.speed,
                        data.heading,
                        data.vertical_speed,
                        data.latitude,
                        data.longitude,
                        data.on_ground,
                    )
                }
                None => return Err("Not connected to MSFS".to_string()),
            }
        }
        _ => return Err("No simulator connected".to_string())
    };
    
    // 更新飞行阶段
    let mut detector = state.phase_detector.lock().unwrap();
    let phase = detector.update(&modules::flight_phase::FlightData {
        altitude,
        speed,
        heading,
        vertical_speed,
        on_ground,
    });
    
    Ok(FlightDataResponse {
        callsign,
        altitude,
        speed,
        heading,
        vertical_speed,
        latitude,
        longitude,
        phase: phase.as_str().to_string(),
        phase_display: phase.display_name().to_string(),
    })
}

#[tauri::command]
async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut whisper = state.whisper.lock().unwrap();
    if whisper.is_none() {
        let mut engine = WhisperEngine::new().map_err(|e| {
            format!("❌ 初始化 Whisper 引擎失败\n\n详细错误：{}", e)
        })?;
        
        // 从环境变量读取模型路径
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| "./models/ggml-medium.bin".to_string());
        
        engine.load_model(&model_path).map_err(|e| {
            format!("❌ 无法加载 Whisper 模型\n\n可能的原因：\n• 模型文件不存在：{}\n• 模型文件损坏\n• 权限不足\n\n解决方法：\n1. 点击「模型管理」标签\n2. 下载 medium 模型（推荐）\n3. 或手动下载到 models/ 目录\n\n详细错误：{}", model_path, e)
        })?;
        *whisper = Some(engine);
    }
    
    if let Some(engine) = &mut *whisper {
        engine.start_recording().map_err(|e| {
            format!("❌ 无法启动录音\n\n可能的原因：\n• 未检测到麦克风\n• 麦克风被其他程序占用\n• 权限不足（需要麦克风权限）\n\n详细错误：{}", e)
        })?;
    }
    
    Ok(())
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    let mut whisper = state.whisper.lock().unwrap();
    
    if let Some(engine) = &mut *whisper {
        let audio_data = engine.stop_recording().map_err(|e| {
            format!("❌ 停止录音失败\n\n详细错误：{}", e)
        })?;
        
        if audio_data.is_empty() {
            return Err("⚠️ 未检测到音频输入\n\n请确保：\n• 麦克风已连接\n• 麦克风未静音\n• 说话时按住 PTT 按钮".to_string());
        }
        
        let transcript = engine.transcribe(&audio_data).map_err(|e| {
            format!("❌ 语音识别失败\n\n可能的原因：\n• 音频质量太差\n• 背景噪音过大\n• 模型不支持该语言\n\n详细错误：{}", e)
        })?;
        
        if transcript.trim().is_empty() {
            return Err("⚠️ 未识别到语音内容\n\n建议：\n• 在安静环境中使用\n• 说话清晰、语速适中\n• 靠近麦克风".to_string());
        }
        
        Ok(transcript)
    } else {
        Err("❌ Whisper 引擎未初始化\n\n请重启应用或重新加载模型".to_string())
    }
}

#[tauri::command]
async fn get_atc_response(
    message: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let llm = state.llm.lock().unwrap();
    
    // 获取当前飞行数据
    let current_sim = state.current_sim.lock().unwrap();
    let flight_data = match current_sim.as_str() {
        "xplane" => {
            let sim = state.simulator.lock().unwrap();
            sim.as_ref().and_then(|c| c.get_flight_data().ok())
        }
        "msfs" => {
            let msfs = state.msfs.lock().unwrap();
            msfs.as_ref().and_then(|c| c.get_flight_data().ok()).map(|data| {
                modules::simulator::FlightData {
                    callsign: data.callsign,
                    altitude: data.altitude,
                    speed: data.speed,
                    heading: data.heading,
                    vertical_speed: data.vertical_speed,
                    latitude: data.latitude,
                    longitude: data.longitude,
                }
            })
        }
        _ => None,
    };
    
    // 自动检测机场（如果��飞行数据）
    if let Some(ref data) = flight_data {
        let mut atc_db = state.atc_database.lock().unwrap();
        atc_db.detect_nearest_airport(data.latitude, data.longitude);
    }
    
    // 获取机场上下文
    let atc_db = state.atc_database.lock().unwrap();
    let airport_context = atc_db.get_atc_context(&language);
    
    // 获取飞行阶段上下文
    let detector = state.phase_detector.lock().unwrap();
    let phase_context = detector.get_atc_context(&language);
    
    // 构建完整的上下文
    let full_context = format!(
        "{}\n\n当前飞行阶段：{}\n\n{}\n\n飞行员消息：{}",
        airport_context,
        detector.get_current_phase().display_name(),
        phase_context,
        message
    );
    
    let response = llm.get_atc_response(&full_context, &language, flight_data)
        .await
        .map_err(|e| e.to_string())?;
    
    // 播放 TTS 语音
    let tts = state.tts.lock().unwrap();
    if let Err(e) = tts.speak(&response, &language).await {
        eprintln!("TTS error: {}", e);
    }
    
    Ok(response)
}

#[tauri::command]
async fn get_current_phase(state: State<'_, AppState>) -> Result<PhaseInfo, String> {
    let detector = state.phase_detector.lock().unwrap();
    let phase = detector.get_current_phase();
    let duration = detector.get_phase_duration();
    
    Ok(PhaseInfo {
        phase: phase.as_str().to_string(),
        display_name: phase.display_name().to_string(),
        duration_seconds: duration.as_secs(),
    })
}

// ========== 模型管理命令 ==========

#[tauri::command]
fn get_available_models() -> Vec<WhisperModel> {
    ModelManager::get_available_models()
}

#[tauri::command]
fn get_downloaded_models(state: State<'_, AppState>) -> Vec<String> {
    let manager = state.model_manager.lock().unwrap();
    manager.get_downloaded_models()
}

#[tauri::command]
async fn download_model(
    model_name: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<String, String> {
    let manager = state.model_manager.lock().unwrap();
    let models = ModelManager::get_available_models();
    
    let model = models.iter()
        .find(|m| m.name == model_name)
        .ok_or("Model not found")?
        .clone();
    
    drop(manager); // 释放锁
    
    // 下载模型，带进度回调
    let manager = state.model_manager.lock().unwrap();
    let result = manager.download_model(&model, move |downloaded, total| {
        let progress = if total > 0 {
            (downloaded as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };
        
        // 发送进度事件到前端
        let _ = window.emit("download-progress", serde_json::json!({
            "model": model.name,
            "downloaded": downloaded,
            "total": total,
            "progress": progress,
        }));
    }).await;
    
    match result {
        Ok(path) => Ok(format!("Model downloaded to: {:?}", path)),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn delete_model(filename: String, state: State<'_, AppState>) -> Result<String, String> {
    let manager = state.model_manager.lock().unwrap();
    manager.delete_model(&filename).map_err(|e| e.to_string())?;
    Ok(format!("Model {} deleted", filename))
}

#[tauri::command]
fn load_model(filename: String, state: State<'_, AppState>) -> Result<String, String> {
    let manager = state.model_manager.lock().unwrap();
    let model_path = manager.get_model_path(&filename);
    
    if !model_path.exists() {
        return Err(format!("Model file not found: {:?}", model_path));
    }
    
    let mut whisper = state.whisper.lock().unwrap();
    
    // 创建或更新 Whisper 引擎
    if whisper.is_none() {
        *whisper = Some(WhisperEngine::new());
    }
    
    if let Some(engine) = whisper.as_mut() {
        engine.load_model(model_path.to_str().unwrap())
            .map_err(|e| e.to_string())?;
        Ok(format!("Model {} loaded successfully", filename))
    } else {
        Err("Failed to initialize Whisper engine".to_string())
    }
}

#[derive(serde::Serialize)]
struct PhaseInfo {
    phase: String,
    display_name: String,
    duration_seconds: u64,
}

#[derive(serde::Serialize)]
struct FlightDataResponse {
    callsign: String,
    altitude: f64,
    speed: f64,
    heading: f64,
    vertical_speed: f64,
    latitude: f64,
    longitude: f64,
    phase: String,
    phase_display: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let llm_client = LLMClient::new();
    let tts_engine = TTSEngine::new();
    let atc_db = ATCDatabase::new();
    let model_manager = ModelManager::new();
    
    // 尝试加载 Little Navmap 数据库
    let little_navmap = match LittleNavmapDB::new() {
        Ok(db) => {
            println!("✓ Little Navmap 数据库加载成功");
            Some(db)
        }
        Err(e) => {
            println!("⚠ Little Navmap 数据库未找到: {}", e);
            println!("  提示：安装 Little Navmap 以获取完整的机场数据");
            None
        }
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            simulator: Mutex::new(None),
            msfs: Mutex::new(None),
            whisper: Mutex::new(None),
            llm: Mutex::new(llm_client),
            tts: Mutex::new(tts_engine),
            current_sim: Mutex::new(String::new()),
            phase_detector: Mutex::new(FlightPhaseDetector::new()),
            atc_database: Mutex::new(atc_db),
            little_navmap: Mutex::new(little_navmap),
            model_manager: Mutex::new(model_manager),
        })
        .invoke_handler(tauri::generate_handler![
            connect_simulator,
            disconnect_simulator,
            get_flight_data,
            start_recording,
            stop_recording,
            get_atc_response,
            get_current_phase,
            get_available_models,
            get_downloaded_models,
            download_model,
            delete_model,
            load_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
