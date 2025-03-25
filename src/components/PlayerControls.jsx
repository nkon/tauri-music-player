// src/components/PlayerControls.jsx
import React from 'react';
import { FaPlay, FaPause, FaStepForward, FaStepBackward, FaRedo, FaRandom, FaUndo } from 'react-icons/fa';
import './PlayerControls.css';

const PlayerControls = ({
  isPlaying,
  onPlay,
  onPause,
  onNext,
  onPrevious,
  onRestart,
  onShuffleToggle,
  isShuffleMode,
  currentTrack
}) => {
  return (
    <div className="player-controls">
      <div className="now-playing">
        {currentTrack ? (
          <>
            <h3>{currentTrack.title || currentTrack.file_name}</h3>
            <p>{currentTrack.artist || 'Unknown Artist'} - {currentTrack.album || 'Unknown Album'}</p>
          </>
        ) : (
          <h3>曲を選択してください</h3>
        )}
      </div>

      <div className="controls-buttons">
        <button onClick={onRestart} disabled={!currentTrack}>
          <FaUndo />
        </button>
        <button onClick={isPlaying ? onPause : onPlay} disabled={!currentTrack} className="play-button">
          {isPlaying ? <FaPause /> : <FaPlay />}
        </button>
        <button onClick={onNext} disabled={!currentTrack}>
          <FaStepForward />
        </button>
        <button
          onClick={onShuffleToggle}
          className={`shuffle-button ${isShuffleMode ? 'active' : 'sequential'}`}
        >
          <FaRandom />
        </button>
      </div>
    </div>
  );
};

export default PlayerControls;
