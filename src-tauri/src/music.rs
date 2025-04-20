// src-tauri/src/music.rs
use id3::v1::Tag as V1Tag;
use id3::{Tag, TagLike};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Track {
    pub id: String,
    pub file_name: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    path: String,
    pub play_count: u32,
}

pub fn init_music_directory(app_handle: &AppHandle) -> Result<(), io::Error> {
    let music_dir = get_music_dir(app_handle)?;
    if !music_dir.exists() {
        fs::create_dir_all(&music_dir)?;
    }

    // 再生回数データのディレクトリも作成
    // let stats_dir = get_stats_dir(app_handle)?;
    // if !stats_dir.exists() {
    //     fs::create_dir_all(&stats_dir)?;
    // }

    Ok(())
}

pub fn get_music_dir(app_handle: &AppHandle) -> Result<PathBuf, io::Error> {
    let app_dir = app_handle.path().app_data_dir().map_err(|_| {
        io::Error::new(io::ErrorKind::NotFound, "Could not find app data directory")
    })?;
    Ok(app_dir.join("music"))
}

// fn get_stats_dir(app_handle: &AppHandle) -> Result<PathBuf, io::Error> {
//     let app_dir = app_handle.path_resolver().app_data_dir().ok_or_else(|| {
//         io::Error::new(io::ErrorKind::NotFound, "Could not find app data directory")
//     })?;
//     Ok(app_dir.join("stats"))
// }

pub fn get_all_tracks(app_handle: &AppHandle) -> Result<Vec<Track>, io::Error> {
    let music_dir = get_music_dir(app_handle)?;

    // let stats_dir = get_stats_dir(app_handle)?;
    let mut tracks = Vec::new();

    if !music_dir.exists() {
        return Ok(tracks);
    }

    for entry in fs::read_dir(music_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "mp3") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown.mp3")
                .to_string();

            let id = file_name.clone();

            // ID3タグ情報を抽出
            if let Ok(tag) = Tag::read_from_path(&path) {
                // 再生回数を取得
                // let play_count = get_play_count(&stats_dir, &id)?;
                tracks.push(Track {
                    id,
                    file_name,
                    title: tag.title().map(|s| s.to_string()),
                    artist: tag.artist().map(|s| s.to_string()),
                    album: tag.album().map(|s| s.to_string()),
                    path: path.to_string_lossy().to_string(),
                    play_count: 0,
                });
            } else if let Ok(tag) = V1Tag::read_from_path(&path) {
                // 再生回数を取得
                // let play_count = get_play_count(&stats_dir, &id)?;
                tracks.push(Track {
                    id,
                    file_name,
                    title: Some(tag.title),
                    artist: Some(tag.artist),
                    album: Some(tag.album),
                    path: path.to_string_lossy().to_string(),
                    play_count: 0,
                });
            } else {
                tracks.push(Track {
                    id,
                    file_name,
                    title: None,
                    artist: None,
                    album: None,
                    path: path.to_string_lossy().to_string(),
                    play_count: 0,
                });
            }
        }
    }

    Ok(tracks)
}

// fn get_play_count(stats_dir: &Path, track_id: &str) -> Result<u32, io::Error> {
//     let stats_path = stats_dir.join(format!("{}.count", track_id));

//     if !stats_path.exists() {
//         return Ok(0);
//     }

//     let mut file = File::open(stats_path)?;
//     let mut content = String::new();
//     file.read_to_string(&mut content)?;

//     content.trim().parse::<u32>().map_err(|_| {
//         io::Error::new(io::ErrorKind::InvalidData, "Invalid play count data")
//     })
// }

// pub fn increment_track_play_count(app_handle: &AppHandle, track_id: &str) -> Result<(), io::Error> {
//     let stats_dir = get_stats_dir(app_handle)?;
//     let stats_path = stats_dir.join(format!("{}.count", track_id));

//     let play_count = if stats_path.exists() {
//         let mut file = File::open(&stats_path)?;
//         let mut content = String::new();
//         file.read_to_string(&mut content)?;

//         content.trim().parse::<u32>().unwrap_or(0) + 1
//     } else {
//         1
//     };

//     let mut file = File::create(stats_path)?;
//     write!(file, "{}", play_count)?;

//     Ok(())
// }
