// src/components/ServerControls.jsx
import React from 'react';
import './ServerControls.css';

const ServerControls = ({ isRunning, url, onStart, onStop }) => {
  return (
    <div className="server-controls">
      <div className="server-buttons">
        {isRunning ? (
          <button onClick={onStop} className="stop-button">サーバー停止</button>
        ) : (
          <button onClick={onStart} className="start-button">サーバー起動</button>
        )}
      </div>
      {isRunning && url && (
        <div className="server-url">
          <p>サーバーURL: <a href={url} target="_blank" rel="noopener noreferrer">{url}</a></p>
          <p>このURLにアクセスして音楽ファイルをアップロードできます</p>
        </div>
      )}
    </div>
  );
};

export default ServerControls;
