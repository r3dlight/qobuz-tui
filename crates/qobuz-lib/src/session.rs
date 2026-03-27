// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Session persistence — saves and restores play queue, volume, and loop mode
//! to `~/.config/qobuz-tui/session.json` between app restarts.

use crate::api::Track;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Session {
    pub queue: Vec<SessionTrack>,
    pub queue_index: usize,
    pub volume: f32,
    pub loop_mode: u8, // 0=Off, 1=Track, 2=Queue
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
    pub track_number: Option<u32>,
}

impl SessionTrack {
    pub fn from_track(t: &Track) -> Self {
        Self {
            id: t.id.clone(),
            title: t.title.clone(),
            artist: t.artist_name().to_string(),
            album: t.album_title().to_string(),
            duration: t.duration,
            track_number: t.track_number,
        }
    }

    pub fn to_track(&self) -> Track {
        Track {
            id: self.id.clone(),
            title: self.title.clone(),
            duration: self.duration,
            track_number: self.track_number,
            performer: Some(crate::api::Artist {
                name: self.artist.clone(),
            }),
            album: Some(crate::api::AlbumBrief {
                title: self.album.clone(),
                artist: None,
            }),
        }
    }
}

fn session_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("qobuz-tui")
        .join("session.json")
}

pub fn load() -> Session {
    let path = session_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Session::default()
    }
}

/// Save session state. Returns true on success, false on failure.
pub fn save(session: &Session) -> bool {
    let path = session_path();
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    let Ok(content) = serde_json::to_string_pretty(session) else {
        return false;
    };
    fs::write(&path, content).is_ok()
}
