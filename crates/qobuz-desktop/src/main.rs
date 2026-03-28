// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Qobuz Desktop — Tauri 2 application entry point.

#![deny(unsafe_code)]

mod commands;
mod state;

use qobuz_lib::Player;
use rodio::{OutputStream, Sink};
use state::AppState;

fn main() {
    // Audio output must be created on the main thread (OutputStream is !Send).
    // Keep _stream alive for the entire app lifetime.
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to open audio output");
    let sink = Sink::try_new(&stream_handle).expect("Failed to create audio sink");
    let player = Player::new(sink);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new(player))
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::check_auth,
            commands::search,
            commands::get_album,
            commands::get_featured,
            commands::get_featured_by_genre,
            commands::get_genres,
            commands::get_artist,
            commands::get_favorites,
            commands::get_playlists,
            commands::get_playlist,
            commands::play_track,
            commands::play_queue_from,

            commands::pause,
            commands::next_track,
            commands::previous_track,
            commands::seek,
            commands::set_volume,
            commands::get_player_state,
            commands::get_recent,
            commands::shuffle_queue,
            commands::toggle_loop,
            commands::add_favorite,
            commands::remove_favorite,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
