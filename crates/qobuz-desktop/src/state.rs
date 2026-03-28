// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Shared application state managed by Tauri.

use qobuz_lib::api::Track;
use qobuz_lib::cache::AudioCache;
use qobuz_lib::config::Config;
use qobuz_lib::player::Player;
use qobuz_lib::QobuzClient;
use std::sync::Mutex;

/// Loop mode values: 0=Off, 1=Track, 2=Queue
pub const LOOP_OFF: u8 = 0;
pub const LOOP_TRACK: u8 = 1;
pub const LOOP_QUEUE: u8 = 2;

/// Application state shared across Tauri commands.
pub struct AppState {
    pub player: Mutex<Player>,
    pub api: Mutex<QobuzClient>,
    pub config: Mutex<Config>,
    pub cache: AudioCache,
    pub queue: Mutex<Vec<Track>>,
    pub queue_index: Mutex<usize>,
    pub loop_mode: Mutex<u8>,
}

impl AppState {
    pub fn new(player: Player) -> Self {
        let config = Config::load();
        let mut api = QobuzClient::new(&config.app_id, &config.app_secret);
        if let Some(token) = &config.user_auth_token {
            api.set_token(token.clone());
        }

        Self {
            player: Mutex::new(player),
            api: Mutex::new(api),
            config: Mutex::new(config),
            cache: AudioCache::new(),
            queue: Mutex::new(Vec::new()),
            queue_index: Mutex::new(0),
            loop_mode: Mutex::new(LOOP_OFF),
        }
    }
}
