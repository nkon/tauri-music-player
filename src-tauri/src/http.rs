// src-tauri/src/http.rs
use crate::music::{get_all_tracks, get_music_dir};
use bytes::Buf;
use futures::TryStreamExt;
use std::fs::{self, File};
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;
use tokio::sync::oneshot;
use urlencoding::decode;
use warp::http::{Response, StatusCode};
use warp::multipart::{FormData, Part};
use warp::{Filter, Rejection, Reply};

// グローバル状態の管理
lazy_static::lazy_static! {
    static ref SERVER_STATE: Arc<Mutex<Option<ServerState>>> = Arc::new(Mutex::new(None));
}

struct ServerState {
    addr: SocketAddr,
    shutdown_tx: oneshot::Sender<()>,
}

fn start_streaming_server(app_handle: AppHandle) {
    let app_handle_filter = warp::any().map(move || app_handle.clone());

    // ルートページ - ファイルアップロードフォーム
    let index2 = warp::path::end()
        .and(warp::get())
        .and(app_handle_filter.clone())
        .and_then(handle_index2);

    // MP3ファイルのストリーミング
    let stream = warp::path("stream")
        .and(warp::path::param::<String>())
        .and(warp::get())
        .and(app_handle_filter.clone())
        .and_then(handle_stream);

    let routes = index2.or(stream);

    // サーバーを別スレッドで起動
    tokio::spawn(async move {
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3031);
        println!("Starting HTTP server2 on {}", addr2);
        warp::serve(routes).bind(addr2).await;
    });
}

fn start_upload_server(
    app_handle: AppHandle,
    addr: SocketAddr,
    shutdown_rx: oneshot::Receiver<()>,
) {
    let music_dir = get_music_dir(&app_handle)
        .map_err(|e| e.to_string())
        .unwrap();
    println!("music_dir: {}", music_dir.to_string_lossy());

    let app_handle_filter = warp::any().map(move || app_handle.clone());

    // ルートページ - ファイルアップロードフォーム
    let index = warp::path::end()
        .and(warp::get())
        .and(app_handle_filter.clone())
        .and_then(handle_index);

    // ファイルアップロード処理
    let upload = warp::path("upload")
        .and(warp::post())
        .and(warp::multipart::form().max_length(100_000_000))
        .and(app_handle_filter.clone())
        .and_then(handle_upload);

    // ファイル削除処理
    let delete = warp::path("delete")
        .and(warp::post())
        .and(warp::body::form())
        .and(app_handle_filter.clone())
        .and_then(handle_delete);

    let routes = index.or(upload).or(delete);

    // サーバーを別スレッドで起動
    tokio::spawn(async move {
        println!("Starting HTTP server on {}", addr);
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

        server.await;
        println!("HTTP server stopped");
    });
}

pub async fn start_server(app_handle: AppHandle) -> Result<(), String> {
    let mut state = SERVER_STATE.lock().unwrap();
    if state.is_some() {
        return Err("Server is already running".to_string());
    }

    start_streaming_server(app_handle.clone());

    // 停止用のチャネル
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // ローカルIPアドレスの取得
    let ip = local_ip_address::local_ip().map_err(|e| e.to_string())?;
    let addr = SocketAddr::new(ip, 3030);

    start_upload_server(app_handle.clone(), addr, shutdown_rx);

    // サーバー情報を保存
    *state = Some(ServerState { addr, shutdown_tx });

    Ok(())
}

pub async fn stop_server() -> Result<(), String> {
    let mut state = SERVER_STATE.lock().unwrap();
    if let Some(server_state) = state.take() {
        // 停止信号を送信
        let _ = server_state.shutdown_tx.send(());
        Ok(())
    } else {
        Err("Server is not running".to_string())
    }
}

pub async fn get_url() -> Result<String, String> {
    let state = SERVER_STATE.lock().unwrap();
    if let Some(server_state) = state.as_ref() {
        Ok(format!("http://{}", server_state.addr))
    } else {
        Err("Server is not running".to_string())
    }
}

async fn handle_index(app_handle: AppHandle) -> Result<impl Reply, Rejection> {
    let tracks = get_all_tracks(&app_handle).map_err(|e| {
        eprintln!("Error getting tracks: {}", e);
        warp::reject::custom(ServerError(e.to_string()))
    })?;

    // HTMLでアップロードフォームと曲リストを生成
    let mut html = String::from(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>音楽プレイヤー管理</title>
        <style>
            body { font-family: sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }
            .upload-form { border: 1px solid #ccc; padding: 20px; margin-bottom: 20px; border-radius: 5px; }
            .track-list { border: 1px solid #ccc; padding: 20px; border-radius: 5px; }
            table { width: 100%; border-collapse: collapse; }
            th, td { padding: 8px; text-align: left; border-bottom: 1px solid #ddd; }
            button { cursor: pointer; padding: 5px 10px; }
        </style>
    </head>
    <body>
        <h1>音楽プレイヤー管理</h1>
        
        <div class="upload-form">
            <h2>MP3ファイルのアップロード</h2>
            <form action="/upload" method="post" enctype="multipart/form-data">
                <input type="file" name="mp3file" accept=".mp3" required>
                <button type="submit">アップロード</button>
            </form>
        </div>
        
        <div class="track-list">
            <h2>楽曲リスト</h2>
            <table>
                <thead>
                    <tr>
                        <th>曲名</th>
                        <th>アーティスト</th>
                        <th>アルバム</th>
                        <th>ファイル名</th>
                        <th>再生回数</th>
                        <th>操作</th>
                    </tr>
                </thead>
                <tbody>
    "#,
    );

    for track in &tracks {
        let title = track.title.as_deref().unwrap_or("不明");
        let artist = track.artist.as_deref().unwrap_or("不明");
        let album = track.album.as_deref().unwrap_or("不明");

        html.push_str(&format!(
            r#"
                    <tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>
                            <form action="/delete" method="post">
                                <input type="hidden" name="file_id" value="{}">
                                <button type="submit">削除</button>
                            </form>
                        </td>
                    </tr>
        "#,
            title, artist, album, track.file_name, track.play_count, track.id
        ));
    }

    html.push_str(
        r#"
                </tbody>
            </table>
        </div>
    </body>
    </html>
    "#,
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html))
}

async fn handle_index2(_app_handle: AppHandle) -> Result<impl Reply, Rejection> {
    let html = String::from(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>音楽プレイヤーストリーミング</title>
    </head>
    <body>
        <h1>音楽プレイヤーストリーミング</h1>
    </body>
    </html>
    "#,
    );

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html))
}

