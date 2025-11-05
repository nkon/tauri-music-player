# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Tauri 2.0 cross-platform music player for macOS and iOS. It features MP3 playback with ID3 tag extraction, a dual HTTP server architecture for file upload/streaming, and a React frontend.

**Key Technologies:**
- Backend: Rust (Tauri 2.0, Warp, Tokio)
- Frontend: React + JavaScript (Vite)
- Package Manager: pnpm

## Build Commands

```bash
# Development (Desktop)
pnpm tauri dev

# Development (iOS - requires Xcode setup)
pnpm tauri ios dev

# Production Build (macOS Apple Silicon)
cargo tauri build --target aarch64-apple-darwin

# Production Build (macOS Intel)
cargo tauri build --target x86_64-apple-darwin

# Production Build (iOS)
pnpm tauri ios build
# Output: src-tauri/target/aarch64-apple-ios/release/bundle/{name}.ipa

# Frontend only (for testing)
pnpm dev  # Runs on port 1420
pnpm build
```

## Architecture

### Dual HTTP Server Pattern

The backend runs TWO separate HTTP servers with distinct purposes:

1. **Upload Server (port 3030)** - Dynamically binds to local IP
   - HTML form for file upload/deletion
   - Graceful shutdown support via oneshot channels
   - Started/stopped by user via ServerControls component

2. **Streaming Server (port 3031)** - Always on localhost:3031
   - Streams MP3 files for playback
   - Auto-starts on app initialization (in `lib.rs`)
   - Cannot be stopped (intentional design)

**Critical:** Frontend hardcodes `http://127.0.0.1:3031` for streaming (`src/App.jsx:58`). This bypasses Tauri's `asset://` protocol limitation for dynamic user files.

### Frontend Audio Playback

```javascript
// App.jsx playback flow:
audioRef.current.src = "http://127.0.0.1:3031/stream/{track.id}"
audioRef.current.play()
```

Uses HTML5 `<audio>` element, NOT Web Audio API. MediaSession API integration enables OS-level media controls.

### ID3 Tag Strategy

`music.rs` implements fallback logic:
1. Try ID3v2 tags (`id3::Tag::read_from_path`)
2. Fallback to ID3v1 tags (`id3::v1tag::read_from_path`)
3. If both fail, use filename as title

### State Management

- **Rust:** Global `SERVER_STATE` via `lazy_static!` (prevents multiple server instances)
- **React:** All state centralized in `App.jsx` (tracks, currentTrack, isPlaying, isShuffleMode)

## File Structure

```
src/                          # React frontend
├── App.jsx                   # Central state hub, Tauri IPC calls
├── components/
│   ├── PlayerControls.jsx    # Playback UI + MediaSession API
│   ├── TrackList.jsx         # Track listing
│   └── ServerControls.jsx    # Upload server controls

src-tauri/src/                # Rust backend
├── lib.rs                    # Tauri app setup, auto-starts streaming server
├── commands.rs               # Tauri IPC command definitions
├── music.rs                  # Track management, ID3 parsing
└── http.rs                   # Dual HTTP server implementation (405 LOC)
```

## Important Implementation Details

### 1. Play Count Tracking (Currently Disabled)
The code infrastructure exists (`increment_play_count`, `.count` files) but is **not actively being written** to disk. All tracks show `play_count: 0`. The Tauri commands are registered but may not be called from frontend.

### 2. Audio Initialization Pattern
```javascript
// CORRECT way (in App.jsx):
const audioRef = useRef(null);

useEffect(() => {
  audioRef.current = new Audio();  // Initialize AFTER mount
  // ...
}, []);
```

**Do NOT** use `useRef(new Audio())` directly - it fails in Tauri's SSR-like environment.

### 3. iOS Icon Sizing
iOS requires explicit SVG sizing in CSS:
```css
.controls-buttons button svg {
  width: 18px;
  height: 18px;
  min-width: 18px;  /* Critical for iOS Safari */
  min-height: 18px;
}
```

### 4. Security Measures
- Directory traversal protection: checks for ".." in paths
- File type filtering: only `.mp3` files accepted
- Multipart upload limit: 100MB max (`http.rs:364`)

## Platform-Specific Notes

### macOS
- Supports both aarch64 (Apple Silicon) and x86_64 (Intel)
- Full file system access available
- GitHub Actions builds both architectures automatically

### iOS
- Requires Apple Developer Team ID in Xcode project
- **Environment Setup:** Set `TAURI_APPLE_DEVELOPMENT_TEAM` environment variable
  - Run `source set_dev_env.sh` in project root before building
  - Or export manually: `export TAURI_APPLE_DEVELOPMENT_TEAM="YOUR_TEAM_ID"`
- First build must be done via Xcode GUI to configure signing
- File system limited to app sandbox (`{APP_DATA}/music/`)
- MediaSession API provides lockscreen controls
- Info.plist must include `UIBackgroundModes: audio`

## Known Issues & Limitations

1. **No shuffle queue management** - Uses pure random selection, not true shuffle
2. **No progress seeking** - Cannot scrub within tracks
3. **No volume control** - Uses system volume only
4. **Hardcoded streaming port** - Assumes 3031 is available
5. **Memory usage** - Entire MP3 loaded for streaming (not chunked)
6. **Single playlist** - No user-defined playlists

## Development Workflow

### Testing iOS Locally
1. Set up Apple Developer Team ID:
   ```bash
   source set_dev_env.sh  # Sets TAURI_APPLE_DEVELOPMENT_TEAM
   ```
2. Connect physical iPhone via USB OR use simulator
3. `pnpm tauri ios dev` - auto-detects device, builds, and deploys
4. Vite dev server binds to local WiFi IP (e.g., `192.168.0.29:1420`)
5. App connects to dev server for hot reload

**Note:** `set_dev_env.sh` is not tracked in git (contains personal Team ID)

### Troubleshooting iOS Builds
```bash
# Error: "Signing requires a development team"
# Solution 1: Set environment variable
source set_dev_env.sh

# Solution 2: Open Xcode and set Team ID manually
open src-tauri/gen/apple/tauri-music-player.xcodeproj

# Error: "project.yml not found"
# Solution: Regenerate iOS project
rm -rf src-tauri/gen/apple
pnpm tauri ios init
```

### Adding New Tauri Commands
1. Define function in `commands.rs` with `#[tauri::command]`
2. Register in `lib.rs`: `invoke_handler(tauri::generate_handler![...])`
3. Call from React: `import { invoke } from '@tauri-apps/api/tauri'`

## Vite Configuration Notes

- **Port 1420** is fixed (required by Tauri)
- HMR uses port 1421
- `src-tauri/` is excluded from file watching
- Changes to Rust code require restart of `pnpm tauri dev`

## External Documentation

- Project documentation: https://deepwiki.com/nkon/tauri-music-player
- Build instructions: See `README-build.md` for detailed macOS/iOS setup
- Development notes: See `NOTE.md` (Japanese)
