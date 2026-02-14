use std::error::Error;
use serde_json::json;

pub struct LLMClient {
    api_key: String,
    api_url: String,
}

impl LLMClient {
    pub fn new() -> Self {
        let api_key = std::env::var("SILICONFLOW_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        LLMClient {
            api_key,
            api_url: "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        }
    }
    
    pub async fn get_atc_response(
        &self,
        message: &str,
        language: &str,
        flight_data: Option<super::simulator::FlightData>,
    ) -> Result<String, Box<dyn Error>> {
        let system_prompt = self.build_system_prompt(language, flight_data);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "Qwen/Qwen2.5-7B-Instruct",
                "messages": [
                    {
                        "role": "system",
                        "content": system_prompt
                    },
                    {
                        "role": "user",
                        "content": message
                    }
                ],
                "temperature": 0.3,
                "max_tokens": 150
            }))
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        
        let atc_response = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("收到")
            .to_string();
        
        Ok(atc_response)
    }
    
    fn build_system_prompt(
        &self,
        language: &str,
        flight_data: Option<super::simulator::FlightData>,
    ) -> String {
        // 读取标准用语文档
        let base_prompt = if language == "zh" {
            include_str!("../../../docs/atc-phraseology/chinese.md")
        } else {
            include_str!("../../../docs/atc-phraseology/english.md")
        };
        
        if let Some(data) = flight_data {
            let context = if language == "zh" {
                format!(
                    "\n\n## 当前飞行数据\n\n- 呼号：{}\n- 高度：{:.0} 米\n- 速度：{:.0} 节\n- 航向：{:.0}°\n\n根据以上飞行数据和飞行员的请求，给出一条符合标准的管制指令。",
                    data.callsign, 
                    data.altitude * 0.3048,  // 英尺转米
                    data.speed, 
                    data.heading
                )
            } else {
                format!(
                    "\n\n## Current Flight Data\n\n- Callsign: {}\n- Altitude: {:.0} feet\n- Speed: {:.0} knots\n- Heading: {:.0}°\n\nBased on the flight data and pilot request, provide one standard ATC instruction.",
                    data.callsign, data.altitude, data.speed, data.heading
                )
            };
            format!("{}{}", base_prompt, context)
        } else {
            base_prompt.to_string()
        }
    }
}
