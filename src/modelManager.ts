const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

interface WhisperModel {
  name: string;
  size: string;
  url: string;
  filename: string;
  description: string;
}

interface DownloadProgress {
  model: string;
  downloaded: number;
  total: number;
  progress: number;
}

let availableModels: WhisperModel[] = [];
let downloadedModels: string[] = [];
let downloading: string | null = null;
let loadedModel: string | null = null;

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

async function loadModels() {
  try {
    availableModels = await invoke<WhisperModel[]>('get_available_models');
    downloadedModels = await invoke<string[]>('get_downloaded_models');
    renderModels();
  } catch (error) {
    console.error('Failed to load models:', error);
  }
}

async function handleDownload(modelName: string) {
  try {
    downloading = modelName;
    renderModels();
    
    await invoke('download_model', { modelName });
    await loadModels();
    alert(`模型 ${modelName} 下载成功！`);
  } catch (error) {
    alert(`下载失败: ${error}`);
  } finally {
    downloading = null;
    renderModels();
  }
}

async function handleDelete(filename: string) {
  if (!confirm(`确定要删除模型 ${filename} 吗？`)) {
    return;
  }

  try {
    await invoke('delete_model', { filename });
    await loadModels();
    if (loadedModel === filename) {
      loadedModel = null;
    }
  } catch (error) {
    alert(`删除失败: ${error}`);
  }
}

async function handleLoad(filename: string) {
  try {
    await invoke('load_model', { filename });
    loadedModel = filename;
    renderModels();
    alert(`模型 ${filename} 加载成功！`);
  } catch (error) {
    alert(`加载失败: ${error}`);
  }
}

function renderModels() {
  const container = document.getElementById('model-manager')!;
  
  let html = '<h2 style="color: #00ff00; text-align: center; margin-bottom: 20px;">🎙️ Whisper 模型管理</h2>';
  html += '<div class="models-list">';
  
  availableModels.forEach(model => {
    const isDownloaded = downloadedModels.includes(model.filename);
    const isDownloading = downloading === model.name;
    const isLoaded = loadedModel === model.filename;
    
    html += `
      <div class="model-card ${isLoaded ? 'loaded' : ''}">
        <div class="model-header">
          <h3>${model.name}</h3>
          <span class="model-size">${model.size}</span>
        </div>
        <p class="model-description">${model.description}</p>
        
        ${isDownloading ? '<div class="progress-bar" id="progress-' + model.name + '"><div class="progress-fill" style="width: 0%"></div><span class="progress-text">0%</span></div>' : ''}
        
        <div class="model-actions">
          ${!isDownloaded && !isDownloading ? `<button class="btn-download" onclick="window.downloadModel('${model.name}')">⬇️ 下载</button>` : ''}
          ${isDownloading ? '<button class="btn-downloading" disabled>⏳ 下载中...</button>' : ''}
          ${isDownloaded && !isDownloading ? `
            <button class="btn-load ${isLoaded ? 'loaded' : ''}" onclick="window.loadModel('${model.filename}')" ${isLoaded ? 'disabled' : ''}>
              ${isLoaded ? '✅ 已加载' : '📂 加载'}
            </button>
            <button class="btn-delete" onclick="window.deleteModel('${model.filename}')">🗑️ 删除</button>
          ` : ''}
        </div>
      </div>
    `;
  });
  
  html += '</div>';
  container.innerHTML = html;
}

// Listen for download progress
listen<DownloadProgress>('download-progress', (event) => {
  const progress = event.payload;
  const progressBar = document.getElementById(`progress-${progress.model}`);
  
  if (progressBar) {
    const fill = progressBar.querySelector('.progress-fill') as HTMLElement;
    const text = progressBar.querySelector('.progress-text') as HTMLElement;
    
    if (fill) fill.style.width = `${progress.progress}%`;
    if (text) text.textContent = `${progress.progress}% (${formatBytes(progress.downloaded)} / ${formatBytes(progress.total)})`;
  }
});

// Export functions to window for onclick handlers
(window as any).downloadModel = handleDownload;
(window as any).deleteModel = handleDelete;
(window as any).loadModel = handleLoad;

export function renderModelManager() {
  loadModels();
}
