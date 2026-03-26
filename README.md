<p align="center">
  <img src="assets/logo.svg" alt="Qobuz TUI" width="400"/>
</p>

<p align="center">
  A terminal-based Qobuz music player built with Rust, <a href="https://github.com/ratatui/ratatui">ratatui</a>, and <a href="https://github.com/RustAudio/rodio">rodio</a>.
</p>

Browse your Qobuz library, search for music, and play Hi-Res FLAC directly from your terminal.

## Features

- **Authentication** — Log in with your Qobuz email and password (API credentials are fetched automatically)
- **Search** — Search for tracks and albums, switch between result types with Tab
- **Album browsing** — Open any album to see its full tracklist
- **Favorite albums** — Browse and manage your Qobuz favorite albums
- **Playlists** — Browse and play your Qobuz playlists
- **Play queue** — Automatically queues all tracks when playing from an album, playlist, or search results
- **Auto-advance** — Next track plays automatically when the current one finishes
- **Gapless playback** — Next track is pre-downloaded 15 seconds before the current one ends
- **Session persistence** — Queue, volume, and loop mode are saved on exit and restored on startup
- **Streaming playback** — Audio plays while still downloading (progressive streaming), no more waiting for full download
- **Seek** — Skip forward/backward 10 seconds with `,` / `;`
- **Audio playback** — Play FLAC (up to 24-bit/192kHz), MP3, and other formats via rodio + symphonia, with automatic quality fallback
- **Album download** — Download an entire album to local cache with `d`
- **Progress bar** — Real-time playback progress with elapsed/total time
- **Scrollable lists** — Navigate large lists with scroll indicators
- **Audio cache** — Downloaded tracks are cached locally to avoid re-downloading
- **Playback controls** — Play/pause, next/previous track, volume
- **Loop mode** — Loop current track, entire queue, or off (`r` to cycle)
- **Sandboxed** — [Landlock](https://landlock.io/) restricts filesystem access to only what the app needs (Linux 5.13+)

## Prerequisites

### System dependencies

The ALSA development library is required for audio output on Linux.

On Debian/Ubuntu:

```bash
sudo apt-get install -y libasound2-dev
```

On Fedora:

```bash
sudo dnf install alsa-lib-devel
```

On Arch:

```bash
sudo pacman -S alsa-lib
```

> **Troubleshooting:** If you see `The system library 'alsa' required by crate 'alsa-sys' was not found`, install the ALSA dev package for your distro and retry.

### Qobuz account

You need an active Qobuz account with a streaming subscription. The internal API credentials (`app_id` and `app_secret`) are **fetched automatically** from the Qobuz web player on first launch.

### Rust toolchain

This project uses Rust **edition 2024**, which requires a **nightly** toolchain. The included `rust-toolchain.toml` will automatically select it when you run `cargo build`. To install nightly manually:

```bash
rustup toolchain install nightly
```

## Build

```bash
cargo build --release
```

The binary will be at `./target/release/qobuz-tui`.

## Usage

```bash
./target/release/qobuz-tui
```

On first launch, the app automatically fetches API credentials from the Qobuz web player, then presents a login screen where you enter your email and password. Credentials are saved locally so you only need to log in once.

## Keyboard shortcuts

### Login screen

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Navigate between fields |
| `Enter` | Submit / move to next field |
| `Esc` | Quit |

### Search screen

The search bar is always active — just type your query and press Enter.

| Key | Action |
|-----|--------|
| *(type)* | Edit search query |
| `Enter` | Search (if no results) / Play track / Open album |
| `Up` / `Down` | Navigate results (with scroll) |
| `Shift+Tab` | Switch between Tracks and Albums results |

### Favorite albums

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate albums (with scroll) |
| `Enter` | Open album tracklist |
| `x` | Remove album from favorites |

### Playlists

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate playlists (with scroll) |
| `Enter` | Open playlist |

### Playlist view

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate tracks (with scroll) |
| `Enter` | Play from selected track (queues entire playlist) |
| `Backspace` | Go back |

### Album view

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate tracks (with scroll) |
| `Enter` | Play from selected track (queues remaining tracks) |
| `d` | Download entire album to cache |
| `f` | Add album to favorites |
| `Backspace` | Go back |

### Global (works on all screens)

| Key | Action |
|-----|--------|
| `Tab` | Next tab (Search → Albums → Playlists) |
| `p` | Play / Pause |
| `n` | Next track in queue |
| `N` | Previous track in queue |
| `r` | Cycle loop mode: Off → `LOOP:TRACK` → `LOOP:ALL` → Off |
| `,` / `;` | Seek backward / forward 10 seconds |
| `Left` / `Right` | Volume down / up |
| `Esc` | Quit |
| `Ctrl+C` | Force quit |

> **Note:** `p`, `n`, `N`, `r`, `f`, `x`, `d`, `,`, `;` are shortcut keys — they type normally on the search and login screens.

## Configuration

Settings are stored in `~/.config/qobuz-tui/config.toml`:

```toml
# Fetched automatically on first launch.
# Override manually if auto-detection fails.
app_id = ""
app_secret = ""

email = "your@email.com"

# Audio quality (format_id):
#   5  = MP3 320kbps
#   6  = FLAC 16-bit / 44.1kHz (CD)
#   7  = FLAC 24-bit / 96kHz (Hi-Res)
#   27 = FLAC 24-bit / 192kHz (Hi-Res, default)
format_id = 27
```

The `user_auth_token` is stored in this file after login. Delete the file to log out.

Session state (queue, volume, loop mode) is saved to `~/.config/qobuz-tui/session.json` on exit and automatically restored on startup.

If a track is not available in the selected quality (or the download fails), the player automatically falls back to a lower quality: 27 → 7 → 6 → 5.

Audio cache is stored in `~/.cache/qobuz-tui/`, organized by artist and album:

```
~/.cache/qobuz-tui/
└── Artist Name/
    └── Album Title/
        ├── 01 - Track Title.flac
        ├── 02 - Another Track.flac
        └── ...
```

> **Note:** The config file contains your authentication token in plaintext. It is stored in your user config directory with standard file permissions. Do not share this file.

> **Fallback:** If auto-detection of API credentials fails, you can extract them manually from the Qobuz web player using browser DevTools and set `app_id` / `app_secret` in the config file.

## Project structure

Cargo workspace with a reusable library and a TUI binary:

```
crates/
├── qobuz-lib/              Reusable library (for TUI, Tauri, or other frontends)
│   └── src/
│       ├── lib.rs           Public API re-exports
│       ├── api.rs           Qobuz API client, models, authentication
│       ├── config.rs        Config loading/saving
│       ├── cache.rs         Local audio file cache (Artist/Album/Track)
│       ├── player.rs        Audio playback with seek and progress
│       ├── stream.rs        Streaming buffer for progressive playback
│       └── session.rs       Session persistence (queue, volume, loop)
└── qobuz-tui/              Terminal UI binary
    └── src/
        ├── main.rs          Entry point, terminal setup, event loop
        ├── app.rs           Application state, play queue, input handling
        ├── ui.rs            TUI rendering with ratatui
        └── sandbox.rs       Landlock filesystem sandbox (Linux)
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
