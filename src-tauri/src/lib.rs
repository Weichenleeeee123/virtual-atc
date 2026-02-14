// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules;

use modules::simulator::SimulatorConnection;
use modules::whisper::WhisperEngine;
use modules::llm::LLMClient;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    simulator: Mutex<Option<SimulatorConnection>>,
    whisper: Mutex<Option<WhisperEngine>>,
    llm: Mutex<LLMClient>,
}

#[tauri::command]
async fn connect_simulator(state: State<'_, AppState>) -> Result<String, String> {
    let mut sim = state.simulator.lock().unwrap();
    *sim = Some(SimulatorConnection::new().map_err(|e| e.to_string())?);
    Ok("Connected".to_string())
}

#[tauri::command]
async fn get_flight_data(state: State<'_, AppState>) -> Result<FlightData, String> {
    let sim = state.simulator.lock().unwrap();
    match &*sim {
        Some(connection) => connection.get_flight_data().map_err(|e| e.to_string()),
        None => Err("Not connected".to_string()),
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
    
    llm.get_atc_response(&message, &language, flight_data)
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct FlightData {
    callsign: String,
    altitude: f64,
    speed: f64,
    heading: f64,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let llm_client = LLMClient::new();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            simulator: Mutex::new(None),
            whisper: Mutex::new(None),
            llm: Mutex::new(llm_client),
        })
        .invoke_handler(tauri::generate_handler![
            connect_simulator,
            get_flight_data,
            start_recording,
            stop_recording,
            get_atc_response,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
