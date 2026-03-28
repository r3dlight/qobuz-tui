// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Tauri commands — each function is callable from the Vue.js frontend.

use crate::state::AppState;
use qobuz_lib::api::{Album, ArtistDetail, Playlist, Track};
use qobuz_lib::player::AudioQuality;
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
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

#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut api = state.api.lock().unwrap();
    let token = api.login(&email, &password).await.map_err(|e| e.to_string())?;
    let mut config = state.config.lock().unwrap();
    config.user_auth_token = Some(token.clone());
    config.email = Some(email);
    let _ = config.save();
    Ok(token)
}

#[tauri::command]
pub async fn search(
    query: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let api = state.api.lock().unwrap().clone();
    let results = api.search_with_offset(&query, limit, offset).await.map_err(|e| e.to_string())?;
    serde_json::to_value(results).map_err(|e| e.to_string())
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
    api.get_playlist(&playlist_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn play_track(track_json: String, state: State<'_, AppState>) -> Result<(), String> {
    let track: Track = serde_json::from_str(&track_json).map_err(|e| e.to_string())?;
    let api = state.api.lock().unwrap().clone();
    let format_id = state.config.lock().unwrap().format_id();
    let data = api
        .download_track(&track.id, format_id)
        .await
        .map_err(|e| e.to_string())?;
    let mut player = state.player.lock().unwrap();
    if let Some(p) = player.as_mut() {
        p.play_audio(data, &track.title, track.artist_name(), track.duration)
            .map_err(|e| e.to_string())?;
        p.set_quality(AudioQuality::from_format_id(format_id));
    }
    Ok(())
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) {
    if let Some(p) = state.player.lock().unwrap().as_mut() {
        p.toggle_pause();
    }
}

#[tauri::command]
pub fn next_track(state: State<'_, AppState>) -> Result<(), String> {
    let mut idx = state.queue_index.lock().unwrap();
    let queue = state.queue.lock().unwrap();
    if *idx + 1 < queue.len() {
        *idx += 1;
    }
    Ok(())
}

#[tauri::command]
pub fn previous_track(state: State<'_, AppState>) -> Result<(), String> {
    let mut idx = state.queue_index.lock().unwrap();
    if *idx > 0 {
        *idx -= 1;
    }
    Ok(())
}

#[tauri::command]
pub fn set_volume(volume: f32, state: State<'_, AppState>) {
    if let Some(p) = state.player.lock().unwrap().as_mut() {
        p.set_volume(volume);
    }
}

#[tauri::command]
pub fn get_player_state(state: State<'_, AppState>) -> PlayerState {
    let player = state.player.lock().unwrap();
    let queue = state.queue.lock().unwrap();
    let idx = *state.queue_index.lock().unwrap();
    let loop_mode = *state.loop_mode.lock().unwrap();

    match player.as_ref() {
        Some(p) => PlayerState {
            is_playing: p.is_playing,
            is_loading: p.is_loading,
            title: p.current_track_title.clone(),
            artist: p.current_track_artist.clone(),
            duration: p.current_track_duration,
            elapsed: p.elapsed_secs(),
            progress: p.progress(),
            volume: p.volume,
            quality: p.quality().map(|q| q.label().to_string()),
            seekable: p.is_seekable(),
            queue_len: queue.len(),
            queue_index: idx,
            loop_mode,
        },
        None => PlayerState {
            is_playing: false,
            is_loading: false,
            title: None,
            artist: None,
            duration: 0,
            elapsed: 0,
            progress: 0.0,
            volume: 0.8,
            quality: None,
            seekable: false,
            queue_len: queue.len(),
            queue_index: idx,
            loop_mode,
        },
    }
}

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

#[tauri::command]
pub async fn add_favorite(album_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().unwrap().clone();
    api.favorite_add_album(&album_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_favorite(album_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().unwrap().clone();
    api.favorite_remove_album(&album_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn download_album(tracks_json: String, state: State<'_, AppState>) -> Result<(), String> {
    let tracks: Vec<Track> = serde_json::from_str(&tracks_json).map_err(|e| e.to_string())?;
    let api = state.api.lock().unwrap().clone();
    let format_id = state.config.lock().unwrap().format_id();
    for track in &tracks {
        if !state.cache.has(&track.id) {
            if let Ok(data) = api.download_track(&track.id, format_id).await {
                let meta = qobuz_lib::cache::TrackMeta {
                    artist: track.artist_name(),
                    album: track.album_title(),
                    track_number: track.track_number,
                    title: &track.title,
                };
                state.cache.put(&track.id, &data, &meta);
            }
        }
    }
    Ok(())
}
