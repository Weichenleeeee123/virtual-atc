use std::error::Error;
use std::sync::{Arc, Mutex};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

pub struct WhisperEngine {
    is_recording: bool,
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<Stream>,
    sample_rate: u32,
}

impl WhisperEngine {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(WhisperEngine {
            is_recording: false,
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            sample_rate: 16000, // Whisper 需要 16kHz
        })
    }
    
    pub fn start_recording(&mut self) -> Result<(), Box<dyn Error>> {
        if self.is_recording {
            return Ok(());
        }
        
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        
        println!("Recording with sample rate: {} Hz", sample_rate);
        
        // 清空之前的录音
        self.samples.lock().unwrap().clear();
        
        let samples = self.samples.clone();
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                self.build_input_stream::<f32>(&device, &config.into(), samples)?
            }
            cpal::SampleFormat::I16 => {
                self.build_input_stream::<i16>(&device, &config.into(), samples)?
            }
            cpal::SampleFormat::U16 => {
                self.build_input_stream::<u16>(&device, &config.into(), samples)?
            }
            _ => return Err("Unsupported sample format".into()),
        };
        
        stream.play()?;
        self.stream = Some(stream);
        self.is_recording = true;
        self.sample_rate = sample_rate;
        
        Ok(())
    }
    
    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        samples: Arc<Mutex<Vec<f32>>>,
    ) -> Result<Stream, Box<dyn Error>>
    where
        T: cpal::Sample,
    {
        let err_fn = |err| eprintln!("Stream error: {}", err);
        
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut samples = samples.lock().unwrap();
                for &sample in data {
                    samples.push(sample.to_f32());
                }
            },
            err_fn,
            None,
        )?;
        
        Ok(stream)
    }
    
    pub fn stop_recording(&mut self) -> Result<Vec<f32>, Box<dyn Error>> {
        if !self.is_recording {
            return Ok(Vec::new());
        }
        
        self.stream = None;
        self.is_recording = false;
        
        let samples = self.samples.lock().unwrap().clone();
        
        // 重采样到 16kHz（如果需要）
        let resampled = if self.sample_rate != 16000 {
            self.resample(&samples, self.sample_rate, 16000)
        } else {
            samples
        };
        
        Ok(resampled)
    }
    
    pub fn transcribe(&self, audio_data: &[f32]) -> Result<String, Box<dyn Error>> {
        // TODO: 集成 whisper.cpp
        // 目前返回模拟数据
        
        if audio_data.is_empty() {
            return Ok(String::new());
        }
        
        // 模拟转录结果
        let duration = audio_data.len() as f32 / 16000.0;
        
        if duration < 0.5 {
            return Ok(String::new());
        }
        
        // 根据音频长度返回不同的模拟文本
        let mock_transcripts = vec![
            "北京塔台，国航123，请求起飞",
            "上海进近，东航456，请求下降",
            "广州塔台，南航789，跑道外，请求落地",
            "成都进近，川航321，高度8000米，请求进近",
            "深圳塔台，深航654，停机位A12，请求开车",
        ];
        
        let index = (duration * 10.0) as usize % mock_transcripts.len();
        Ok(mock_transcripts[index].to_string())
    }
    
    fn resample(&self, audio: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return audio.to_vec();
        }
        
        let ratio = from_rate as f32 / to_rate as f32;
        let output_len = (audio.len() as f32 / ratio) as usize;
        let mut output = Vec::with_capacity(output_len);
        
        for i in 0..output_len {
            let pos = i as f32 * ratio;
            let idx = pos as usize;
            
            if idx < audio.len() {
                // 简单的线性插值
                let frac = pos - idx as f32;
                let sample = if idx + 1 < audio.len() {
                    audio[idx] * (1.0 - frac) + audio[idx + 1] * frac
                } else {
                    audio[idx]
                };
                output.push(sample);
            }
        }
        
        output
    }
    
    pub fn save_to_wav(&self, audio: &[f32], path: &str) -> Result<(), Box<dyn Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 16000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        
        let mut writer = hound::WavWriter::create(path, spec)?;
        
        for &sample in audio {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }
        
        writer.finalize()?;
        Ok(())
    }
}

impl Drop for WhisperEngine {
    fn drop(&mut self) {
        self.stream = None;
    }
}

// 辅助 trait 用于样本格式转换
trait SampleExt {
    fn to_f32(&self) -> f32;
}

impl SampleExt for f32 {
    fn to_f32(&self) -> f32 {
        *self
    }
}

impl SampleExt for i16 {
    fn to_f32(&self) -> f32 {
        *self as f32 / i16::MAX as f32
    }
}

impl SampleExt for u16 {
    fn to_f32(&self) -> f32 {
        (*self as f32 - 32768.0) / 32768.0
    }
}
