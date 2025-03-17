// src-tauri/src/commands.rs
use crate::http::{get_url, start_server, stop_server};
use crate::music::{get_all_tracks, Track};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Serialize)]
pub struct TrackResponse {
    tracks: Vec<Track>,
}

#[tauri::command]
pub async fn get_tracks(app_handle: AppHandle) -> Result<TrackResponse, String> {
    let tracks = get_all_tracks(&app_handle).map_err(|e| e.to_string())?;
    Ok(TrackResponse { tracks })
}

// #[tauri::command]
// pub async fn play_track(id: String) -> Result<(), String> {
//     // このコマンドはフロントエンド側でWeb Audio APIを使用するため、
//     // バックエンド側では再生処理は実装しません
//     Ok(())
// }

// #[tauri::command]
// pub async fn increment_play_count(app_handle: AppHandle, id: String) -> Result<(), String> {
//     increment_track_play_count(&app_handle, &id).map_err(|e| e.to_string())
// }

#[tauri::command]
pub async fn start_http_server(app_handle: AppHandle) -> Result<(), String> {
    start_server(app_handle).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_http_server() -> Result<(), String> {
    stop_server().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_server_url() -> Result<String, String> {
    get_url().await.map_err(|e| e.to_string())
}
