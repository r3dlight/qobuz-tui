// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Qobuz Desktop — Tauri 2 application entry point.
//!
//! Exposes qobuz-lib functionality as Tauri commands for the Vue.js frontend.

#![deny(unsafe_code)]

mod commands;
mod state;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::search,
            commands::get_album,
            commands::get_artist,
            commands::get_favorites,
            commands::get_playlists,
            commands::get_playlist,
            commands::play_track,
            commands::pause,
            commands::next_track,
            commands::previous_track,
            commands::set_volume,
            commands::get_player_state,
            commands::shuffle_queue,
            commands::toggle_loop,
            commands::add_favorite,
            commands::remove_favorite,
            commands::download_album,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
