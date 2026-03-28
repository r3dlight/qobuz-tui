// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Shared application state managed by Tauri.

use qobuz_lib::api::Track;
use qobuz_lib::cache::AudioCache;
use qobuz_lib::config::Config;
use qobuz_lib::player::Player;
use qobuz_lib::{AudioQuality, QobuzClient};
use std::sync::Mutex;

/// Application state shared across Tauri commands.
/// Wrapped in `Mutex` because Tauri manages state as `State<AppState>`.
pub struct AppState {
    pub player: Mutex<Option<Player>>,
    pub api: Mutex<QobuzClient>,
    pub config: Mutex<Config>,
    pub cache: AudioCache,
    pub queue: Mutex<Vec<Track>>,
    pub queue_index: Mutex<usize>,
    pub loop_mode: Mutex<u8>,
}

impl AppState {
    pub fn new() -> Self {
        let config = Config::load();
        let mut api = QobuzClient::new(&config.app_id, &config.app_secret);
        if let Some(token) = &config.user_auth_token {
            api.set_token(token.clone());
        }

        Self {
            player: Mutex::new(None), // Initialized after OutputStream is created
            api: Mutex::new(api),
            config: Mutex::new(config),
            cache: AudioCache::new(),
            queue: Mutex::new(Vec::new()),
            queue_index: Mutex::new(0),
            loop_mode: Mutex::new(0),
        }
    }
}
