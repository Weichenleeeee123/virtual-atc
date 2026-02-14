// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;

use modules::simulator::SimulatorConnection;
use modules::whisper::WhisperEngine;
use modules::llm::LLMClient;
use modules::tts::TTSEngine;
use modules::msfs::MSFSConnection;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    simulator: Mutex<Option<SimulatorConnection>>,
    msfs: Mutex<Option<MSFSConnection>>,
    whisper: Mutex<Option<WhisperEngine>>,
    llm: Mutex<LLMClient>,
    tts: Mutex<TTSEngine>,
    current_sim: Mutex<String>, // "xplane" or "msfs"
}

#[tauri::command]
async fn connect_simulator(sim_type: String, state: State<'_, AppState>) -> Result<String, String> {
    let mut current_sim = state.current_sim.lock().unwrap();
    
    match sim_type.as_str() {
        "xplane" => {
            let mut sim = state.simulator.lock().unwrap();
            *sim = Some(SimulatorConnection::new().map_err(|e| e.to_string())?);
            *current_sim = "xplane".to_string();
            Ok("Connected to X-Plane".to_string())
        }
        "msfs" => {
            let mut msfs = state.msfs.lock().unwrap();
            let mut connection = MSFSConnection::new().map_err(|e| e.to_string())?;
            connection.connect().map_err(|e| e.to_string())?;
            *msfs = Some(connection);
            *current_sim = "msfs".to_string();
            Ok("Connected to MSFS".to_string())
        }
        _ => Err("Invalid simulator type. Use 'xplane' or 'msfs'".to_string())
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
async fn get_flight_data(state: State<'_, AppState>) -> Result<FlightData, String> {
    let current_sim = state.current_sim.lock().unwrap();
    
    match current_sim.as_str() {
        "xplane" => {
            let sim = state.simulator.lock().unwrap();
            match &*sim {
                Some(connection) => {
                    let data = connection.get_flight_data().map_err(|e| e.to_string())?;
                    Ok(FlightData {
                        callsign: data.callsign,
                        altitude: data.altitude,
                        speed: data.speed,
                        heading: data.heading,
                        vertical_speed: data.vertical_speed,
                        latitude: data.latitude,
                        longitude: data.longitude,
                    })
                }
                None => Err("Not connected to X-Plane".to_string()),
            }
        }
        "msfs" => {
            let msfs = state.msfs.lock().unwrap();
            match &*msfs {
                Some(connection) => {
                    let data = connection.get_flight_data().map_err(|e| e.to_string())?;
                    Ok(FlightData {
                        callsign: data.callsign,
                        altitude: data.altitude,
                        speed: data.speed,
                        heading: data.heading,
                        vertical_speed: data.vertical_speed,
                        latitude: data.latitude,
                        longitude: data.longitude,
                    })
                }
                None => Err("Not connected to MSFS".to_string()),
            }
        }
        _ => Err("No simulator connected".to_string())
    }
}

#[tauri::command]
async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    let mut whisper = state.whisper.lock().unwrap();
    if whisper.is_none() {
        let mut engine = WhisperEngine::new().map_err(|e| e.to_string())?;
        
        // 从环境变量读取模型路径
        let model_path = std::env::var("WHISPER_MODEL_PATH")
            .unwrap_or_else(|_| "./models/ggml-medium.bin".to_string());
        
        engine.load_model(&model_path).map_err(|e| e.to_string())?;
        *whisper = Some(engine);
    }
    
    if let Some(engine) = &mut *whisper {
        engine.start_recording().map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

#[tauri::command]
async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    let mut whisper = state.whisper.lock().unwrap();
    
    if let Some(engine) = &mut *whisper {
        let audio_data = engine.stop_recording().map_err(|e| e.to_string())?;
        let transcript = engine.transcribe(&audio_data).map_err(|e| e.to_string())?;
        Ok(transcript)
    } else {
        Err("Whisper engine not initialized".to_string())
    }
}

#[tauri::command]
async fn get_atc_response(
    message: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let llm = state.llm.lock().unwrap();
    
    // Get current flight data for context
    let sim = state.simulator.lock().unwrap();
    let flight_data = match &*sim {
        Some(connection) => connection.get_flight_data().ok(),
        None => None,
    };
    
    let response = llm.get_atc_response(&message, &language, flight_data)
        .await
        .map_err(|e| e.to_string())?;
    
    // 播放 TTS 语音
    let tts = state.tts.lock().unwrap();
    if let Err(e) = tts.speak(&response, &language).await {
        eprintln!("TTS error: {}", e);
        // TTS 失败不影响返回结果
    }
    
    Ok(response)
}

#[derive(serde::Serialize)]
struct FlightData {
    callsign: String,
    altitude: f64,
    speed: f64,
    heading: f64,
    vertical_speed: f64,
    latitude: f64,
    longitude: f64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let llm_client = LLMClient::new();
    let tts_engine = TTSEngine::new();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            simulator: Mutex::new(None),
            msfs: Mutex::new(None),
            whisper: Mutex::new(None),
            llm: Mutex::new(llm_client),
            tts: Mutex::new(tts_engine),
            current_sim: Mutex::new(String::new()),
        })
        .invoke_handler(tauri::generate_handler![
            connect_simulator,
            disconnect_simulator,
            get_flight_data,
            start_recording,
            stop_recording,
            get_atc_response,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
