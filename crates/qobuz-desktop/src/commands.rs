// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Tauri commands — each function is callable from the Vue.js frontend.
//! All Mutex locks use poison recovery via `AppState::lock()`.

use crate::state::AppState;
use qobuz_lib::api::{Album, ArtistDetail, Genre, Playlist, SearchResults, Track};
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
    pub cover_url: Option<String>,
}

// ── Auth ─────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn login(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut api = AppState::lock(&state.api).clone();
    let token = api
        .login(&email, &password)
        .await
        .map_err(|e| e.to_string())?;
    *AppState::lock(&state.api) = api;
    let mut config = AppState::lock(&state.config);
    config.user_auth_token = Some(token.clone());
    config.email = Some(email);
    let _ = config.save();
    Ok(token)
}

#[tauri::command]
pub fn check_auth(state: State<'_, AppState>) -> bool {
    AppState::lock(&state.config).is_logged_in()
}

// ── Browse ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn search(
    query: String,
    limit: u32,
    offset: u32,
    state: State<'_, AppState>,
) -> Result<SearchResults, String> {
    let api = AppState::lock(&state.api).clone();
    api.search_with_offset(&query, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_album(album_id: String, state: State<'_, AppState>) -> Result<Album, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_album(&album_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_featured(
    #[allow(non_snake_case)]
    featuredType: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<Album>, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_featured_albums(&featuredType, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_featured_by_genre(
    #[allow(non_snake_case)]
    featuredType: String,
    #[allow(non_snake_case)]
    genreId: u32,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<Album>, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_featured_by_genre(&featuredType, genreId, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_genres(state: State<'_, AppState>) -> Result<Vec<Genre>, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_genres().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_artist(
    artist_id: String,
    state: State<'_, AppState>,
) -> Result<ArtistDetail, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_artist(&artist_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_favorites(state: State<'_, AppState>) -> Result<Vec<Album>, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_favorite_albums(500)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playlists(state: State<'_, AppState>) -> Result<Vec<Playlist>, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_user_playlists(500)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playlist(
    playlist_id: String,
    state: State<'_, AppState>,
) -> Result<Playlist, String> {
    let api = AppState::lock(&state.api).clone();
    api.get_playlist(&playlist_id)
        .await
        .map_err(|e| e.to_string())
}

// ── Playback ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn play_track(
    track_json: String,
    cover_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let track: Track = serde_json::from_str(&track_json).map_err(|e| e.to_string())?;
    let track_id = track.id.clone();
    let title = track.title.clone();
    let artist_name = track.artist_name().to_string();
    let album_title = track.album_title().to_string();
    let track_num = track.track_number;
    let duration = track.duration;

    // Store cover URL
    *AppState::lock(&state.current_cover_url) = cover_url;

    // Check cache first
    if let Some(data) = state.cache.get(&track_id) {
        let mut player = AppState::lock(&state.player);
        player
            .play_audio(data, &title, &artist_name, duration)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Download with quality fallback
    let api = AppState::lock(&state.api).clone();
    let format_id = AppState::lock(&state.config).format_id();
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
    let mut player = AppState::lock(&state.player);
    player
        .play_audio(data, &title, &artist_name, duration)
        .map_err(|e| e.to_string())?;
    player.set_quality(AudioQuality::from_format_id(format_id));

    // Track in recent history
    let recent = crate::state::RecentTrack {
        title: title.clone(),
        artist: artist_name.clone(),
        album_id: track.album.as_ref().map(|a| a.id.clone()).unwrap_or_default(),
        cover_url: AppState::lock(&state.current_cover_url).clone(),
    };
    let mut history = AppState::lock(&state.recent_tracks);
    history.retain(|r| r.title != recent.title || r.artist != recent.artist);
    history.insert(0, recent);
    history.truncate(50);

    Ok(())
}

#[tauri::command]
pub async fn play_queue_from(
    tracks_json: String,
    index: usize,
    cover_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tracks: Vec<Track> = serde_json::from_str(&tracks_json).map_err(|e| e.to_string())?;
    if let Some(track) = tracks.get(index) {
        let track_json = serde_json::to_string(track).map_err(|e| e.to_string())?;
        *AppState::lock(&state.queue) = tracks;
        *AppState::lock(&state.queue_index) = index;
        play_track(track_json, cover_url, state).await?;
    }
    Ok(())
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>) {
    AppState::lock(&state.player).toggle_pause();
}

#[tauri::command]
pub async fn next_track(state: State<'_, AppState>) -> Result<(), String> {
    let track_json = {
        let queue = AppState::lock(&state.queue);
        let mut idx = AppState::lock(&state.queue_index);
        let loop_mode = *AppState::lock(&state.loop_mode);

        let next = if loop_mode == crate::state::LOOP_TRACK {
            Some(*idx) // Replay same track
        } else if *idx + 1 < queue.len() {
            Some(*idx + 1)
        } else if loop_mode == crate::state::LOOP_QUEUE {
            Some(0)
        } else {
            None
        };

        match next {
            Some(i) => {
                *idx = i;
                match queue.get(i) {
                    Some(t) => serde_json::to_string(t).map_err(|e| e.to_string())?,
                    None => return Ok(()),
                }
            }
            None => return Ok(()),
        }
    };

    let cover = AppState::lock(&state.current_cover_url).clone();
    play_track(track_json, cover, state).await?;
    Ok(())
}

#[tauri::command]
pub async fn previous_track(state: State<'_, AppState>) -> Result<(), String> {
    let track_json = {
        let queue = AppState::lock(&state.queue);
        let mut idx = AppState::lock(&state.queue_index);
        if *idx > 0 {
            *idx -= 1;
            match queue.get(*idx) {
                Some(t) => serde_json::to_string(t).map_err(|e| e.to_string())?,
                None => return Ok(()),
            }
        } else {
            return Ok(());
        }
    };

    let cover = AppState::lock(&state.current_cover_url).clone();
    play_track(track_json, cover, state).await?;
    Ok(())
}

#[tauri::command]
pub fn seek(position_secs: u64, state: State<'_, AppState>) -> bool {
    let mut player = AppState::lock(&state.player);
    let elapsed = player.elapsed_secs();
    if position_secs < elapsed {
        player.seek_backward(elapsed - position_secs)
    } else {
        player.seek_forward(position_secs - elapsed)
    }
}

#[tauri::command]
pub fn set_volume(volume: f32, state: State<'_, AppState>) {
    AppState::lock(&state.player).set_volume(volume);
}

#[tauri::command]
pub fn get_player_state(state: State<'_, AppState>) -> PlayerState {
    let player = AppState::lock(&state.player);
    let queue = AppState::lock(&state.queue);
    let idx = *AppState::lock(&state.queue_index);
    let loop_mode = *AppState::lock(&state.loop_mode);
    let cover_url = AppState::lock(&state.current_cover_url).clone();

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
        cover_url,
    }
}

// ── History ──────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_recent(state: State<'_, AppState>) -> Vec<crate::state::RecentTrack> {
    AppState::lock(&state.recent_tracks).clone()
}

// ── Queue control ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn shuffle_queue(state: State<'_, AppState>) {
    let mut queue = AppState::lock(&state.queue);
    let mut idx = AppState::lock(&state.queue_index);
    *idx = qobuz_lib::shuffle(&mut queue, *idx);
}

#[tauri::command]
pub fn toggle_loop(state: State<'_, AppState>) -> u8 {
    let mut mode = AppState::lock(&state.loop_mode);
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
    let api = AppState::lock(&state.api).clone();
    api.favorite_add_album(&album_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_favorite(
    album_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let api = AppState::lock(&state.api).clone();
    api.favorite_remove_album(&album_id)
        .await
        .map_err(|e| e.to_string())
}
