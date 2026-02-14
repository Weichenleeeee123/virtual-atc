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
            let altitude_m = data.altitude * 0.3048;  // 英尺转米
            let vertical_speed_mpm = data.vertical_speed * 0.3048;  // 英尺/分钟转米/分钟
            
            // 判断飞行阶段（简化版）
            let phase_hint = if data.altitude < 10.0 && data.speed < 5.0 {
                if language == "zh" {
                    "飞机在停机位，可能需要推出许可或滑行指令"
                } else {
                    "Aircraft is parked, may need pushback clearance or taxi instructions"
                }
            } else if data.altitude < 10.0 && data.speed < 40.0 {
                if language == "zh" {
                    "飞机正在滑行，可能需要跑道进入许可或起飞许可"
                } else {
                    "Aircraft is taxiing, may need runway entry or takeoff clearance"
                }
            } else if data.altitude < 10.0 && data.speed >= 40.0 {
                if language == "zh" {
                    "飞机正在起飞滑跑，给予起飞许可"
                } else {
                    "Aircraft is taking off, provide takeoff clearance"
                }
            } else if data.vertical_speed > 500.0 {
                if language == "zh" {
                    "飞机正在爬升，可以给予高度指令或频率切换"
                } else {
                    "Aircraft is climbing, provide altitude instruction or frequency change"
                }
            } else if data.vertical_speed < -500.0 && data.altitude < 3000.0 {
                if language == "zh" {
                    "飞机正在进近，可以给予进近许可或着陆许可"
                } else {
                    "Aircraft is on approach, provide approach or landing clearance"
                }
            } else if data.vertical_speed < -500.0 {
                if language == "zh" {
                    "飞机正在下降，可以给予下降高度指令"
                } else {
                    "Aircraft is descending, provide descent altitude instruction"
                }
            } else {
                if language == "zh" {
                    "飞机正在巡航，可以给予航向、高度或频率指令"
                } else {
                    "Aircraft is cruising, provide heading, altitude or frequency instruction"
                }
            };
            
            let context = if language == "zh" {
                format!(
                    "\n\n## 实时飞行数据（必须使用）\n\n- **呼号**：{}\n- **高度**：{:.0} 米（{:.0} 英尺）\n- **速度**：{:.0} 节\n- **航向**：{:.0}°\n- **垂直速度**：{:.0} 米/分钟\n- **经纬度**：{:.4}°, {:.4}°\n\n## 飞行状态分析\n{}\n\n## 重要指令\n1. **必须在回复中使用呼号**：{}\n2. **必须根据实际高度和速度给出合理指令**\n3. **回复必须简短**（不超过30字）\n4. **只给一条指令**，不要解释\n5. 如果飞行员请求起飞，检查速度是否足够（>40节才能起飞）\n6. 如果飞行员请求着陆，检查高度是否合适（<1000米才能着陆）\n\n根据以上**真实飞行数据**和飞行员的请求，给出一条符合标准的管制指令。",
                    data.callsign,
                    altitude_m,
                    data.altitude,
                    data.speed,
                    data.heading,
                    vertical_speed_mpm,
                    data.latitude,
                    data.longitude,
                    phase_hint,
                    data.callsign
                )
            } else {
                format!(
                    "\n\n## Real-time Flight Data (MUST USE)\n\n- **Callsign**: {}\n- **Altitude**: {:.0} feet\n- **Speed**: {:.0} knots\n- **Heading**: {:.0}°\n- **Vertical Speed**: {:.0} fpm\n- **Position**: {:.4}°, {:.4}°\n\n## Flight Status Analysis\n{}\n\n## Important Instructions\n1. **MUST use callsign** in response: {}\n2. **MUST provide reasonable instruction based on actual altitude and speed**\n3. **Keep response brief** (under 20 words)\n4. **Give only ONE instruction**, no explanation\n5. If pilot requests takeoff, check speed is sufficient (>40 knots)\n6. If pilot requests landing, check altitude is appropriate (<3000 feet)\n\nBased on the **real flight data** above and pilot request, provide one standard ATC instruction.",
                    data.callsign,
                    data.altitude,
                    data.speed,
                    data.heading,
                    data.vertical_speed,
                    data.latitude,
                    data.longitude,
                    phase_hint,
                    data.callsign
                )
            };
            format!("{}{}", base_prompt, context)
        } else {
            // 没有飞行数据时的提示
            let no_data_prompt = if language == "zh" {
                "\n\n## ⚠️ 警告：未连接模拟器\n\n当前没有飞行数据。请先连接 X-Plane 或 MSFS。\n\n如果飞行员发送消息，回复：\"请先连接模拟器，我需要飞行数据才能提供管制服务。\""
            } else {
                "\n\n## ⚠️ Warning: Simulator Not Connected\n\nNo flight data available. Please connect X-Plane or MSFS first.\n\nIf pilot sends message, reply: \"Please connect simulator first. I need flight data to provide ATC service.\""
            };
            format!("{}{}", base_prompt, no_data_prompt)
        }
    }
}
