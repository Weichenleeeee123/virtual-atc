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
                "temperature": 0.7,
                "max_tokens": 200
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
        let base_prompt = if language == "zh" {
            r#"你是一名专业的中国民航空中交通管制员（ATC）。

职责：
- 使用标准的中国民航陆空通话用语
- 根据飞行员的请求提供清晰、简洁的指令
- 确保飞行安全，维护空域秩序
- 保持专业、冷静的语气

标准用语示例：
- 起飞许可："XX航空XXX，跑道XX，可以起飞，地面风XX度XX米"
- 高度指令："XX航空XXX，可以下降至XXXX米"
- 航向指令："XX航空XXX，左转航向XXX"
- 频率切换："XX航空XXX，联系XX进近XXX.X"

注意事项：
- 始终使用航空公司呼号（如"国航123"）
- 高度使用米制单位
- 风向使用度数，风速使用米/秒
- 回复要简洁明了，避免冗余信息"#
        } else {
            r#"You are a professional Air Traffic Controller (ATC).

Responsibilities:
- Use standard ICAO phraseology
- Provide clear, concise instructions based on pilot requests
- Ensure flight safety and maintain airspace order
- Maintain a professional, calm tone

Standard phraseology examples:
- Takeoff clearance: "XX Air XXX, runway XX, cleared for takeoff, wind XX at XX"
- Altitude instruction: "XX Air XXX, descend and maintain flight level XXX"
- Heading instruction: "XX Air XXX, turn left heading XXX"
- Frequency change: "XX Air XXX, contact XX Approach XXX.X"

Important notes:
- Always use airline callsign (e.g., "Air China 123")
- Altitude in feet
- Wind direction in degrees, speed in knots
- Keep responses concise and clear"#
        };
        
        if let Some(data) = flight_data {
            format!(
                "{}\n\n当前飞行数据：\n- 呼号：{}\n- 高度：{:.0} 英尺\n- 速度：{:.0} 节\n- 航向：{:.0}°",
                base_prompt, data.callsign, data.altitude, data.speed, data.heading
            )
        } else {
            base_prompt.to_string()
        }
    }
}
