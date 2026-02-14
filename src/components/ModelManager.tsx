import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

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

export function ModelManager() {
  const [availableModels, setAvailableModels] = useState<WhisperModel[]>([]);
  const [downloadedModels, setDownloadedModels] = useState<string[]>([]);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [loadedModel, setLoadedModel] = useState<string | null>(null);

  useEffect(() => {
    loadModels();
    
    // 监听下载进度
    const unlisten = listen<DownloadProgress>('download-progress', (event) => {
      setProgress(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const loadModels = async () => {
    try {
      const available = await invoke<WhisperModel[]>('get_available_models');
      const downloaded = await invoke<string[]>('get_downloaded_models');
      setAvailableModels(available);
      setDownloadedModels(downloaded);
    } catch (error) {
      console.error('Failed to load models:', error);
    }
  };

  const handleDownload = async (modelName: string) => {
    try {
      setDownloading(modelName);
      setProgress(null);
      await invoke('download_model', { modelName });
      await loadModels();
      alert(`模型 ${modelName} 下载成功！`);
    } catch (error) {
      alert(`下载失败: ${error}`);
    } finally {
      setDownloading(null);
      setProgress(null);
    }
  };

  const handleDelete = async (filename: string) => {
    if (!confirm(`确定要删除模型 ${filename} 吗？`)) {
      return;
    }

    try {
      await invoke('delete_model', { filename });
      await loadModels();
      if (loadedModel === filename) {
        setLoadedModel(null);
      }
    } catch (error) {
      alert(`删除失败: ${error}`);
    }
  };

  const handleLoad = async (filename: string) => {
    try {
      await invoke('load_model', { filename });
      setLoadedModel(filename);
      alert(`模型 ${filename} 加载成功！`);
    } catch (error) {
      alert(`加载失败: ${error}`);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
  };

  return (
    <div className="model-manager">
      <h2>🎙️ Whisper 模型管理</h2>
      
      <div className="models-list">
        {availableModels.map((model) => {
          const isDownloaded = downloadedModels.includes(model.filename);
          const isDownloading = downloading === model.name;
          const isLoaded = loadedModel === model.filename;
          const showProgress = isDownloading && progress?.model === model.name;

          return (
            <div key={model.name} className={`model-card ${isLoaded ? 'loaded' : ''}`}>
              <div className="model-header">
                <h3>{model.name}</h3>
                <span className="model-size">{model.size}</span>
              </div>
              
              <p className="model-description">{model.description}</p>
              
              {showProgress && (
                <div className="progress-bar">
                  <div 
                    className="progress-fill" 
                    style={{ width: `${progress.progress}%` }}
                  />
                  <span className="progress-text">
                    {progress.progress}% ({formatBytes(progress.downloaded)} / {formatBytes(progress.total)})
                  </span>
                </div>
              )}
              
              <div className="model-actions">
                {!isDownloaded && !isDownloading && (
                  <button 
                    className="btn-download"
                    onClick={() => handleDownload(model.name)}
                  >
                    ⬇️ 下载
                  </button>
                )}
                
                {isDownloading && (
                  <button className="btn-downloading" disabled>
                    ⏳ 下载中...
                  </button>
                )}
                
                {isDownloaded && !isDownloading && (
                  <>
                    <button 
                      className={`btn-load ${isLoaded ? 'loaded' : ''}`}
                      onClick={() => handleLoad(model.filename)}
                      disabled={isLoaded}
                    >
                      {isLoaded ? '✅ 已加载' : '📂 加载'}
                    </button>
                    <button 
                      className="btn-delete"
                      onClick={() => handleDelete(model.filename)}
                    >
                      🗑️ 删除
                    </button>
                  </>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <style>{`
        .model-manager {
          padding: 20px;
          max-width: 800px;
          margin: 0 auto;
        }

        .model-manager h2 {
          color: #00ff00;
          margin-bottom: 20px;
          text-align: center;
        }

        .models-list {
          display: flex;
          flex-direction: column;
          gap: 15px;
        }

        .model-card {
          background: rgba(0, 255, 0, 0.05);
          border: 1px solid rgba(0, 255, 0, 0.3);
          border-radius: 8px;
          padding: 15px;
          transition: all 0.3s;
        }

        .model-card.loaded {
          border-color: #00ff00;
          background: rgba(0, 255, 0, 0.1);
        }

        .model-card:hover {
          border-color: rgba(0, 255, 0, 0.6);
        }

        .model-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 10px;
        }

        .model-header h3 {
          color: #00ff00;
          margin: 0;
          font-size: 18px;
        }

        .model-size {
          color: #888;
          font-size: 14px;
        }

        .model-description {
          color: #aaa;
          margin: 10px 0;
          font-size: 14px;
        }

        .progress-bar {
          position: relative;
          height: 30px;
          background: rgba(0, 0, 0, 0.3);
          border-radius: 4px;
          margin: 10px 0;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          background: linear-gradient(90deg, #00ff00, #00cc00);
          transition: width 0.3s;
        }

        .progress-text {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          color: white;
          font-size: 12px;
          font-weight: bold;
          text-shadow: 0 0 4px black;
        }

        .model-actions {
          display: flex;
          gap: 10px;
          margin-top: 10px;
        }

        .model-actions button {
          flex: 1;
          padding: 8px 16px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.3s;
        }

        .btn-download {
          background: #00ff00;
          color: black;
        }

        .btn-download:hover {
          background: #00cc00;
        }

        .btn-downloading {
          background: #666;
          color: white;
          cursor: not-allowed;
        }

        .btn-load {
          background: #0088ff;
          color: white;
        }

        .btn-load:hover:not(:disabled) {
          background: #0066cc;
        }

        .btn-load.loaded {
          background: #00ff00;
          color: black;
          cursor: default;
        }

        .btn-delete {
          background: #ff4444;
          color: white;
        }

        .btn-delete:hover {
          background: #cc0000;
        }
      `}</style>
    </div>
  );
}
