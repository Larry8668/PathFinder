import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import Fuse from "fuse.js";
import { getCurrentWindow } from '@tauri-apps/api/window';

export default function OpenFilePage({ query }) {
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [isIndexed, setIsIndexed] = useState(false);

  useEffect(() => {
    loadFiles();
  }, []);

  useEffect(() => {
    if (query && query.length > 0) {
      searchFiles(query);
    } else {
      loadFiles();
    }
  }, [query]);

  const loadFiles = async () => {
    try {
      setLoading(true);
      const [apps, recentFiles] = await Promise.all([
        invoke('get_applications'),
        invoke('get_recent_files')
      ]);
      
      const allFiles = [...apps, ...recentFiles];
      setFiles(allFiles);
      setError(null);
      setIsIndexed(true);
    } catch (err) {
      console.error('Failed to load files:', err);
      setError('Failed to load files');
      setIsIndexed(false);
    } finally {
      setLoading(false);
    }
  };

  const searchFiles = async (searchQuery) => {
    try {
      setLoading(true);
      const results = await invoke('search_files', { query: searchQuery });
      setFiles(results);
      setError(null);
    } catch (err) {
      console.error('Failed to search files:', err);
      setError('Failed to search files');
    } finally {
      setLoading(false);
    }
  };

  const refreshIndex = async () => {
    try {
      setLoading(true);
      await invoke('refresh_file_index');
      await loadFiles();
    } catch (err) {
      console.error('Failed to refresh index:', err);
      setError('Failed to refresh file index');
    } finally {
      setLoading(false);
    }
  };

  const handleSelect = async (item) => {
    try {
      // Hide window first
      const window = getCurrentWindow();
      await window.hide();
      
      // Wait for window to hide
      await new Promise(resolve => setTimeout(resolve, 50));
      
      // Open the file/app
      await invoke('open_file', { path: item.path });
      
      console.log('Opened:', item.name);
    } catch (err) {
      console.error('Failed to open file:', err);
      setError('Failed to open file');
    }
  };

  const getFileIcon = (item) => {
    if (item.is_app) {
      return '🚀';
    }
    
    const extension = item.file_type.toLowerCase();
    switch (extension) {
      case 'pdf': return '📄';
      case 'doc':
      case 'docx': return '📝';
      case 'xls':
      case 'xlsx': return '📊';
      case 'ppt':
      case 'pptx': return '📊';
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif': return '🖼️';
      case 'mp4':
      case 'avi':
      case 'mov': return '🎥';
      case 'mp3':
      case 'wav': return '🎵';
      case 'zip':
      case 'rar': return '📦';
      case 'txt': return '📄';
      default: return '📁';
    }
  };

  const formatFileSize = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(1)} GB`;
  };

  const formatDate = (timestamp) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now - date;
    
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    
    if (minutes < 1) return 'Just now';
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    
    return date.toLocaleDateString();
  };

  const { getItemProps } = useKeyboardNavigation(files, handleSelect);

  if (loading) {
    return (
      <div className="option-list">
        <div className="loading">Loading files...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="option-list">
        <div className="error-message">{error}</div>
        <button className="retry-btn" onClick={loadFiles}>Retry</button>
      </div>
    );
  }

  if (!isIndexed) {
    return (
      <div className="option-list">
        <div className="empty-state">
          <p>File index not found. Click to build index.</p>
          <button className="refresh-btn" onClick={refreshIndex}>
            Build File Index
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="file-container">
      <div className="file-header">
        <span className="file-count">{files.length} items</span>
        <button className="refresh-btn" onClick={refreshIndex}>
          Refresh Index
        </button>
      </div>
      
      <div className="option-list">
        {files.length === 0 ? (
          <div className="empty-state">
            {query ? 'No matching files found' : 'No files found. Click "Refresh Index" to build the file index.'}
          </div>
        ) : (
          files.map((file, idx) => (
            <div
              {...getItemProps(idx)}
              className={`option-item file-item ${getItemProps(idx).className}`}
              key={`${file.path}-${idx}`}
            >
              <div className="file-main-content">
                <div className="file-icon">{getFileIcon(file)}</div>
                <div className="file-info">
                  <div className="file-name">{file.name}</div>
                  <div className="file-metadata">
                    <span className="meta-item">
                      <span className="meta-label">Type:</span>
                      <span className="meta-value">{file.file_type || 'Unknown'}</span>
                    </span>
                    <span className="meta-divider">•</span>
                    <span className="meta-item">
                      <span className="meta-label">Size:</span>
                      <span className="meta-value">{formatFileSize(file.size)}</span>
                    </span>
                    <span className="meta-divider">•</span>
                    <span className="meta-item">
                      <span className="meta-label">Modified:</span>
                      <span className="meta-value">{formatDate(file.modified)}</span>
                    </span>
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
