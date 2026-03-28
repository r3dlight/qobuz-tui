// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Tauri commands — each function is callable from the Vue.js frontend.

use crate::state::AppState;
use qobuz_lib::api::{Album, ArtistDetail, Playlist, SearchResults, Track};
use qobuz_lib::cache::TrackMeta;
use qobuz_lib::player::AudioQuality;
use serde::Serialize;
use tauri::State;

/// Player state sent to the frontend for rendering.
#[derive(Serialize, Clone)]
pub struct PlayerState {
    pub is_playing: bool,
    pub is_loading: bool,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub duration: u64,
    pub elapsed: u64,
    pub progress: f64,
    pub volume: f32,
    pub quality: Option<String>,
    pub seekable: bool,
    pub queue_len: usize,
    pub queue_index: usize,
    pub loop_mode: u8,
}

// ── Auth ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut api = state.api.lock().unwrap().clone();
    let token = api.login(&email, &password).await.map_err(|e| e.to_string())?;
    // Update shared state with authenticated client
    *state.api.lock().unwrap() = api;
    let mut config = state.config.lock().unwrap();
    config.user_auth_token = Some(token.clone());
    config.email = Some(email);
    config.app_id = state.api.lock().unwrap().app_id.clone();
    config.app_secret = state.api.lock().unwrap().app_secret.clone();
    let _ = config.save();
    Ok(token)
}

#[tauri::command]
pub fn check_auth(state: State<'_, AppState>) -> bool {
    state.config.lock().unwrap().is_logged_in()
}

// ── Browse ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn search(
    query: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<SearchResults, String> {
    let api = state.api.lock().unwrap().clone();
    api.search_with_offset(&query, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_album(album_id: String, state: State<'_, AppState>) -> Result<Album, String> {
    let api = state.api.lock().unwrap().clone();
    api.get_album(&album_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_artist(
    artist_id: String,
    state: State<'_, AppState>,
) -> Result<ArtistDetail, String> {
    let api = state.api.lock().unwrap().clone();
    api.get_artist(&artist_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Album>, String> {
    let api = state.api.lock().unwrap().clone();
    api.get_favorite_albums(500).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let api = state.api.lock().unwrap().clone();
    api.get_user_playlists(500).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<Playlist, String> {
    let api = state.api.lock().unwrap().clone();
    api.get_playlist(&playlist_id)
        .await
        .map_err(|e| e.to_string())
}

// ── Playback ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn play_track(track_json: String, state: State<'_, AppState>) -> Result<(), String> {
    let track: Track = serde_json::from_str(&track_json).map_err(|e| e.to_string())?;
    let track_id = track.id.clone();
    let title = track.title.clone();
    let artist_name = track.artist_name().to_string();
    let album_title = track.album_title().to_string();
    let track_num = track.track_number;
    let duration = track.duration;

    // Check cache first
    if let Some(data) = state.cache.get(&track_id) {
        let mut player = state.player.lock().unwrap();
        player
            .play_audio(data, &title, &artist_name, duration)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Download with quality fallback
    let api = state.api.lock().unwrap().clone();
    let format_id = state.config.lock().unwrap().format_id();
    let data = api
        .download_track(&track_id, format_id)
        .await
        .map_err(|e| e.to_string())?;

    // Cache
    let meta = TrackMeta {
        artist: &artist_name,
        album: &album_title,
        track_number: track_num,
        title: &title,
    };
    state.cache.put(&track_id, &data, &meta);

    // Play
    let mut player = state.player.lock().unwrap();
    player
        .play_audio(data, &title, &artist_name, duration)
        .map_err(|e| e.to_string())?;
    player.set_quality(AudioQuality::from_format_id(format_id));
    Ok(())
}

#[tauri::command]
pub async fn play_queue_from(
    tracks_json: String,
    index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tracks: Vec<Track> = serde_json::from_str(&tracks_json).map_err(|e| e.to_string())?;
    if let Some(track) = tracks.get(index) {
        let track_json = serde_json::to_string(track).map_err(|e| e.to_string())?;
        *state.queue.lock().unwrap() = tracks;
        *state.queue_index.lock().unwrap() = index;
        play_track(track_json, state).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) {
    state.player.lock().unwrap().toggle_pause();
}

#[tauri::command]
pub async fn next_track(state: State<'_, AppState>) -> Result<(), String> {
    let (track_json, new_idx) = {
        let queue = state.queue.lock().unwrap();
        let mut idx = state.queue_index.lock().unwrap();
        let loop_mode = *state.loop_mode.lock().unwrap();

        let next = if *idx + 1 < queue.len() {
            Some(*idx + 1)
        } else if loop_mode == crate::state::LOOP_QUEUE {
            Some(0)
        } else {
            None
        };

        match next {
            Some(i) => {
                *idx = i;
                let track = queue.get(i).cloned();
                let json = track
                    .map(|t| serde_json::to_string(&t).unwrap_or_default())
                    .unwrap_or_default();
                (json, i)
            }
            None => return Ok(()),
        }
    };

    if !track_json.is_empty() {
        play_track(track_json, state).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn previous_track(state: State<'_, AppState>) -> Result<(), String> {
    let track_json = {
        let queue = state.queue.lock().unwrap();
        let mut idx = state.queue_index.lock().unwrap();
        if *idx > 0 {
            *idx -= 1;
            queue
                .get(*idx)
                .map(|t| serde_json::to_string(t).unwrap_or_default())
                .unwrap_or_default()
        } else {
            return Ok(());
        }
    };

    if !track_json.is_empty() {
        play_track(track_json, state).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn seek(position_secs: u64, state: State<'_, AppState>) -> bool {
    let mut player = state.player.lock().unwrap();
    if position_secs < player.elapsed_secs() {
        player.seek_backward(player.elapsed_secs() - position_secs)
    } else {
        player.seek_forward(position_secs - player.elapsed_secs())
    }
}

#[tauri::command]
pub fn set_volume(volume: f32, state: State<'_, AppState>) {
    state.player.lock().unwrap().set_volume(volume);
}

#[tauri::command]
pub fn get_player_state(state: State<'_, AppState>) -> PlayerState {
    let player = state.player.lock().unwrap();
    let queue = state.queue.lock().unwrap();
    let idx = *state.queue_index.lock().unwrap();
    let loop_mode = *state.loop_mode.lock().unwrap();

    PlayerState {
        is_playing: player.is_playing,
        is_loading: player.is_loading,
        title: player.current_track_title.clone(),
        artist: player.current_track_artist.clone(),
        duration: player.current_track_duration,
        elapsed: player.elapsed_secs(),
        progress: player.progress(),
        volume: player.volume,
        quality: player.quality().map(|q| q.label().to_string()),
        seekable: player.is_seekable(),
        queue_len: queue.len(),
        queue_index: idx,
        loop_mode,
    }
}

// ── Queue control ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn shuffle_queue(state: State<'_, AppState>) {
    let mut queue = state.queue.lock().unwrap();
    let mut idx = state.queue_index.lock().unwrap();
    *idx = qobuz_lib::shuffle(&mut queue, *idx);
}

#[tauri::command]
pub fn toggle_loop(state: State<'_, AppState>) -> u8 {
    let mut mode = state.loop_mode.lock().unwrap();
    *mode = match *mode {
        0 => 1,
        1 => 2,
        _ => 0,
    };
    *mode
}

// ── Favorites ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn add_favorite(album_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().unwrap().clone();
    api.favorite_add_album(&album_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_favorite(album_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().unwrap().clone();
    api.favorite_remove_album(&album_id)
        .await
        .map_err(|e| e.to_string())
}
