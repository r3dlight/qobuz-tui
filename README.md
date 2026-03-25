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
- **Favorite albums** — Quick access to your Qobuz favorite albums via F2
- **Play queue** — Automatically queues all tracks when playing from an album or search results
- **Auto-advance** — Next track plays automatically when the current one finishes
- **Audio playback** — Play FLAC (up to 24-bit/192kHz), MP3, and other formats via rodio + symphonia, with automatic quality fallback
- **Album download** — Download an entire album to local cache with `d`
- **Progress bar** — Real-time playback progress with elapsed/total time
- **Scrollable lists** — Navigate large lists with scroll indicators
- **Audio cache** — Downloaded tracks are cached locally to avoid re-downloading
- **Playback controls** — Play/pause, next/previous track, volume
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
| `Tab` | Switch between Tracks and Albums results |
| `F1` | Search tab |
| `F2` | Favorite albums tab |

### Favorite albums

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate albums (with scroll) |
| `Enter` | Open album tracklist |
| `F1` | Back to search |

### Album view

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate tracks (with scroll) |
| `Enter` | Play from selected track (queues remaining tracks) |
| `d` | Download entire album to cache |
| `Backspace` | Go back |

### Global (works on all screens)

| Key | Action |
|-----|--------|
| `p` | Play / Pause |
| `n` | Next track in queue |
| `N` | Previous track in queue |
| `Left` / `Right` | Volume down / up |
| `Esc` | Quit |
| `Ctrl+C` | Force quit |

> **Note:** `p`, `n`, `N` are used for typing on the search and login screens. Playback controls work on the favorites, album view, and all other screens.

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

```
src/
├── main.rs      Entry point, terminal setup, event loop
├── config.rs    Config loading/saving
├── api.rs       Qobuz API client (auth, search, streaming URLs, favorites)
├── player.rs    Audio playback with progress tracking
├── cache.rs     Local audio file cache (Artist/Album/Track)
├── sandbox.rs   Landlock filesystem/network sandbox
├── app.rs       Application state, play queue, input handling
└── ui.rs        TUI rendering with ratatui
```

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).
