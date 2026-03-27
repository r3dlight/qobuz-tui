// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! TUI application state, play queue, input handling, and async message processing.
//!
//! The `App` struct owns all UI and domain state. Async operations (API calls,
//! audio downloads) communicate via [`AppMessage`] through an unbounded channel.

use qobuz_lib::api::{self, Album, Playlist, QobuzClient, Track};
use qobuz_lib::cache::{AudioCache, TrackMeta};
use qobuz_lib::config::Config;
use qobuz_lib::player::{AudioQuality, Player};
use qobuz_lib::session::{self, SessionTrack};
use qobuz_lib::stream::{self, StreamingBuffer};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

/// Quality fallback chain for a given preferred format_id.
fn format_fallback_chain(preferred: u32) -> &'static [u32] {
    match preferred {
        27 => &[27, 7, 6, 5],
        7 => &[7, 6, 5],
        6 => &[6, 5],
        _ => &[5],
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Main,
    AlbumView,
    PlaylistView,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Search,
    Favorites,
    Playlists,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoginField {
    Email,
    Password,
    Submit,
}

pub enum AppMessage {
    CredentialsFetched(String, String),
    CredentialsError(String),
    LoginSuccess(String, String, String),
    LoginError(String),
    SearchResults(Vec<Track>, Vec<Album>),
    SearchError(String),
    FavoritesResults(Vec<Album>),
    FavoritesError(String),
    /// Streaming buffer ready for playback (progressive download started)
    /// buffer, title, artist, duration, format_id
    StreamReady(StreamingBuffer, String, String, u64, u32),
    /// Temporary status message (e.g. quality fallback notification)
    StatusMessage(String),
    /// Download error
    AudioError(String),
    AlbumLoaded(Album),
    AlbumError(String),
    DownloadProgress(usize, usize),
    DownloadDone,
    DownloadError(String),
    /// Stream download completed — enable seek on current track
    StreamCached(Vec<u8>, String), // data, track_id
    FavoriteToggled(String, bool),
    FavoriteToggleError(String),
    PlaylistsLoaded(Vec<api::Playlist>),
    PlaylistsError(String),
    PlaylistLoaded(api::Playlist),
    PlaylistError(String),
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub tab: Tab,

    // Login
    pub login_fields: [String; 2],
    pub login_focus: LoginField,
    pub login_error: Option<String>,
    pub login_loading: bool,
    pub login_status: Option<String>,

    // Search
    pub search_query: String,
    pub search_tracks: Vec<Track>,
    pub search_albums: Vec<Album>,
    pub search_selected: usize,
    pub search_scroll: usize,
    pub search_mode: SearchMode,

    // Favorites (albums)
    pub favorite_albums: Vec<Album>,
    pub favorites_selected: usize,
    pub favorites_scroll: usize,
    pub favorites_loaded: bool,

    // Album view
    pub album: Option<Album>,
    pub album_tracks: Vec<Track>,
    pub album_selected: usize,
    pub album_scroll: usize,

    // Playlists
    pub playlists: Vec<Playlist>,
    pub playlists_selected: usize,
    pub playlists_scroll: usize,
    pub playlists_loaded: bool,
    pub playlist_tracks: Vec<Track>,
    pub playlist_name: Option<String>,
    pub playlist_selected: usize,
    pub playlist_scroll: usize,

    // Play queue
    pub queue: Vec<Track>,
    pub queue_index: usize,
    pub loop_mode: LoopMode,
    pub next_track_prefetched: bool,

    // Player
    pub player: Player,


    // Cache
    pub cache: AudioCache,

    // Status
    pub status_message: Option<String>,
    status_expires: Option<std::time::Instant>,

    // API client
    pub api: QobuzClient,
    pub config: Config,
    pub tx: mpsc::UnboundedSender<AppMessage>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopMode {
    Off,
    Track,
    Queue,
}

impl LoopMode {
    pub fn next(&self) -> Self {
        match self {
            LoopMode::Off => LoopMode::Track,
            LoopMode::Track => LoopMode::Queue,
            LoopMode::Queue => LoopMode::Off,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            LoopMode::Off => "",
            LoopMode::Track => "[LOOP:TRACK]",
            LoopMode::Queue => "[LOOP:ALL]",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Tracks,
    Albums,
}

impl App {
    pub fn new(
        config: Config,
        player: Player,
        tx: mpsc::UnboundedSender<AppMessage>,
    ) -> Self {
        let mut api = QobuzClient::new(&config.app_id, &config.app_secret);
        if let Some(token) = &config.user_auth_token {
            api.set_token(token.clone());
        }

        let screen = if config.is_logged_in() {
            Screen::Main
        } else {
            Screen::Login
        };

        let mut login_fields = [String::new(), String::new()];
        if let Some(email) = &config.email {
            login_fields[0] = email.clone();
        }

        if !config.has_app_credentials() {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match api::fetch_app_credentials().await {
                    Ok((id, secret)) => {
                        tx_clone.send(AppMessage::CredentialsFetched(id, secret)).ok();
                    }
                    Err(e) => {
                        tx_clone.send(AppMessage::CredentialsError(e.to_string())).ok();
                    }
                }
            });
        }

        let login_status = if !config.has_app_credentials() {
            Some("Fetching Qobuz API credentials...".to_string())
        } else {
            None
        };

        // Restore session
        let saved = session::load();
        let mut player = player;
        player.set_volume(saved.volume.clamp(0.0, 1.0));
        let restored_queue: Vec<Track> = saved.queue.iter().map(|t| t.to_track()).collect();
        let loop_mode = match saved.loop_mode {
            1 => LoopMode::Track,
            2 => LoopMode::Queue,
            _ => LoopMode::Off,
        };

        Self {
            screen,
            should_quit: false,
            tab: Tab::Search,
            login_fields,
            login_focus: LoginField::Email,
            login_error: None,
            login_loading: false,
            login_status,
            search_query: String::new(),
            search_tracks: Vec::new(),
            search_albums: Vec::new(),
            search_selected: 0,
            search_scroll: 0,
            search_mode: SearchMode::Tracks,
            favorite_albums: Vec::new(),
            favorites_selected: 0,
            favorites_scroll: 0,
            favorites_loaded: false,
            album: None,
            album_tracks: Vec::new(),
            album_selected: 0,
            album_scroll: 0,
            playlists: Vec::new(),
            playlists_selected: 0,
            playlists_scroll: 0,
            playlists_loaded: false,
            playlist_tracks: Vec::new(),
            playlist_name: None,
            playlist_selected: 0,
            playlist_scroll: 0,
            queue: restored_queue,
            queue_index: saved.queue_index,
            loop_mode,
            next_track_prefetched: false,
            player,
            cache: AudioCache::new(),
            status_message: None,
            status_expires: None,
            api,
            config,
            tx,
        }
    }

    pub fn handle_message(&mut self, msg: AppMessage) {
        match msg {
            AppMessage::CredentialsFetched(app_id, app_secret) => {
                self.config.app_id = app_id;
                self.config.app_secret = app_secret;
                let _ = self.config.save();
                self.api = QobuzClient::new(&self.config.app_id, &self.config.app_secret);
                self.login_status = Some("Ready to login".to_string());
            }
            AppMessage::CredentialsError(err) => {
                self.login_status = None;
                self.login_error = Some(format!(
                    "Could not fetch API credentials: {}. Set app_id/app_secret manually in {:?}",
                    err,
                    Config::path()
                ));
            }
            AppMessage::LoginSuccess(token, app_id, app_secret) => {
                self.login_loading = false;
                self.config.app_id = app_id;
                self.config.app_secret = app_secret;
                self.config.email = Some(self.login_fields[0].clone());
                self.config.user_auth_token = Some(token);
                let _ = self.config.save();
                self.api = QobuzClient::new(&self.config.app_id, &self.config.app_secret);
                if let Some(t) = &self.config.user_auth_token {
                    self.api.set_token(t.clone());
                }
                self.screen = Screen::Main;
                self.status_message = Some("Logged in successfully".to_string());
            }
            AppMessage::LoginError(err) => {
                self.login_loading = false;
                self.login_error = Some(err);
            }
            AppMessage::SearchResults(tracks, albums) => {
                self.search_tracks = tracks;
                self.search_albums = albums;
                self.search_selected = 0;
                self.search_scroll = 0;
                self.status_message = None;
            }
            AppMessage::SearchError(err) => {
                self.status_message = Some(format!("Search error: {}", err));
            }
            AppMessage::FavoritesResults(albums) => {
                self.favorite_albums = albums;
                self.favorites_selected = 0;
                self.favorites_scroll = 0;
                self.favorites_loaded = true;
                self.status_message = None;
            }
            AppMessage::FavoritesError(err) => {
                self.status_message = Some(format!("Favorites error: {}", err));
            }
            AppMessage::StreamReady(buffer, title, artist, duration, format_id) => {
                if self.player.play_streaming(buffer, &title, &artist, duration).is_ok() {
                    self.player.current_quality = AudioQuality::from_format_id(format_id);
                }
                // Decoder failure is silent — format fallback will send another StreamReady.
            }
            AppMessage::StatusMessage(msg) => {
                self.set_temp_status(msg);
            }
            AppMessage::AudioError(err) => {
                self.player.is_loading = false;
                self.status_message = Some(format!("Audio error: {}", err));
            }
            AppMessage::AlbumLoaded(album) => {
                self.album_tracks = album
                    .tracks
                    .as_ref()
                    .map(|t| t.items.clone())
                    .unwrap_or_default();
                self.album = Some(album);
                self.album_selected = 0;
                self.album_scroll = 0;
                self.screen = Screen::AlbumView;
                self.status_message = None;
            }
            AppMessage::AlbumError(err) => {
                self.status_message = Some(format!("Album error: {}", err));
            }
            AppMessage::FavoriteToggled(album_id, added) => {
                if added {
                    self.status_message = Some("Added to favorites".to_string());
                } else {
                    self.status_message = Some("Removed from favorites".to_string());
                    // Remove from local list
                    self.favorite_albums.retain(|a| a.id != album_id);
                    if self.favorites_selected > 0
                        && self.favorites_selected >= self.favorite_albums.len()
                    {
                        self.favorites_selected = self.favorite_albums.len().saturating_sub(1);
                    }
                }
            }
            AppMessage::FavoriteToggleError(err) => {
                self.status_message = Some(format!("Favorite error: {}", err));
            }
            AppMessage::StreamCached(data, _track_id) => {
                // Download finished while streaming — enable seek
                self.player.cached_data = Some(data);
                self.set_temp_status("Track cached — seek enabled (,/;)".to_string());
            }
            AppMessage::PlaylistsLoaded(playlists) => {
                self.playlists = playlists;
                self.playlists_selected = 0;
                self.playlists_scroll = 0;
                self.playlists_loaded = true;
                self.status_message = None;
            }
            AppMessage::PlaylistsError(err) => {
                self.status_message = Some(format!("Playlists error: {}", err));
            }
            AppMessage::PlaylistLoaded(playlist) => {
                self.playlist_tracks = playlist
                    .tracks
                    .as_ref()
                    .map(|t| t.items.clone())
                    .unwrap_or_default();
                self.playlist_name = Some(playlist.name);
                self.playlist_selected = 0;
                self.playlist_scroll = 0;
                self.screen = Screen::PlaylistView;
                self.status_message = None;
            }
            AppMessage::PlaylistError(err) => {
                self.status_message = Some(format!("Playlist error: {}", err));
            }
            AppMessage::DownloadProgress(done, total) => {
                self.status_message = Some(format!("Downloading album: {}/{} tracks", done, total));
            }
            AppMessage::DownloadDone => {
                self.status_message = Some("Album downloaded to cache".to_string());
            }
            AppMessage::DownloadError(err) => {
                self.status_message = Some(format!("Download error: {}", err));
            }
        }
    }

    fn is_typing(&self) -> bool {
        self.screen == Screen::Login
            || (self.screen == Screen::Main && self.tab == Tab::Search)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        if key.code == KeyCode::Esc && self.screen != Screen::Login {
            self.should_quit = true;
            return;
        }

        // Global playback controls (not on login screen)
        if self.screen != Screen::Login {
            match key.code {
                KeyCode::Left => {
                    self.player.volume_down();
                    return;
                }
                KeyCode::Right => {
                    self.player.volume_up();
                    return;
                }
                KeyCode::Tab => {
                    self.screen = Screen::Main;
                    self.tab = match self.tab {
                        Tab::Search => Tab::Favorites,
                        Tab::Favorites => Tab::Playlists,
                        Tab::Playlists => Tab::Search,
                    };
                    if self.tab == Tab::Favorites && !self.favorites_loaded { self.do_load_favorites(); }
                    if self.tab == Tab::Playlists && !self.playlists_loaded { self.do_load_playlists(); }
                    return;
                }
                _ => {}
            }
        }

        // Controls that only work when not typing in search bar
        if !self.is_typing() {
            match key.code {
                KeyCode::Char('p') => {
                    self.player.toggle_pause();
                    return;
                }
                KeyCode::Char('n') => {
                    self.play_next();
                    return;
                }
                KeyCode::Char('N') => {
                    self.play_previous();
                    return;
                }
                KeyCode::Char('r') => {
                    self.loop_mode = self.loop_mode.next();
                    return;
                }
                KeyCode::Char(',') => {
                    if !self.player.seek_backward(10) {
                        let err = self.player.last_seek_error().unwrap_or("unknown").to_string();
                        self.set_temp_status(format!("Seek: {}", err));
                    }
                    return;
                }
                KeyCode::Char(';') => {
                    if !self.player.seek_forward(10) {
                        let err = self.player.last_seek_error().unwrap_or("unknown").to_string();
                        self.set_temp_status(format!("Seek: {}", err));
                    }
                    return;
                }
                _ => {}
            }
        }

        match self.screen {
            Screen::Login => self.handle_login_key(key),
            Screen::Main => self.handle_main_key(key),
            Screen::AlbumView => self.handle_album_key(key),
            Screen::PlaylistView => self.handle_playlist_view_key(key),
        }

    }

    fn handle_login_key(&mut self, key: KeyEvent) {
        if self.login_loading {
            return;
        }
        match key.code {
            KeyCode::Tab => {
                self.login_focus = match self.login_focus {
                    LoginField::Email => LoginField::Password,
                    LoginField::Password => LoginField::Submit,
                    LoginField::Submit => LoginField::Email,
                };
            }
            KeyCode::BackTab => {
                self.login_focus = match self.login_focus {
                    LoginField::Email => LoginField::Submit,
                    LoginField::Password => LoginField::Email,
                    LoginField::Submit => LoginField::Password,
                };
            }
            KeyCode::Enter => {
                if self.login_focus == LoginField::Submit {
                    self.do_login();
                } else {
                    self.login_focus = match self.login_focus {
                        LoginField::Email => LoginField::Password,
                        LoginField::Password => LoginField::Submit,
                        LoginField::Submit => LoginField::Submit,
                    };
                }
            }
            KeyCode::Char(c) => {
                let idx = self.login_field_index();
                if idx < 2 {
                    self.login_fields[idx].push(c);
                }
            }
            KeyCode::Backspace => {
                let idx = self.login_field_index();
                if idx < 2 {
                    self.login_fields[idx].pop();
                }
            }
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) {
        match self.tab {
            Tab::Search => self.handle_search_key(key),
            Tab::Favorites => self.handle_favorites_key(key),
            Tab::Playlists => self.handle_playlists_key(key),
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if self.current_list_len() > 0 {
                    self.select_search_item();
                } else if !self.search_query.is_empty() {
                    self.do_search();
                }
            }
            KeyCode::Up if self.search_selected > 0 => {
                self.search_selected -= 1;
            }
            KeyCode::Down if self.search_selected + 1 < self.current_list_len() => {
                self.search_selected += 1;
            }
            KeyCode::BackTab => {
                self.search_mode = match self.search_mode {
                    SearchMode::Tracks => SearchMode::Albums,
                    SearchMode::Albums => SearchMode::Tracks,
                };
                self.search_selected = 0;
                self.search_scroll = 0;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.search_tracks.clear();
                self.search_albums.clear();
                self.search_selected = 0;
                self.search_scroll = 0;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_query.is_empty() {
                    self.search_tracks.clear();
                    self.search_albums.clear();
                    self.search_selected = 0;
                    self.search_scroll = 0;
                }
            }
            _ => {}
        }
    }

    fn handle_favorites_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up if self.favorites_selected > 0 => {
                self.favorites_selected -= 1;
            }
            KeyCode::Down if self.favorites_selected + 1 < self.favorite_albums.len() => {
                self.favorites_selected += 1;
            }
            KeyCode::Enter => {
                if let Some(album) = self.favorite_albums.get(self.favorites_selected).cloned() {
                    self.open_album(album.id);
                }
            }
            KeyCode::Char('x') => {
                // Remove from favorites
                if let Some(album) = self.favorite_albums.get(self.favorites_selected).cloned() {
                    self.toggle_favorite(&album.id, false);
                }
            }
            _ => {}
        }
    }

    fn handle_playlists_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up if self.playlists_selected > 0 => {
                self.playlists_selected -= 1;
            }
            KeyCode::Down if self.playlists_selected + 1 < self.playlists.len() => {
                self.playlists_selected += 1;
            }
            KeyCode::Enter => {
                if let Some(pl) = self.playlists.get(self.playlists_selected).cloned() {
                    self.open_playlist(pl.id);
                }
            }
            _ => {}
        }
    }

    fn handle_playlist_view_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up if self.playlist_selected > 0 => {
                self.playlist_selected -= 1;
            }
            KeyCode::Down if self.playlist_selected + 1 < self.playlist_tracks.len() => {
                self.playlist_selected += 1;
            }
            KeyCode::Enter => {
                self.queue = self.playlist_tracks.clone();
                self.queue_index = self.playlist_selected;
                if let Some(track) = self.queue.get(self.queue_index) {
                    self.play_track(track.clone());
                }
            }
            KeyCode::Backspace => self.screen = Screen::Main,
            _ => {}
        }
    }

    fn handle_album_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up if self.album_selected > 0 => {
                self.album_selected -= 1;
            }
            KeyCode::Down if self.album_selected + 1 < self.album_tracks.len() => {
                self.album_selected += 1;
            }
            KeyCode::Enter => {
                self.play_album_from(self.album_selected);
            }
            KeyCode::Char('d') => {
                self.download_album();
            }
            KeyCode::Char('f') => {
                // Add current album to favorites
                if let Some(album) = &self.album {
                    let album_id = album.id.clone();
                    self.toggle_favorite(&album_id, true);
                }
            }
            KeyCode::Backspace => self.screen = Screen::Main,
            _ => {}
        }
    }

    fn toggle_favorite(&mut self, album_id: &str, add: bool) {
        let album_id = album_id.to_string();
        let tx = self.tx.clone();
        let api = self.api.clone();
        self.status_message = Some(if add {
            "Adding to favorites...".to_string()
        } else {
            "Removing from favorites...".to_string()
        });
        tokio::spawn(async move {
            let result = if add {
                api.favorite_add_album(&album_id).await
            } else {
                api.favorite_remove_album(&album_id).await
            };
            match result {
                Ok(()) => { tx.send(AppMessage::FavoriteToggled(album_id, add)).ok(); }
                Err(e) => { tx.send(AppMessage::FavoriteToggleError(e.to_string())).ok(); }
            }
        });
    }

    fn download_album(&mut self) {
        if self.album_tracks.is_empty() {
            return;
        }
        let album_name = self.album.as_ref().map(|a| a.title.clone()).unwrap_or_default();
        let album_artist = self
            .album
            .as_ref()
            .and_then(|a| a.artist.as_ref())
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        self.status_message = Some(format!("Downloading \"{}\"...", album_name));

        let tracks = self.album_tracks.clone();
        let total = tracks.len();
        let tx = self.tx.clone();
        let api = self.api.clone();
        let cache = self.cache.clone();
        let format_id = self.config.format_id();

        tokio::spawn(async move {
            let mut failed = 0usize;
            for (i, track) in tracks.iter().enumerate() {
                // Skip if already cached
                if cache.has(&track.id) {
                    tx.send(AppMessage::DownloadProgress(i + 1, total)).ok();
                    continue;
                }

                // Retry up to 3 times with increasing delay
                let mut success = false;
                for attempt in 0..3 {
                    if attempt > 0 {
                        tokio::time::sleep(std::time::Duration::from_secs(2 * attempt as u64)).await;
                    }
                    match download_track(&api, &track.id, format_id).await {
                        Ok(data) => {
                            let meta = TrackMeta {
                                artist: &album_artist,
                                album: &album_name,
                                track_number: track.track_number,
                                title: &track.title,
                            };
                            cache.put(&track.id, &data, &meta);
                            success = true;
                            break;
                        }
                        Err(_) if attempt < 2 => continue,
                        Err(e) => {
                            tx.send(AppMessage::DownloadProgress(i + 1, total)).ok();
                            tx.send(AppMessage::DownloadError(format!(
                                "Skipped \"{}\": {}", track.title, e
                            ))).ok();
                            break;
                        }
                    }
                }
                if success {
                    tx.send(AppMessage::DownloadProgress(i + 1, total)).ok();
                } else {
                    failed += 1;
                }
            }
            if failed > 0 {
                tx.send(AppMessage::DownloadDone).ok();
                tx.send(AppMessage::DownloadError(format!(
                    "{} track(s) failed to download", failed
                ))).ok();
            } else {
                tx.send(AppMessage::DownloadDone).ok();
            }
        });
    }

    fn login_field_index(&self) -> usize {
        match self.login_focus {
            LoginField::Email => 0,
            LoginField::Password => 1,
            LoginField::Submit => 2,
        }
    }

    fn current_list_len(&self) -> usize {
        match self.search_mode {
            SearchMode::Tracks => self.search_tracks.len(),
            SearchMode::Albums => self.search_albums.len(),
        }
    }

    fn select_search_item(&mut self) {
        match self.search_mode {
            SearchMode::Tracks => {
                // Build queue from search results, play from selected
                let idx = self.search_selected;
                self.queue = self.search_tracks.clone();
                self.queue_index = idx;
                if let Some(track) = self.queue.get(idx) {
                    self.play_track(track.clone());
                }
            }
            SearchMode::Albums => {
                if let Some(album) = self.search_albums.get(self.search_selected).cloned() {
                    self.open_album(album.id);
                }
            }
        }
    }

    fn play_album_from(&mut self, index: usize) {
        // Build queue from album tracks, play from index
        self.queue = self.album_tracks.clone();
        self.queue_index = index;
        if let Some(track) = self.queue.get(index) {
            self.play_track(track.clone());
        }
    }

    pub fn play_next(&mut self) {
        if self.queue_index + 1 < self.queue.len() {
            self.queue_index += 1;
            if let Some(track) = self.queue.get(self.queue_index).cloned() {
                self.play_track(track);
            }
        }
    }

    pub fn play_previous(&mut self) {
        if self.queue_index > 0 {
            self.queue_index -= 1;
            if let Some(track) = self.queue.get(self.queue_index).cloned() {
                self.play_track(track);
            }
        }
    }

    fn play_track(&mut self, track: Track) {
        let title = track.title.clone();
        let artist = track.artist_name().to_string();
        let album_title = track.album_title().to_string();
        let track_num = track.track_number;
        let duration = track.duration;
        let track_id = track.id.clone();
        let format_id = self.config.format_id();

        // Check cache first — instant playback (seekable)
        if self.cache.has(&track_id)
            && let Some(data) = self.cache.get(&track_id)
        {
            if let Err(e) = self.player.play_audio(data, &title, &artist, duration) {
                self.status_message = Some(format!("Playback error: {}", e));
            }
            return;
        }

        self.player.set_loading(&title, &artist);

        let tx = self.tx.clone();
        let api = self.api.clone();
        let cache = self.cache.clone();

        // Stream: get URL, start download, send buffer for immediate playback
        tokio::spawn(async move {
            match stream_track(&api, &track_id, format_id, &tx, &title, &artist, duration).await {
                Ok(data) => {
                    let meta = TrackMeta {
                        artist: &artist,
                        album: &album_title,
                        track_number: track_num,
                        title: &title,
                    };
                    cache.put(&track_id, &data, &meta);
                    // Enable seek on the currently playing track
                    tx.send(AppMessage::StreamCached(data, track_id)).ok();
                }
                Err(e) => {
                    tx.send(AppMessage::AudioError(e.to_string())).ok();
                }
            }
        });
    }

    fn open_album(&mut self, album_id: String) {
        self.status_message = Some("Loading album...".to_string());
        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match api.get_album(&album_id).await {
                Ok(album) => { tx.send(AppMessage::AlbumLoaded(album)).ok(); }
                Err(e) => { tx.send(AppMessage::AlbumError(e.to_string())).ok(); }
            }
        });
    }

    fn do_login(&mut self) {
        if !self.config.has_app_credentials() {
            self.login_error = Some(format!(
                "API credentials not available yet. Set app_id and app_secret in {:?}",
                Config::path()
            ));
            return;
        }
        self.login_loading = true;
        self.login_error = None;
        self.login_status = Some("Logging in...".to_string());

        let app_id = self.config.app_id.clone();
        let app_secret = self.config.app_secret.clone();
        let email = self.login_fields[0].clone();
        let password = self.login_fields[1].clone();

        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut client = QobuzClient::new(&app_id, &app_secret);
            match client.login(&email, &password).await {
                Ok(token) => { tx.send(AppMessage::LoginSuccess(token, app_id, app_secret)).ok(); }
                Err(e) => { tx.send(AppMessage::LoginError(e.to_string())).ok(); }
            }
        });
    }

    fn do_search(&mut self) {
        let query = self.search_query.clone();
        self.status_message = Some("Searching...".to_string());
        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match api.search(&query, 50).await {
                Ok(results) => {
                    let tracks = results.tracks.map(|t| t.items).unwrap_or_default();
                    let albums = results.albums.map(|a| a.items).unwrap_or_default();
                    tx.send(AppMessage::SearchResults(tracks, albums)).ok();
                }
                Err(e) => { tx.send(AppMessage::SearchError(e.to_string())).ok(); }
            }
        });
    }

    fn do_load_playlists(&mut self) {
        self.status_message = Some("Loading playlists...".to_string());
        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match api.get_user_playlists(500).await {
                Ok(playlists) => { tx.send(AppMessage::PlaylistsLoaded(playlists)).ok(); }
                Err(e) => { tx.send(AppMessage::PlaylistsError(e.to_string())).ok(); }
            }
        });
    }

    fn open_playlist(&mut self, playlist_id: String) {
        self.status_message = Some("Loading playlist...".to_string());
        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match api.get_playlist(&playlist_id).await {
                Ok(pl) => { tx.send(AppMessage::PlaylistLoaded(pl)).ok(); }
                Err(e) => { tx.send(AppMessage::PlaylistError(e.to_string())).ok(); }
            }
        });
    }


    /// Pre-fetch next track for gapless playback (~15s before end).
    fn prefetch_next(&mut self) {
        if self.next_track_prefetched {
            return;
        }
        let next_idx = if self.queue_index + 1 < self.queue.len() {
            self.queue_index + 1
        } else if self.loop_mode == LoopMode::Queue {
            0
        } else {
            return;
        };
        if let Some(track) = self.queue.get(next_idx) {
            if self.cache.has(&track.id) {
                return; // Already cached, instant playback
            }
            // Pre-download next track in background
            let track_id = track.id.clone();
            let format_id = self.config.format_id();
            let api = self.api.clone();
            let cache = self.cache.clone();
            let title = track.title.clone();
            let artist = track.artist_name().to_string();
            let album = track.album_title().to_string();
            let track_num = track.track_number;
            self.next_track_prefetched = true;
            tokio::spawn(async move {
                if let Ok(data) = download_track(&api, &track_id, format_id).await {
                    let meta = TrackMeta {
                        artist: &artist,
                        album: &album,
                        track_number: track_num,
                        title: &title,
                    };
                    cache.put(&track_id, &data, &meta);
                }
            });
        }
    }

    pub fn save_session(&self) {
        let s = session::Session {
            queue: self.queue.iter().map(SessionTrack::from_track).collect(),
            queue_index: self.queue_index,
            volume: self.player.volume,
            loop_mode: match self.loop_mode {
                LoopMode::Off => 0,
                LoopMode::Track => 1,
                LoopMode::Queue => 2,
            },
        };
        session::save(&s);
    }

    fn do_load_favorites(&mut self) {
        self.status_message = Some("Loading favorite albums...".to_string());
        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match api.get_favorite_albums(500).await {
                Ok(albums) => { tx.send(AppMessage::FavoritesResults(albums)).ok(); }
                Err(e) => { tx.send(AppMessage::FavoritesError(e.to_string())).ok(); }
            }
        });
    }

    /// Set a temporary status message that auto-clears after 3 seconds.
    fn set_temp_status(&mut self, msg: String) {
        self.status_message = Some(msg);
        self.status_expires = Some(std::time::Instant::now() + std::time::Duration::from_secs(3));
    }

    pub fn tick(&mut self) {
        // Auto-clear expired status messages
        if let Some(expires) = self.status_expires
            && std::time::Instant::now() >= expires
        {
            self.status_message = None;
            self.status_expires = None;
        }

        // Pre-fetch next track for gapless playback (15s before end)
        if self.player.is_playing
            && self.player.current_track_duration > 0
            && self.player.elapsed_secs() + 15 >= self.player.current_track_duration
        {
            self.prefetch_next();
        }

        if self.player.is_finished() {
            self.next_track_prefetched = false;
            match self.loop_mode {
                LoopMode::Track => {
                    // Replay the same track
                    if let Some(track) = self.queue.get(self.queue_index).cloned() {
                        self.play_track(track);
                    }
                }
                LoopMode::Queue => {
                    // Advance, wrap around to beginning
                    if self.queue_index + 1 < self.queue.len() {
                        self.queue_index += 1;
                    } else {
                        self.queue_index = 0;
                    }
                    if let Some(track) = self.queue.get(self.queue_index).cloned() {
                        self.play_track(track);
                    }
                }
                LoopMode::Off => {
                    // Advance, stop at end
                    if self.queue_index + 1 < self.queue.len() {
                        self.queue_index += 1;
                        if let Some(track) = self.queue.get(self.queue_index).cloned() {
                            self.play_track(track);
                        }
                    } else {
                        self.player.clear();
                    }
                }
            }
        }
    }
}