async fn handle_upload(mut form: FormData, app_handle: AppHandle) -> Result<impl Reply, Rejection> {
    let music_dir = get_music_dir(&app_handle).map_err(|e| {
        eprintln!("Error getting music directory: {}", e);
        warp::reject::custom(ServerError(e.to_string()))
    })?;

    println!("handle_upload:{:?}", form);

    while let Ok(Some(part)) = form.try_next().await {
        let file_name = match part.filename() {
            Some(file_name) => file_name.to_string(),
            None => {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("ファイルが見つかりません".to_string()));
            }
        };
        println!("Receiving file: {}", file_name);

        // MP3ファイルのみ許可
        if !(file_name.ends_with(".mp3") || file_name.ends_with(".MP3")) {
            continue;
        }

        let file_path = music_dir.join(file_name);
        if let Err(e) = save_file(part, &file_path).await {
            eprintln!("Error saving file: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("ファイルの保存に失敗しました: {}", e)));
        }
    }

    // アップロード完了後、インデックスページにリダイレクト
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/")
        .body("".to_string()))
}

async fn save_file(mut part: Part, file_path: &Path) -> Result<(), String> {
    let mut file = File::create(file_path).map_err(|e| e.to_string())?;

    while let Some(chunk) = part.data().await {
        file.write_all(chunk.unwrap().chunk())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn handle_delete(
    form: std::collections::HashMap<String, String>,
    app_handle: AppHandle,
) -> Result<impl Reply, Rejection> {
    let music_dir = get_music_dir(&app_handle).map_err(|e| {
        eprintln!("Error getting music directory: {}", e);
        warp::reject::custom(ServerError(e.to_string()))
    })?;

    if let Some(file_id) = form.get("file_id") {
        // let sanitized_id = sanitize_filename::sanitize(file_id);
        let sanitized_id = file_id;

        // ディレクトリトラバーサル対策
        if sanitized_id.contains("..") {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("無効なファイル名です"));
        }

        let mut found = false;

        // 指定されたIDに一致するファイルを探す
        for entry in fs::read_dir(&music_dir)
            .map_err(|e| warp::reject::custom(ServerError(e.to_string())))?
        {
            let entry = entry.map_err(|e| warp::reject::custom(ServerError(e.to_string())))?;
            let path = entry.path();

            if path.is_file() && path.file_name().and_then(|n| n.to_str()) == Some(sanitized_id) {
                // ファイルを削除
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Error removing file: {}", e);
                    return Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("ファイルの削除に失敗しました"));
                }

                found = true;
                break;
            }
        }

        if !found {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("指定されたファイルが見つかりませんでした"));
        }
    }

    // 削除完了後、インデックスページにリダイレクト
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/")
        .body(""))
}

async fn handle_stream(file_id: String, app_handle: AppHandle) -> Result<impl Reply, Rejection> {
    let music_dir = get_music_dir(&app_handle).map_err(|e| {
        eprintln!("Error getting music directory: {}", e);
        warp::reject::custom(ServerError(e.to_string()))
    })?;

    let file_id = decode(&file_id).unwrap().into_owned();
    println!("handle_stream: field_id:{}", file_id);

    // let sanitized_id = sanitize_filename::sanitize(&file_id);
    let sanitized_id = file_id;

    // ディレクトリトラバーサル対策
    if sanitized_id.contains("..") {
        return Err(warp::reject::not_found());
    }

    let file_path = music_dir.join(&sanitized_id);

    println!("handle_stream: file_path:{}", file_path.to_string_lossy());

    if !file_path.exists() || !file_path.is_file() {
        return Err(warp::reject::not_found());
    }

    let file = tokio::fs::File::open(file_path).await.map_err(|e| {
        eprintln!("Error opening file: {}", e);
        warp::reject::custom(ServerError(e.to_string()))
    })?;

    let stream = tokio_util::io::ReaderStream::new(file);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "audio/mpeg")
        .body(warp::hyper::Body::wrap_stream(stream)))
}

#[derive(Debug)]
struct ServerError(String);

impl warp::reject::Reject for ServerError {}
