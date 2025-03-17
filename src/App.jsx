import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import TrackList from './components/TrackList';
import PlayerControls from './components/PlayerControls';
import ServerControls from './components/ServerControls'
import "./App.css";

function App() {
  const [tracks, setTracks] = useState([]);
  const [currentTrack, setCurrentTrack] = useState(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isShuffleMode, setIsShuffleMode] = useState(false);
  const [serverUrl, setServerUrl] = useState('');
  const [isServerRunning, setIsServerRunning] = useState(false);
  const audioRef = useRef(new Audio());

  // HTTPサーバーの起動
  const startServer = async () => {
    try {
      await invoke('start_http_server');
      const url = await invoke('get_server_url');
      setServerUrl(url);
      setIsServerRunning(true);
    } catch (error) {
      console.error('Failed to start server:', error);
    }
    fetchTracks();
  };

  // HTTPサーバーの停止
  const stopServer = async () => {
    try {
      await invoke('stop_http_server');
      setServerUrl('');
      setIsServerRunning(false);
    } catch (error) {
      console.error('Failed to stop server:', error);
    }
  };



  // 楽曲再生処理
  const playTrack = async (track) => {
    if (currentTrack && currentTrack.id === track.id && isPlaying) {
      // すでに再生中の曲をタップした場合は一時停止
      audioRef.current.pause();
      setIsPlaying(false);
      return;
    }

    setCurrentTrack(track);

    // タウリコマンドでファイルパスを取得
    try {
      // オーディオ要素を更新して再生
      const url = await invoke('get_server_url');
      // const url="http://127.0.0.1:3030"; これではうまく動かない。warpがこのアドレスにはバインドされていないのだろう
      audioRef.current.src = url + "/stream/" + track.id;
      console.error('Now playing:', audioRef.current.src);
      audioRef.current.play();
      setIsPlaying(true);

      // 再生回数を増加
      // await invoke('increment_play_count', { id: track.id });

      // 表示を更新するため楽曲リストを再取得
      fetchTracks();
    } catch (error) {
      console.error('Failed to play track:', error);
    }
  };

  // 次の曲を再生
  const playNextTrack = () => {
    if (!currentTrack || tracks.length === 0) return;

    let nextTrackIndex;

    if (isShuffleMode) {
      // ランダムモードの場合、ランダムに選択
      nextTrackIndex = Math.floor(Math.random() * tracks.length);
    } else {
      // 通常モードの場合、次の曲
      const currentIndex = tracks.findIndex(t => t.id === currentTrack.id);
      nextTrackIndex = (currentIndex + 1) % tracks.length;
    }

    playTrack(tracks[nextTrackIndex]);
  };

  // 前の曲を再生
  const playPreviousTrack = () => {
    if (!currentTrack || tracks.length === 0) return;

    const currentIndex = tracks.findIndex(t => t.id === currentTrack.id);
    let prevTrackIndex = (currentIndex - 1 + tracks.length) % tracks.length;

    playTrack(tracks[prevTrackIndex]);
  };

  // 再生/一時停止の切り替え
  const togglePlay = () => {
    if (!currentTrack) return;

    if (isPlaying) {
      audioRef.current.pause();
    } else {
      audioRef.current.play();
    }

    setIsPlaying(!isPlaying);
  };

  // 最初から再生
  const restartTrack = () => {
    if (!currentTrack) return;

    audioRef.current.currentTime = 0;
    audioRef.current.play();
    setIsPlaying(true);
  };

  // 楽曲リストの取得
  const fetchTracks = async () => {
    try {
      const response = await invoke('get_tracks');
      setTracks(response.tracks);
    } catch (error) {
      console.error('Failed to fetch tracks:', error);
    }
  };

  // コンポーネントマウント時に楽曲リストを取得
  useEffect(() => {
    fetchTracks();

    if (!isServerRunning) {
      startServer();
    }

    // 再生終了時の処理
    const handleEnded = () => {
      playNextTrack();
    };

    audioRef.current.addEventListener('ended', handleEnded);

    return () => {
      audioRef.current.removeEventListener('ended', handleEnded);
    };
  }, []);



  return (
    <div className="app-container">
      <h1>音楽プレイヤー</h1>

      <ServerControls
        isRunning={isServerRunning}
        url={serverUrl}
        onStart={startServer}
        onStop={stopServer}
      />

      <div className="player-section">
        <PlayerControls
          isPlaying={isPlaying}
          onPlay={togglePlay}
          onPause={togglePlay}
          onNext={playNextTrack}
          onPrevious={playPreviousTrack}
          onRestart={restartTrack}
          onShuffleToggle={() => setIsShuffleMode(!isShuffleMode)}
          isShuffleMode={isShuffleMode}
          currentTrack={currentTrack}
        />
      </div>

      <div className="track-list-section">
        <TrackList
          tracks={tracks}
          currentTrack={currentTrack}
          onTrackSelect={playTrack}
          isPlaying={isPlaying}
        />
      </div>
    </div>

  );
}

export default App;
