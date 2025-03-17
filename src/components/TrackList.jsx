// src/components/TrackList.jsx
import React from 'react';
import { FaPlay, FaPause } from 'react-icons/fa';
import './TrackList.css';

const TrackList = ({ tracks, currentTrack, onTrackSelect, isPlaying }) => {
  return (
    <div className="track-list">
      <h2>楽曲リスト</h2>
      {tracks.length === 0 ? (
        <p>楽曲が見つかりません。サーバーからMP3ファイルをアップロードしてください。</p>
      ) : (
        <ul className="track-items">
          {tracks.map((track) => (
            <li 
              key={track.id} 
              className={`track-item ${currentTrack && currentTrack.id === track.id ? 'active' : ''}`}
              onClick={() => onTrackSelect(track)}
            >
              <div className="track-info">
                <span className="track-title">{track.title || track.file_name}</span>
                <span className="track-artist">{track.artist || 'Unknown Artist'}</span>
                <span className="track-album">{track.album || 'Unknown Album'}</span>
              </div>
              <div className="track-controls">
                {currentTrack && currentTrack.id === track.id && (
                  <span className="track-status">
                    {isPlaying ? <FaPause /> : <FaPlay />}
                  </span>
                )}
                <span className="track-play-count">再生回数: {track.play_count}</span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default TrackList;