/// Compute scroll offset to keep `selected` visible in a window of `visible_height`.
pub fn scroll_offset(selected: usize, current_offset: usize, visible_height: usize) -> usize {
    if visible_height == 0 {
        return 0;
    }
    if selected < current_offset {
        selected
    } else if selected >= current_offset + visible_height {
        selected - visible_height + 1
    } else {
        current_offset
    }
}

/// Download a track, retrying with fresh URLs and falling back to lower quality.
/// Stream a track: get URL, start downloading, send a StreamingBuffer for
/// immediate playback, and return the complete data for caching.
async fn stream_track(
    api: &QobuzClient,
    track_id: &str,
    preferred_format: u32,
    tx: &mpsc::UnboundedSender<AppMessage>,
    title: &str,
    artist: &str,
    duration: u64,
) -> Result<Vec<u8>> {
    let formats = format_fallback_chain(preferred_format);
    let mut last_err = anyhow::anyhow!("No format available");
    let mut tried_first = false;

    for &fmt in formats {
        let url = match api.get_track_url(track_id, fmt).await {
            Ok(u) => u,
            Err(_) => {
                tried_first = true;
                continue;
            }
        };

        // Notify user of quality fallback
        if tried_first && fmt != preferred_format {
            let quality = AudioQuality::from_format_id(fmt)
                .map(|q| q.label())
                .unwrap_or("?");
            tx.send(AppMessage::StatusMessage(
                format!("Quality fallback: {}", quality),
            )).ok();
        }
        tried_first = true;

        let resp = match api.raw_client().get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                last_err = anyhow::anyhow!("HTTP {}", r.status());
                continue;
            }
            Err(e) => {
                last_err = anyhow::anyhow!("{}", e);
                continue;
            }
        };

        let total_size = resp.content_length().unwrap_or(0);
        let (writer, buffer) = stream::new_streaming_pair(total_size);
        let mut sent_buffer = false;
        let mut buffer_opt = Some(buffer);
        let mut resp = resp;
        let mut download_ok = true;
        let threshold = (total_size / 10).clamp(64 * 1024, 256 * 1024) as usize;

        loop {
            match resp.chunk().await {
                Ok(Some(chunk)) => {
                    writer.write(&chunk);
                    if !sent_buffer
                        && writer.downloaded() >= threshold
                        && let Some(buf) = buffer_opt.take()
                    {
                        tx.send(AppMessage::StreamReady(
                            buf, title.to_string(), artist.to_string(), duration, fmt,
                        )).ok();
                        sent_buffer = true;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    last_err = anyhow::anyhow!("Download: {}", e);
                    download_ok = false;
                    break;
                }
            }
        }
        writer.finish();

        if !sent_buffer
            && let Some(buf) = buffer_opt.take()
        {
            tx.send(AppMessage::StreamReady(
                buf, title.to_string(), artist.to_string(), duration, fmt,
            )).ok();
        }

        if download_ok {
            return Ok(writer.get_data());
        }
    }

    Err(last_err)
}

/// Download a full track (for album batch download, no streaming needed).
async fn download_track(
    api: &QobuzClient,
    track_id: &str,
    preferred_format: u32,
) -> Result<Vec<u8>> {
    let formats = format_fallback_chain(preferred_format);
    let mut last_err = anyhow::anyhow!("No format available");

    for &fmt in formats {
        let url = match api.get_track_url(track_id, fmt).await {
            Ok(u) => u,
            Err(_) => continue,
        };
        match api.download_audio(&url).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_err = anyhow::anyhow!("format {}: {}", fmt, e);
            }
        }
    }

    Err(last_err)
}
