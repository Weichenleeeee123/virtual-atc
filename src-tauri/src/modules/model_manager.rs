use std::path::PathBuf;
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub name: String,
    pub size: String,
    pub url: String,
    pub filename: String,
    pub description: String,
}

pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    pub fn new() -> Self {
        let models_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("VirtualATC")
            .join("models");
        
        // 创建模型目录
        fs::create_dir_all(&models_dir).ok();
        
        ModelManager { models_dir }
    }
    
    /// 获取可用的 Whisper 模型列表
    pub fn get_available_models() -> Vec<WhisperModel> {
        vec![
            WhisperModel {
                name: "tiny".to_string(),
                size: "75 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".to_string(),
                filename: "ggml-tiny.bin".to_string(),
                description: "最快，适合测试（英文准确率较低）".to_string(),
            },
            WhisperModel {
                name: "base".to_string(),
                size: "142 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".to_string(),
                filename: "ggml-base.bin".to_string(),
                description: "快速，基础识别".to_string(),
            },
            WhisperModel {
                name: "small".to_string(),
                size: "466 MB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".to_string(),
                filename: "ggml-small.bin".to_string(),
                description: "平衡性能和准确率".to_string(),
            },
            WhisperModel {
                name: "medium".to_string(),
                size: "1.5 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".to_string(),
                filename: "ggml-medium.bin".to_string(),
                description: "推荐，高准确率（中英文效果好）".to_string(),
            },
            WhisperModel {
                name: "large-v3".to_string(),
                size: "3.1 GB".to_string(),
                url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".to_string(),
                filename: "ggml-large-v3.bin".to_string(),
                description: "最高准确率，需要强大硬件".to_string(),
            },
        ]
    }
    
    /// 获取���下载的模型列表
    pub fn get_downloaded_models(&self) -> Vec<String> {
        let mut models = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename.starts_with("ggml-") && filename.ends_with(".bin") {
                        models.push(filename.to_string());
                    }
                }
            }
        }
        
        models
    }
    
    /// 检查模型是否已下载
    pub fn is_model_downloaded(&self, filename: &str) -> bool {
        self.models_dir.join(filename).exists()
    }
    
    /// 获取模型文件路径
    pub fn get_model_path(&self, filename: &str) -> PathBuf {
        self.models_dir.join(filename)
    }
    
    /// 下载模型（带进度回调）
    pub async fn download_model<F>(
        &self,
        model: &WhisperModel,
        progress_callback: F,
    ) -> Result<PathBuf, Box<dyn std::error::Error>>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        let target_path = self.models_dir.join(&model.filename);
        
        // 如果已存在，先删除
        if target_path.exists() {
            fs::remove_file(&target_path)?;
        }
        
        // 下载文件
        let client = reqwest::Client::new();
        let response = client.get(&model.url).send().await?;
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;
        let mut file = fs::File::create(&target_path)?;
        
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        
        Ok(target_path)
    }
    
    /// 删除模型
    pub fn delete_model(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.models_dir.join(filename);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
    
    /// 获取模型目录路径
    pub fn get_models_dir(&self) -> &PathBuf {
        &self.models_dir
    }
}
