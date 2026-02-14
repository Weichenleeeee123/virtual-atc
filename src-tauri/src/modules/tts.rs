use std::error::Error;
use std::io::Cursor;
use rodio::{Decoder, OutputStream, Sink};

pub struct TTSEngine {
    api_key: String,
    api_url: String,
}

impl TTSEngine {
    pub fn new() -> Self {
        let api_key = std::env::var("SILICONFLOW_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        TTSEngine {
            api_key,
            // 使用 edge-tts 的免费 API（微软 Azure TTS）
            api_url: "https://api.siliconflow.cn/v1/audio/speech".to_string(),
        }
    }
    
    /// 将文本转换为语音并播放
    pub async fn speak(&self, text: &str, language: &str) -> Result<(), Box<dyn Error>> {
        if text.is_empty() {
            return Ok(());
        }
        
        println!("TTS: {}", text);
        
        // 选择合适的语音
        let voice = if language == "zh" {
            "zh-CN-XiaoxiaoNeural"  // 中文女声（温柔、专业）
        } else {
            "en-US-JennyNeural"     // 英文女声（清晰、专业）
        };
        
        // 调用 TTS API
        let audio_data = self.generate_speech(text, voice).await?;
        
        // 播放音频
        self.play_audio(&audio_data)?;
        
        Ok(())
    }
    
    /// 调用 TTS API 生成语音
    async fn generate_speech(&self, text: &str, voice: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let client = reqwest::Client::new();
        
        let response = client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "fishaudio/fish-speech-1.4",
                "input": text,
                "voice": voice,
                "response_format": "mp3",
                "speed": 1.0
            }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("TTS API error: {}", response.status()).into());
        }
        
        let audio_bytes = response.bytes().await?;
        Ok(audio_bytes.to_vec())
    }
    
    /// 播放音频数据
    fn play_audio(&self, audio_data: &[u8]) -> Result<(), Box<dyn Error>> {
        // 创建音频输出流
        let (_stream, stream_handle) = OutputStream::try_default()?;
        
        // 创建音频播放器
        let sink = Sink::try_new(&stream_handle)?;
        
        // 解码音频数据
        let cursor = Cursor::new(audio_data.to_vec());
        let source = Decoder::new(cursor)?;
        
        // 播放音频
        sink.append(source);
        sink.sleep_until_end();
        
        Ok(())
    }
    
    /// 保存音频到文件（用于调试）
    pub async fn save_to_file(&self, text: &str, language: &str, path: &str) -> Result<(), Box<dyn Error>> {
        let voice = if language == "zh" {
            "zh-CN-XiaoxiaoNeural"
        } else {
            "en-US-JennyNeural"
        };
        
        let audio_data = self.generate_speech(text, voice).await?;
        std::fs::write(path, audio_data)?;
        
        println!("Audio saved to: {}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tts_chinese() {
        let tts = TTSEngine::new();
        let result = tts.speak("国航123，北京塔台，可以起飞", "zh").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_tts_english() {
        let tts = TTSEngine::new();
        let result = tts.speak("Air China 123, Beijing Tower, cleared for takeoff", "en").await;
        assert!(result.is_ok());
    }
}
