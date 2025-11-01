import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function ScreenSharePage({ query }) {
  const [shareCode, setShareCode] = useState('');
  const [serverUrl, setServerUrl] = useState('');
  const [error, setError] = useState('');
  const [isServerRunning, setIsServerRunning] = useState(false);

  useEffect(() => {
    // Get server URL when component mounts
    invoke('get_signaling_server_url')
      .then((url) => setServerUrl(url))
      .catch((err) => setError(`Failed to get server URL: ${err}`));
  }, []);

  const startServer = async () => {
    try {
      setError('');
      const code = await invoke('start_signaling_server_cmd');
      setShareCode(code);
      setIsServerRunning(true);
      console.log('Server started with code:', code);
    } catch (err) {
      setError(`Failed to start server: ${err}`);
    }
  };

  const openSharerPage = async () => {
    try {
      const url = `${serverUrl}/sharer`;
      await invoke('open_browser_url', { url });
    } catch (err) {
      setError(`Failed to open browser: ${err}`);
    }
  };

  const openViewerPage = async () => {
    try {
      const url = `${serverUrl}/viewer`;
      await invoke('open_browser_url', { url });
    } catch (err) {
      setError(`Failed to open browser: ${err}`);
    }
  };

  return (
    <div className="screen-share-page">
      <div className="screen-share-container">
        <h2>🖥️ Screen Share</h2>
        
        {!isServerRunning && (
          <div className="start-server-section">
            <p>Start the signaling server to begin screen sharing.</p>
            <button onClick={startServer} className="btn-primary">
              Start Server
            </button>
          </div>
        )}

        {isServerRunning && shareCode && (
          <>
            <div className="code-section">
              <p><strong>Share Code:</strong></p>
              <div className="code-display">{shareCode}</div>
              <button 
                onClick={() => navigator.clipboard.writeText(shareCode)}
                className="btn-secondary"
              >
                📋 Copy Code
              </button>
            </div>

            <div className="action-section" style={{ marginTop: '30px' }}>
              <h3>Open in Browser:</h3>
              <div style={{ display: 'flex', gap: '10px', marginTop: '10px' }}>
                <button onClick={openSharerPage} className="btn-primary">
                  🖥️ Open Sharer Page
                </button>
                <button onClick={openViewerPage} className="btn-primary">
                  👁️ Open Viewer Page
                </button>
              </div>
              <p style={{ marginTop: '15px', fontSize: '14px', color: 'rgba(255,255,255,0.7)' }}>
                <strong>Instructions:</strong><br/>
                1. Click "Open Sharer Page" to select and share your screen<br/>
                2. Enter the share code above in the browser page<br/>
                3. Click "Open Viewer Page" in a new tab/window to view<br/>
                4. Enter the same share code in the viewer page
              </p>
            </div>

            <div className="server-info" style={{ marginTop: '20px', fontSize: '12px', color: '#666' }}>
              <p>Server running at: <code>{serverUrl}</code></p>
            </div>
          </>
        )}

        {error && (
          <div className="error-message" style={{ marginTop: '10px', padding: '10px', backgroundColor: '#ffe6e6', borderRadius: '4px', color: '#d00' }}>
            <p>❌ {error}</p>
          </div>
        )}
      </div>
    </div>
  );
}
