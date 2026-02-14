import "./style.css";
import { renderModelManager } from "./modelManager";

const { invoke } = window.__TAURI__.tauri;

// UI Elements
const connectionStatus = document.getElementById("connection-status")!;
const frequency = document.getElementById("frequency")!;
const languageSelect = document.getElementById("language-select") as HTMLSelectElement;
const callsign = document.getElementById("callsign")!;
const altitude = document.getElementById("altitude")!;
const speed = document.getElementById("speed")!;
const heading = document.getElementById("heading")!;
const messagesContainer = document.getElementById("messages")!;
const pttButton = document.getElementById("ptt-button")!;
const connectButton = document.getElementById("connect-button")!;

// Tab switching
const tabButtons = document.querySelectorAll(".tab-button");
const tabContents = document.querySelectorAll(".tab-content");

tabButtons.forEach(button => {
  button.addEventListener("click", () => {
    const tabName = button.getAttribute("data-tab");
    
    // Update active tab button
    tabButtons.forEach(btn => btn.classList.remove("active"));
    button.classList.add("active");
    
    // Update active tab content
    tabContents.forEach(content => content.classList.remove("active"));
    document.getElementById(`${tabName}-tab`)?.classList.add("active");
    
    // Initialize model manager when switching to models tab
    if (tabName === "models") {
      renderModelManager();
    }
  });
});

// State
let isConnected = false;
let isRecording = false;
let currentLanguage = "zh";

// Add message to communication panel
function addMessage(sender: "pilot" | "atc", text: string): HTMLElement {
  const messageDiv = document.createElement("div");
  messageDiv.className = `message ${sender}`;
  
  const time = new Date().toLocaleTimeString("zh-CN", { 
    hour: "2-digit", 
    minute: "2-digit", 
    second: "2-digit" 
  });
  
  messageDiv.innerHTML = `
    <div class="message-header">
      <span class="sender">${sender === "pilot" ? "飞行员" : "ATC"}</span>
      <span class="time">${time}</span>
    </div>
    <div class="message-text">${text}</div>
  `;
  
  messagesContainer.appendChild(messageDiv);
  messagesContainer.scrollTop = messagesContainer.scrollHeight;
  
  return messageDiv;
}

// Connect to simulator
connectButton.addEventListener("click", async () => {
  if (isConnected) {
    // Disconnect
    try {
      await invoke("disconnect_simulator");
      isConnected = false;
      connectionStatus.textContent = "未连接";
      connectionStatus.style.color = "#ef4444";
      connectButton.textContent = "连接模拟器";
      addMessage("atc", "已断开连接");
    } catch (error) {
      console.error("Failed to disconnect:", error);
    }
    return;
  }
  
  // Connect
  connectButton.disabled = true;
  connectButton.textContent = "连接中...";
  
  try {
    const result = await invoke("connect_simulator", { simType: "xplane" }) as string;
    isConnected = true;
    connectionStatus.textContent = "已连接";
    connectionStatus.style.color = "#4ade80";
    connectButton.textContent = "断开连接";
    addMessage("atc", result);
  } catch (error) {
    console.error("Failed to connect:", error);
    const errorMsg = error as string;
    addMessage("atc", errorMsg || "连接失败，请确保模拟器正在运行");
  } finally {
    connectButton.disabled = false;
  }
});

// PTT (Push-to-Talk) button
pttButton.addEventListener("mousedown", async () => {
  if (!isConnected) {
    addMessage("atc", "⚠️ 请先连接模拟器");
    return;
  }
  
  isRecording = true;
  pttButton.classList.add("recording");
  pttButton.querySelector(".ptt-text")!.textContent = "🎙️ 正在录音...";
  
  try {
    await invoke("start_recording");
  } catch (error) {
    console.error("Failed to start recording:", error);
    const errorMsg = error as string;
    addMessage("atc", errorMsg || "❌ 录音启动失败");
    isRecording = false;
    pttButton.classList.remove("recording");
    pttButton.querySelector(".ptt-text")!.textContent = "按住通话 (PTT)";
  }
});

pttButton.addEventListener("mouseup", async () => {
  if (!isRecording) return;
  
  isRecording = false;
  pttButton.classList.remove("recording");
  pttButton.querySelector(".ptt-text")!.textContent = "按住通话 (PTT)";
  
  // Show processing indicator
  const processingMsg = addMessage("atc", "⏳ 正在处理...");
  
  try {
    const transcript = await invoke("stop_recording") as string;
    
    // Remove processing message
    processingMsg.remove();
    
    addMessage("pilot", transcript);
    
    // Show thinking indicator
    const thinkingMsg = addMessage("atc", "🤔 AI 思考中...");
    
    // Get ATC response
    const response = await invoke("get_atc_response", { 
      message: transcript,
      language: currentLanguage 
    }) as string;
    
    // Remove thinking message
    thinkingMsg.remove();
    
    addMessage("atc", response);
  } catch (error) {
    console.error("Failed to process recording:", error);
    const errorMsg = error as string;
    
    // Remove processing/thinking message
    const lastMsg = messagesContainer.lastElementChild;
    if (lastMsg && lastMsg.textContent?.includes("处理中") || lastMsg?.textContent?.includes("思考中")) {
      lastMsg.remove();
    }
    
    addMessage("atc", errorMsg || "❌ 处理失败，请重试");
  }
});

// Language selection
languageSelect.addEventListener("change", (e) => {
  currentLanguage = (e.target as HTMLSelectElement).value;
  addMessage("atc", currentLanguage === "zh" ? "已切换到中文模式" : "Switched to English mode");
});

// Update flight info periodically
setInterval(async () => {
  if (!isConnected) return;
  
  try {
    const flightData = await invoke("get_flight_data") as {
      callsign: string;
      altitude: number;
      speed: number;
      heading: number;
    };
    
    callsign.textContent = flightData.callsign;
    altitude.textContent = `${Math.round(flightData.altitude)} ft`;
    speed.textContent = `${Math.round(flightData.speed)} kts`;
    heading.textContent = `${Math.round(flightData.heading)}°`;
  } catch (error) {
    console.error("Failed to get flight data:", error);
  }
}, 1000);

// Initial message
addMessage("atc", "欢迎使用 Virtual ATC，请连接模拟器开始");
