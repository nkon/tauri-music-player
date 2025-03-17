// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
mod http;
mod music;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle();
            // 必要なディレクトリの初期化
            music::init_music_directory(app_handle)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::start_http_server,
            commands::stop_http_server,
            commands::get_server_url,
            commands::get_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
