import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function HlsScreenSharePage({ query }) {
  const [hasFfmpeg, setHasFfmpeg] = useState(null);
  const [serverInfo, setServerInfo] = useState(null);
  const [error, setError] = useState('');
  const [isServerRunning, setIsServerRunning] = useState(false);

  useEffect(() => {
    // Check FFmpeg availability
    checkFfmpeg();
    // Check if server is already running
    checkServerStatus();
  }, []);

  const checkFfmpeg = async () => {
    try {
      const available = await invoke('check_ffmpeg');
      setHasFfmpeg(available);
      if (!available) {
        setError('FFmpeg is not installed. Please install FFmpeg to use screen sharing.');
      }
    } catch (err) {
      setError(`Failed to check FFmpeg: ${err}`);
      setHasFfmpeg(false);
    }
  };

  const checkServerStatus = async () => {
    try {
      const info = await invoke('get_hls_server_info');
      if (info && info.running) {
        setIsServerRunning(true);
        setServerInfo({
          code: info.code,
          port: info.port,
          url: info.url,
          tunnelUrl: info.tunnelUrl || null,
          tunnelDomain: info.tunnelDomain || null,
        });
      }
    } catch (err) {
      // Server not running, ignore
    }
  };

  const startServer = async () => {
    try {
      setError('');
      const info = await invoke('start_hls_server_cmd');
      setServerInfo(info);
      setIsServerRunning(true);
      console.log('HLS server started:', info);
    } catch (err) {
      setError(`Failed to start server: ${err}`);
    }
  };

  const stopServer = async () => {
    try {
      setError('');
      await invoke('stop_hls_server_cmd');
      setIsServerRunning(false);
      setServerInfo(null);
      console.log('HLS server stopped');
    } catch (err) {
      setError(`Failed to stop server: ${err}`);
    }
  };

  return (
    <div className="screen-share-page">
      <div className="screen-share-container">
        <h2>📺 HLS Screen Share Server</h2>
        
        {hasFfmpeg === false && (
          <div className="error-message" style={{ marginTop: '20px', padding: '15px', backgroundColor: '#ffe6e6', borderRadius: '8px', color: '#d00' }}>
            <p><strong>FFmpeg Not Found</strong></p>
            <p>FFmpeg is required for screen sharing. Please install FFmpeg:</p>
            <ul style={{ marginTop: '10px', paddingLeft: '20px' }}>
              <li><strong>macOS:</strong> <code>brew install ffmpeg</code></li>
              <li><strong>Windows:</strong> <code>choco install ffmpeg</code></li>
              <li><strong>Linux:</strong> <code>sudo apt-get install ffmpeg</code></li>
            </ul>
            <button onClick={checkFfmpeg} className="btn-secondary" style={{ marginTop: '10px' }}>
              Check Again
            </button>
          </div>
        )}

        {hasFfmpeg === null && (
          <div style={{ marginTop: '20px', padding: '15px' }}>
            <p>Checking FFmpeg availability...</p>
          </div>
        )}

        {hasFfmpeg === true && !isServerRunning && (
          <div className="start-server-section" style={{ marginTop: '20px' }}>
            <p>Start the HLS streaming server to begin screen sharing.</p>
            <p style={{ fontSize: '14px', color: 'rgba(255,255,255,0.7)', marginTop: '10px' }}>
              The server will capture your screen and stream it via HLS. Use your Vercel client to view the stream.
            </p>
            <button onClick={startServer} className="btn-primary" style={{ marginTop: '15px' }}>
              ▶️ Start Server
            </button>
          </div>
        )}

        {isServerRunning && serverInfo && (
          <>
            <div className="code-section" style={{ marginTop: '20px' }}>
              <p><strong>Access Code:</strong></p>
              <div className="code-display">{serverInfo.code}</div>
              <button 
                onClick={() => navigator.clipboard.writeText(serverInfo.code)}
                className="btn-secondary"
                style={{ marginTop: '10px' }}
              >
                📋 Copy Code
              </button>
            </div>

            <div className="action-section" style={{ marginTop: '30px' }}>
              <h3>Server Information:</h3>
              <div style={{ marginTop: '10px', padding: '15px', backgroundColor: 'rgba(0,0,0,0.2)', borderRadius: '4px', fontSize: '12px' }}>
                <p><strong>Local URL:</strong> <code>{serverInfo.url}</code></p>
                {serverInfo.tunnelUrl && (
                  <>
                    <p style={{ marginTop: '8px' }}><strong>🌐 Tunnel URL:</strong></p>
                    <code style={{ wordBreak: 'break-all', display: 'block', marginTop: '5px', color: '#4CAF50' }}>
                      {serverInfo.tunnelUrl}
                    </code>
                    {serverInfo.tunnelDomain && (
                      <p style={{ marginTop: '5px', fontSize: '11px', color: 'rgba(255,255,255,0.6)' }}>
                        Domain: <code>{serverInfo.tunnelDomain}</code>
                      </p>
                    )}
                  </>
                )}
                <p style={{ marginTop: '8px' }}><strong>Stream Endpoint:</strong></p>
                <code style={{ wordBreak: 'break-all', display: 'block', marginTop: '5px' }}>
                  {serverInfo.tunnelUrl || serverInfo.url}/stream.m3u8?code={serverInfo.code}
                </code>
              </div>
              <button 
                onClick={() => navigator.clipboard.writeText(`${serverInfo.tunnelUrl || serverInfo.url}/stream.m3u8?code=${serverInfo.code}`)}
                className="btn-secondary"
                style={{ marginTop: '10px' }}
              >
                📋 Copy Stream URL
              </button>
              {serverInfo.tunnelDomain && (
                <button 
                  onClick={() => navigator.clipboard.writeText(serverInfo.tunnelDomain)}
                  className="btn-secondary"
                  style={{ marginTop: '10px', marginLeft: '10px' }}
                >
                  📋 Copy Domain
                </button>
              )}
              <button onClick={stopServer} className="btn-danger" style={{ marginTop: '10px', marginLeft: '10px' }}>
                ⏹️ Stop Server
              </button>
            </div>

            <div style={{ marginTop: '20px', padding: '15px', backgroundColor: 'rgba(255,255,255,0.05)', borderRadius: '8px', fontSize: '14px' }}>
              <p><strong>Instructions:</strong></p>
              <ol style={{ marginTop: '10px', paddingLeft: '20px' }}>
                <li>The server is now capturing your screen and streaming via HLS</li>
                <li>Use your Vercel client to connect to this server</li>
                {serverInfo.tunnelDomain ? (
                  <>
                    <li>Enter the <strong>Domain</strong>: <code>{serverInfo.tunnelDomain}</code></li>
                    <li>Enter the <strong>Access Code</strong>: <code>{serverInfo.code}</code></li>
                    <li>The client will connect via the tunnel URL</li>
                  </>
                ) : (
                  <>
                    <li>Provide the access code and localhost URL to your client</li>
                    <li>The client will connect to: <code>{serverInfo.url}/stream.m3u8?code={serverInfo.code}</code></li>
                    <li style={{ color: '#ffa500' }}>⚠️ Tunnel not available - using localhost only</li>
                  </>
                )}
              </ol>
            </div>

            <div className="server-info" style={{ marginTop: '20px', fontSize: '12px', color: '#666' }}>
              <p>✅ Server running at: <code>{serverInfo.url}</code></p>
            </div>
          </>
        )}

        {error && (
          <div className="error-message" style={{ marginTop: '20px', padding: '10px', backgroundColor: '#ffe6e6', borderRadius: '4px', color: '#d00' }}>
            <p>❌ {error}</p>
          </div>
        )}
      </div>
    </div>
  );
}

