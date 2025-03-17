// src/components/ServerControls.jsx
import React from 'react';
import './ServerControls.css';

const ServerControls = ({ isRunning, url, onStart, onStop }) => {
  return (
    <div className="server-controls">
      <h2>サーバー設定</h2>
      <div className="server-status">
        <p>ステータス: {isRunning ? '起動中' : '停止中'}</p>
        {isRunning && url && (
          <div className="server-url">
            <p>サーバーURL: <a href={url} target="_blank" rel="noopener noreferrer">{url}</a></p>
            <p>このURLにアクセスして音楽ファイルをアップロードできます</p>
          </div>
        )}
      </div>
      <div className="server-buttons">
        {isRunning ? (
          <button onClick={onStop} className="stop-button">サーバー停止</button>
        ) : (
          <button onClick={onStart} className="start-button">サーバー起動</button>
        )}
      </div>
    </div>
  );
};

export default ServerControls;
