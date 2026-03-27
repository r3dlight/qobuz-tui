// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Qobuz API client — authentication, search, streaming URLs, favorites, and playlists.
//!
//! API credentials (`app_id` and `app_secret`) are automatically extracted from
//! the Qobuz web player's JavaScript bundles on first launch. The `app_secret`
//! is reconstructed from base64 fragments hidden in a fake timezone data structure.
//!
//! Note: MD5 is used for password hashing and request signing as required by the
//! Qobuz API protocol. This is not a design choice — the API mandates it.

use anyhow::{anyhow, Result};
use base64::Engine;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};

const BASE_URL: &str = "https://www.qobuz.com/api.json/0.2";
const WEB_PLAYER_URL: &str = "https://play.qobuz.com/login";

#[derive(Clone)]
/// Qobuz API client. Handles authentication, search, streaming, and favorites.
pub struct QobuzClient {
    client: Client,
    /// Client without automatic decompression — for raw audio streaming
    raw_client: Client,
    pub app_id: String,
    pub app_secret: String,
    pub user_auth_token: Option<String>,
}

/// Deserialize an ID that can be either a JSON number or string, preserving as String.
fn deserialize_id<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match &v {
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::String(s) => Ok(s.clone()),
        _ => Ok(String::new()),
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Track {
    #[serde(deserialize_with = "deserialize_id", default)]
    pub id: String,
    pub title: String,
    pub duration: u64,
    pub performer: Option<Artist>,
    pub album: Option<AlbumBrief>,
    pub track_number: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Artist {
    pub name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AlbumBrief {
    pub title: String,
    pub artist: Option<Artist>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AlbumImage {
    pub small: Option<String>,
    pub thumbnail: Option<String>,
    pub large: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Album {
    #[serde(deserialize_with = "deserialize_id", default)]
    pub id: String,
    pub title: String,
    pub artist: Option<Artist>,
    pub tracks: Option<TrackList>,
    pub tracks_count: Option<u32>,
    pub duration: Option<u64>,
    pub release_date_original: Option<String>,
    pub image: Option<AlbumImage>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TrackList {
    pub items: Vec<Track>,
    pub total: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AlbumList {
    pub items: Vec<Album>,
    pub total: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Playlist {
    #[serde(deserialize_with = "deserialize_id", default)]
    pub id: String,
    pub name: String,
    pub tracks_count: Option<u32>,
    pub duration: Option<u64>,
    pub owner: Option<PlaylistOwner>,
    pub tracks: Option<TrackList>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlaylistOwner {
    pub name: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SearchResults {
    pub tracks: Option<TrackList>,
    pub albums: Option<AlbumList>,
}


impl Track {
    /// Artist display name, or "Unknown" if not available.
    pub fn artist_name(&self) -> &str {
        self.performer
            .as_ref()
            .map(|a| a.name.as_str())
            .unwrap_or("Unknown")
    }

    /// Album title, or "Unknown" if not available.
    pub fn album_title(&self) -> &str {
        self.album
            .as_ref()
            .map(|a| a.title.as_str())
            .unwrap_or("Unknown")
    }

    /// Format duration as "M:SS".
    pub fn format_duration(&self) -> String {
        let mins = self.duration / 60;
        let secs = self.duration % 60;
        format!("{}:{:02}", mins, secs)
    }
}

/// Quality fallback chain for a given preferred format_id.
pub fn format_fallback_chain(preferred: u32) -> &'static [u32] {
    match preferred {
        27 => &[27, 7, 6, 5],
        7 => &[7, 6, 5],
        6 => &[6, 5],
        _ => &[5],
    }
}

impl QobuzClient {
    /// Create a new client with the given API credentials.
    pub fn new(app_id: &str, app_secret: &str) -> Self {
        // raw_client: no automatic decompression, for audio downloads.
        // Falls back to default client if builder fails (e.g. TLS init issue).
        let raw_client = Client::builder()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            client: Client::new(),
            raw_client,
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
            user_auth_token: None,
        }
    }

    /// Standard HTTP client (with automatic decompression).
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// HTTP client without decompression — for raw audio data downloads.
    pub fn raw_client(&self) -> &Client {
        &self.raw_client
    }

    /// Set the user authentication token (obtained from login).
    pub fn set_token(&mut self, token: String) {
        self.user_auth_token = Some(token);
    }

    /// Authenticate with Qobuz. Returns the user auth token on success.
    pub async fn login(&mut self, email: &str, password: &str) -> Result<String> {
        let password_hash = format!("{:x}", md5::compute(password.as_bytes()));

        let resp = self
            .client
            .post(format!("{}/user/login", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .form(&[
                ("username", email),
                ("password", &password_hash),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Login failed ({}): {}", status, body));
        }

        let body: serde_json::Value = resp.json().await?;
        let token = body
            .get("user_auth_token")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No auth token in response"))?;

        self.user_auth_token = Some(token.clone());
        Ok(token)
    }

    /// Search for tracks and albums matching `query`.
    pub async fn search(&self, query: &str, limit: u32) -> Result<SearchResults> {
        self.search_with_offset(query, limit, 0).await
    }

    /// Search with pagination offset.
    pub async fn search_with_offset(&self, query: &str, limit: u32, offset: u32) -> Result<SearchResults> {
        let token = self.require_token()?;

        let resp = self
            .client
            .get(format!("{}/catalog/search", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[
                ("query", query),
                ("limit", &limit.to_string()),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Search failed: {}", resp.status()));
        }

        let text = resp.text().await?;
        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Invalid JSON: {} — raw: {}", e, &text[..text.len().min(500)]))?;

        // Check for API error response
        if let Some(message) = body.get("message").and_then(|m| m.as_str()) {
            return Err(anyhow!("API error: {}", message));
        }

        // Parse tracks individually (skip any that fail to deserialize)
        let tracks: Vec<Track> = body
            .get("tracks")
            .and_then(|t| t.get("items"))
            .and_then(|items| items.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse albums individually
        let albums: Vec<Album> = body
            .get("albums")
            .and_then(|a| a.get("items"))
            .and_then(|items| items.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| serde_json::from_value(item.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(SearchResults {
            tracks: Some(TrackList {
                items: tracks,
                total: body
                    .get("tracks")
                    .and_then(|t| t.get("total"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0),
            }),
            albums: Some(AlbumList {
                items: albums,
                total: body
                    .get("albums")
                    .and_then(|a| a.get("total"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0),
            }),
        })
    }

    /// Fetch album details including tracklist.
    pub async fn get_album(&self, album_id: &str) -> Result<Album> {
        let token = self.require_token()?;

        let resp = self
            .client
            .get(format!("{}/album/get", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[("album_id", album_id)])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Get album failed (id={}): {}", album_id, resp.status()));
        }

        let body: serde_json::Value = resp.json().await?;
        let album: Album = serde_json::from_value(body)
            .map_err(|e| anyhow!("Failed to parse album: {}", e))?;
        Ok(album)
    }

    /// Get a signed streaming URL for a track at the given quality.
    pub async fn get_track_url(&self, track_id: &str, format_id: u32) -> Result<String> {
        let token = self.require_token()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs()
            .to_string();

        let sig_input = format!(
            "trackgetFileUrlformat_id{}intentstreamtrack_id{}{}{}",
            format_id, track_id, timestamp, self.app_secret
        );
        let sig = format!("{:x}", md5::compute(sig_input.as_bytes()));

        let resp = self
            .client
            .get(format!("{}/track/getFileUrl", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[
                ("track_id", track_id.to_string()),
                ("format_id", format_id.to_string()),
                ("intent", "stream".to_string()),
                ("request_ts", timestamp),
                ("request_sig", sig),
            ])
            .send()
            .await?;

        let status = resp.status();
        let bytes = resp.bytes().await
            .map_err(|e| anyhow!("Failed to read getFileUrl response: {}", e))?;
        let text = String::from_utf8_lossy(&bytes);

        if !status.is_success() {
            if status.as_u16() == 400 && text.contains("request_sig") {
                return Err(anyhow!(
                    "Invalid request signature — the app_secret is likely wrong. \
                     Edit app_secret in {:?}",
                    crate::config::Config::path()
                ));
            }
            return Err(anyhow!("Get track URL failed ({}): {}", status, text));
        }

        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Invalid JSON from getFileUrl: {}", e))?;
        body.get("url")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No URL in response: {}", body))
    }

    /// Fetch the user's favorite albums.
    pub async fn get_favorite_albums(&self, limit: u32) -> Result<Vec<Album>> {
        let token = self.require_token()?;

        let resp = self
            .client
            .get(format!("{}/favorite/getUserFavorites", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[
                ("type", "albums"),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!("Get favorite albums failed: {}", resp.status()));
        }

        let body: serde_json::Value = resp.json().await?;
        let albums = body
            .get("albums")
            .and_then(|a| a.get("items"))
            .map(|items| {
                items
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| serde_json::from_value::<Album>(item.clone()).ok())
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        Ok(albums)
    }

    /// Download raw audio data from a URL (with 3 retries).
    pub async fn download_audio(&self, url: &str) -> Result<Vec<u8>> {
        let mut last_err = anyhow!("Download failed");
        for attempt in 0..3u64 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_secs(1 + attempt)).await;
            }
            let resp = match self.raw_client.get(url).send().await {
                Ok(r) => r,
                Err(e) => {
                    last_err = anyhow!("Request failed: {}", e);
                    continue;
                }
            };
            if !resp.status().is_success() {
                last_err = anyhow!("HTTP {}", resp.status());
                continue;
            }
            match resp.bytes().await {
                Ok(bytes) if !bytes.is_empty() => return Ok(bytes.to_vec()),
                Ok(_) => last_err = anyhow!("Empty response"),
                Err(e) => last_err = anyhow!("Read failed: {}", e),
            }
        }
        Err(last_err)
    }

    /// Download a full track with automatic quality fallback (27→7→6→5).
    /// Used for album batch downloads and pre-fetching.
    pub async fn download_track(&self, track_id: &str, preferred_format: u32) -> Result<Vec<u8>> {
        let formats = format_fallback_chain(preferred_format);
        let mut last_err = anyhow!("No format available");

        for &fmt in formats {
            let url = match self.get_track_url(track_id, fmt).await {
                Ok(u) => u,
                Err(_) => continue,
            };
            match self.download_audio(&url).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_err = anyhow!("format {}: {}", fmt, e);
                }
            }
        }

        Err(last_err)
    }

    /// Add an album to the user's favorites.
    pub async fn favorite_add_album(&self, album_id: &str) -> Result<()> {
        let token = self.require_token()?;
        let resp = self
            .client
            .post(format!("{}/favorite/create", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .form(&[("album_ids", album_id)])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Add favorite failed: {}", resp.status()));
        }
        Ok(())
    }

    /// Remove an album from the user's favorites.
    pub async fn favorite_remove_album(&self, album_id: &str) -> Result<()> {
        let token = self.require_token()?;
        let resp = self
            .client
            .post(format!("{}/favorite/delete", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .form(&[("album_ids", album_id)])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Remove favorite failed: {}", resp.status()));
        }
        Ok(())
    }

    /// Fetch the user's playlists.
    pub async fn get_user_playlists(&self, limit: u32) -> Result<Vec<Playlist>> {
        let token = self.require_token()?;
        let resp = self
            .client
            .get(format!("{}/playlist/getUserPlaylists", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[("limit", &limit.to_string())])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Get playlists failed: {}", resp.status()));
        }
        let text = resp.text().await?;
        let body: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Invalid JSON: {}", e))?;

        // The API may return playlists at different paths depending on the response
        let items_val = body
            .get("playlists")
            .and_then(|p| p.get("items"))
            .or_else(|| body.get("items"))
            .or_else(|| {
                // Maybe the response IS the array directly
                if body.is_array() { Some(&body) } else { None }
            });

        let playlists = match items_val {
            Some(items) => items
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| serde_json::from_value::<Playlist>(item.clone()).ok())
                        .collect()
                })
                .unwrap_or_default(),
            None => {
                // Debug: show the top-level keys
                let keys: Vec<&str> = body
                    .as_object()
                    .map(|o| o.keys().map(|k| k.as_str()).collect())
                    .unwrap_or_default();
                return Err(anyhow!("Unexpected playlist response keys: [{}]", keys.join(", ")));
            }
        };
        Ok(playlists)
    }

    /// Fetch a playlist with its tracks.
    pub async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist> {
        let token = self.require_token()?;
        let resp = self
            .client
            .get(format!("{}/playlist/get", BASE_URL))
            .header("X-App-Id", &self.app_id)
            .header("X-User-Auth-Token", token)
            .query(&[
                ("playlist_id", playlist_id),
                ("extra", "tracks"),
                ("limit", "500"),
            ])
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(anyhow!("Get playlist failed: {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await?;
        serde_json::from_value(body).map_err(|e| anyhow!("Parse playlist: {}", e))
    }

    fn require_token(&self) -> Result<&str> {
        self.user_auth_token
            .as_deref()
            .ok_or_else(|| anyhow!("Not authenticated"))
    }
}

/// Fetch app_id and app_secret automatically from the Qobuz web player.
///
/// The real app_secret is NOT stored in plaintext in the bundle — it's reconstructed
/// at runtime from base64 fragments hidden in a fake "timezone" data structure.
/// This function replicates that reconstruction.
pub async fn fetch_app_credentials() -> Result<(String, String)> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0")
        .build()?;

    // 1. Fetch the login page HTML
    let html = client.get(WEB_PLAYER_URL).send().await?.text().await?;

    // 2. Find bundle script URLs
    let script_re = Regex::new(r#"<script[^>]+src="([^"]+\.js[^"]*)""#)?;
    let mut bundle_urls: Vec<String> = Vec::new();
    for cap in script_re.captures_iter(&html) {
        let url = &cap[1];
        if url.starts_with('/') {
            bundle_urls.push(format!("https://play.qobuz.com{}", url));
        } else if url.starts_with("http") {
            bundle_urls.push(url.to_string());
        }
    }

    let mut app_id: Option<String> = None;
    let mut app_secret: Option<String> = None;

    // Pre-compile regexes (avoid recompilation per bundle)
    let id_regexes: Vec<Regex> = [
        r#"production[^}]*?appId\s*:\s*"(\d{9,})""#,
        r#"appId\s*[:=]\s*"(\d{9,})""#,
    ].iter().filter_map(|p| Regex::new(p).ok()).collect();

    for url in &bundle_urls {
        let body = match client.get(url).send().await {
            Ok(resp) => match resp.text().await {
                Ok(text) => text,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        // Extract app_id from production config
        if app_id.is_none() {
            for re in &id_regexes {
                if let Some(cap) = re.captures(&body) {
                    app_id = Some(cap[1].to_string());
                    break;
                }
            }
        }

        // Extract the real app_secret via base64 fragment reconstruction
        if app_secret.is_none()
            && let Some(secret) = extract_secret_from_bundle(&body)
        {
            app_secret = Some(secret);
        }

        if app_id.is_some() && app_secret.is_some() {
            break;
        }
    }

    let id = app_id.ok_or_else(|| {
        anyhow!(
            "Could not extract app_id. Set it manually in {:?}",
            crate::config::Config::path()
        )
    })?;

    let secret = app_secret.ok_or_else(|| {
        anyhow!(
            "Could not extract app_secret. Set it manually in {:?}",
            crate::config::Config::path()
        )
    })?;

    Ok((id, secret))
}

/// Extract the real app_secret from the Qobuz bundle JS.
///
/// The bundle contains:
///   1. A call like: initialSeed("BASE64_SEED", window.utimezone.CITY)
///   2. Timezone entries with `info` and `extras` base64 fragments
///   3. The secret = base64_decode((seed + info + extras)[..len-44])
fn extract_secret_from_bundle(bundle: &str) -> Option<String> {
    // Find the production initialSeed call
    // Pattern: initialSeed("SEED", ... .utimezone.CITY)
    // We look for the one associated with production
    let seed_re = Regex::new(
        r#"initialSeed\(\s*"([A-Za-z0-9+/=]+)"\s*,\s*[a-zA-Z_.]*\.utimezone\.([a-z]+)\s*\)"#,
    ).ok()?;

    // Find the production environment seed call
    // Look for "production" context near an initialSeed call
    // Strategy: find all seed calls and their associated city names,
    // then look for which one is in the production block
    let mut seed_calls: Vec<(String, String)> = Vec::new(); // (seed_b64, city)
    for cap in seed_re.captures_iter(bundle) {
        seed_calls.push((cap[1].to_string(), cap[2].to_string()));
    }

    if seed_calls.is_empty() {
        return None;
    }

    // Try to find which seed call belongs to production
    // Look for "production" keyword near each match
    let mut production_seed: Option<&(String, String)> = None;
    for call in &seed_calls {
        // Check if "production" appears within 500 chars before this seed in the bundle
        if let Some(pos) = bundle.find(&call.0) {
            let start = pos.saturating_sub(500);
            let context = &bundle[start..pos];
            if context.contains("production") {
                production_seed = Some(call);
                break;
            }
        }
    }

    // Fallback: use the last seed call (production is typically the else/default branch)
    let (seed_b64, city) = production_seed.or(seed_calls.last())?;

    // The city name (e.g. "berlin") maps to an index in a timezones array.
    // The actual info/extras are on the array entry with name "Europe/Berlin".
    // Capitalize the city name to find the matching timezone entry.
    let city_capitalized = {
        let mut c = city.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().to_string() + c.as_str(),
        }
    };

    // Search for the timezone entry: name:"Europe/Berlin",info:"...",extras:"..."
    // or name:"Africa/Abidjan", name:"Europe/London", name:"Europe/Dublin", etc.
    // Try multiple continent prefixes
    let continents = ["Europe", "Africa", "America", "Asia", "Pacific", "Atlantic"];
    let mut info_val: Option<String> = None;
    let mut extras_val: Option<String> = None;

    for continent in &continents {
        let pattern = format!(
            r#"{}["/]{}[^}}]*?info\s*:\s*"([^"]+)"[^}}]*?extras\s*:\s*"([^"]+)""#,
            continent, &city_capitalized
        );
        if let Ok(re) = Regex::new(&pattern)
            && let Some(cap) = re.captures(bundle)
        {
            info_val = Some(cap[1].to_string());
            extras_val = Some(cap[2].to_string());
            break;
        }
    }

    let info = info_val?;
    let extras = extras_val?;

    // Reconstruct: base64_decode((seed + info + extras)[..total_len - 44])
    let combined = format!("{}{}{}", seed_b64, info, extras);
    let trimmed = if combined.len() > 44 {
        &combined[..combined.len() - 44]
    } else {
        return None;
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .ok()?;
    let secret = String::from_utf8(decoded).ok()?;

    // Validate: should be a 32-char hex string
    if secret.len() == 32 && secret.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(secret)
    } else {
        None
    }
}
