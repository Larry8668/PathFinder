import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useKeyboardNavigation } from "../hooks/useKeyboardNavigation";
import Fuse from "fuse.js";
import { getCurrentWindow } from '@tauri-apps/api/window';

export default function ClipboardPage({ query }) {
  const [clipboardItems, setClipboardItems] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadClipboardHistory();
    
    // Listen for clipboard updates
    const unlisten = listen('clipboard-update', (event) => {
      setClipboardItems(prev => [event.payload, ...prev]);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const loadClipboardHistory = async () => {
    try {
      setLoading(true);
      const history = await invoke('get_clipboard_history');
      setClipboardItems(history);
      setError(null);
    } catch (err) {
      console.error('Failed to load clipboard history:', err);
      setError('Failed to load clipboard history');
    } finally {
      setLoading(false);
    }
  };

 const handleSelect = async (item) => {
  try {
    // Update access metadata
    await invoke('update_clipboard_access', { id: item.id });
    
    // Hide window FIRST so the paste target gets focus
    const window = getCurrentWindow();
    await window.hide();
    
    // Wait for window to hide and previous app to get focus
    await new Promise(resolve => setTimeout(resolve, 50));
    
    // Now copy to clipboard and simulate paste
    await invoke('paste_clipboard_item', { content: item.content });
    
    // Reload to show updated access count (in background)
    loadClipboardHistory();
    
    console.log('Pasted:', item.content.substring(0, 50));
  } catch (err) {
    console.error('Failed to paste item:', err);
    setError('Failed to paste clipboard item');
  }
};

  const handleDelete = async (item, e) => {
    e.stopPropagation();
    try {
      await invoke('delete_clipboard_item', { id: item.id });
      setClipboardItems(prev => prev.filter(i => i.id !== item.id));
    } catch (err) {
      console.error('Failed to delete item:', err);
      setError('Failed to delete clipboard item');
    }
  };

  const handleClearAll = async (e) => {
    e.stopPropagation();
    if (window.confirm('Clear all clipboard history? This cannot be undone.')) {
      try {
        await invoke('clear_clipboard_history');
        setClipboardItems([]);
      } catch (err) {
        console.error('Failed to clear history:', err);
        setError('Failed to clear clipboard history');
      }
    }
  };

  const formatTimestamp = (timestamp) => {
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

  const formatSize = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / 1048576).toFixed(1)} MB`;
  };

  // Filter items based on query
  const fuse = new Fuse(clipboardItems, { 
    keys: ["content"], 
    threshold: 0.4 
  });
  
  const filtered = query
    ? fuse.search(query).map((res) => res.item)
    : clipboardItems;

  const { getItemProps } = useKeyboardNavigation(filtered, handleSelect);

  if (loading) {
    return (
      <div className="option-list">
        <div className="loading">Loading clipboard history...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="option-list">
        <div className="error-message">{error}</div>
        <button className="retry-btn" onClick={loadClipboardHistory}>Retry</button>
      </div>
    );
  }

  return (
    <div className="clipboard-container">
      {clipboardItems.length > 0 && (
        <div className="clipboard-header">
          <span className="clipboard-count">{clipboardItems.length} items</span>
          <button className="clear-all-btn" onClick={handleClearAll}>
            Clear History
          </button>
        </div>
      )}
      
      <div className="option-list">
        {filtered.length === 0 ? (
          <div className="empty-state">
            {query ? 'No matching clipboard items found' : 'No clipboard history yet. Copy something to get started.'}
          </div>
        ) : (
          filtered.map((item, idx) => (
            <div
              {...getItemProps(idx)}
              className={`option-item clipboard-item ${getItemProps(idx).className}`}
              key={item.id}
            >
              <div className="clipboard-main-content">
                <div className="clipboard-text">
                  {item.content.length > 100
                    ? `${item.content.substring(0, 100)}...`
                    : item.content}
                </div>
                <div className="clipboard-metadata">
                  <span className="meta-item">
                    <span className="meta-label">Added:</span>
                    <span className="meta-value">{formatTimestamp(item.created_at)}</span>
                  </span>
                  <span className="meta-divider">•</span>
                  <span className="meta-item">
                    <span className="meta-label">Used:</span>
                    <span className="meta-value">{item.access_count}×</span>
                  </span>
                  <span className="meta-divider">•</span>
                  <span className="meta-item">
                    <span className="meta-label">Size:</span>
                    <span className="meta-value">{formatSize(item.size)}</span>
                  </span>
                </div>
              </div>
              <button
                className="delete-btn"
                onClick={(e) => handleDelete(item, e)}
                title="Delete"
              >
                ×
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}