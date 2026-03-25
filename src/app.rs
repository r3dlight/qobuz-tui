use crate::api::{self, Album, QobuzClient, Track};
use crate::cache::{AudioCache, TrackMeta};
use crate::config::Config;
use crate::player::Player;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Login,
    Main,
    AlbumView,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Search,
    Favorites,
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
    /// data, title, artist, duration, track_id, album_title, track_number
    AudioReady(Vec<u8>, String, String, u64, String, String, Option<u32>),
    AudioError(String),
    AlbumLoaded(Album),
    AlbumError(String),
    DownloadProgress(usize, usize), // downloaded, total
    DownloadDone,
    DownloadError(String),
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

    // Play queue
    pub queue: Vec<Track>,
    pub queue_index: usize,

    // Player
    pub player: Player,

    // Cache
    pub cache: AudioCache,

    // Status
    pub status_message: Option<String>,

    // API client
    pub api: QobuzClient,
    pub config: Config,
    pub tx: mpsc::UnboundedSender<AppMessage>,
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
            queue: Vec::new(),
            queue_index: 0,
            player,
            cache: AudioCache::new(),
            status_message: None,
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
            AppMessage::AudioReady(data, title, artist, duration, track_id, album_title, track_num) => {
                let meta = TrackMeta {
                    artist: &artist,
                    album: &album_title,
                    track_number: track_num,
                    title: &title,
                };
                self.cache.put(&track_id, &data, &meta);
                self.cache.register(&track_id, &meta);
                if let Err(e) = self.player.play_audio(data, &title, &artist, duration) {
                    self.status_message = Some(format!("Playback error: {}", e));
                }
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
                _ => {}
            }
        }

        match self.screen {
            Screen::Login => self.handle_login_key(key),
            Screen::Main => self.handle_main_key(key),
            Screen::AlbumView => self.handle_album_key(key),
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
            KeyCode::Tab => {
                self.search_mode = match self.search_mode {
                    SearchMode::Tracks => SearchMode::Albums,
                    SearchMode::Albums => SearchMode::Tracks,
                };
                self.search_selected = 0;
                self.search_scroll = 0;
            }
            KeyCode::F(2) => {
                self.tab = Tab::Favorites;
                if !self.favorites_loaded {
                    self.do_load_favorites();
                }
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
            KeyCode::F(1) => self.tab = Tab::Search,
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
            KeyCode::Backspace => self.screen = Screen::Main,
            _ => {}
        }
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
                    match get_and_download_track(&api, &track.id, format_id).await {
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
            let track = self.queue[self.queue_index].clone();
            self.play_track(track);
        }
    }

    pub fn play_previous(&mut self) {
        if self.queue_index > 0 {
            self.queue_index -= 1;
            let track = self.queue[self.queue_index].clone();
            self.play_track(track);
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

        // Check cache first
        if let Some(data) = self.cache.get(&track_id) {
            if let Err(e) = self.player.play_audio(data, &title, &artist, duration) {
                self.status_message = Some(format!("Playback error: {}", e));
            }
            return;
        }

        self.player.set_loading(&title, &artist);

        let tx = self.tx.clone();
        let api = self.api.clone();
        tokio::spawn(async move {
            match get_and_download_track(&api, &track_id, format_id).await {
                Ok(data) => {
                    tx.send(AppMessage::AudioReady(
                        data, title, artist, duration, track_id, album_title, track_num,
                    )).ok();
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

    pub fn tick(&mut self) {
        // Auto-advance to next track when current one finishes
        if self.player.is_finished() {
            if self.queue_index + 1 < self.queue.len() {
                self.queue_index += 1;
                let track = self.queue[self.queue_index].clone();
                self.play_track(track);
            } else {
                self.player.clear();
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
async fn get_and_download_track(
    api: &QobuzClient,
    track_id: &str,
    preferred_format: u32,
) -> Result<Vec<u8>> {
    let formats = match preferred_format {
        27 => &[27, 7, 6, 5][..],
        7 => &[7, 6, 5][..],
        6 => &[6, 5][..],
        _ => &[preferred_format, 5][..],
    };

    let mut last_err = anyhow::anyhow!("No format available");

    for &fmt in formats {
        // Get a fresh URL for this format
        let url = match api.get_track_url(track_id, fmt).await {
            Ok(u) => u,
            Err(_) => continue, // format not available, try next
        };

        // Try downloading (download_audio already retries 3 times)
        match api.download_audio(&url).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_err = anyhow::anyhow!("format {}: {}", fmt, e);
                // Connection issue — try next format (smaller file)
            }
        }
    }

    Err(last_err)
}
