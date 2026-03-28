// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Local audio file cache organized as `Artist/Album/01 - Title.flac`.
//!
//! Each cached file has a sidecar `.id` file containing the Qobuz track ID
//! for reverse lookup. The in-memory index is rebuilt on startup by scanning
//! the cache directory.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
/// Thread-safe audio file cache with on-disk storage and in-memory index.
pub struct AudioCache {
    cache_dir: PathBuf,
    index: Arc<RwLock<HashMap<String, PathBuf>>>,
}

/// Metadata for building cache file paths (`Artist/Album/01 - Title.flac`).
pub struct TrackMeta<'a> {
    pub artist: &'a str,
    pub album: &'a str,
    pub track_number: Option<u32>,
    pub title: &'a str,
}

impl Default for AudioCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCache {
    /// Create a new cache, scanning `~/.cache/qobuz-tui/` for existing files.
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qobuz-tui");
        let mut index = HashMap::new();
        build_index(&cache_dir, &mut index);
        Self {
            cache_dir,
            index: Arc::new(RwLock::new(index)),
        }
    }

    /// Read cached audio data for a track. Returns `None` if not cached.
    pub fn get(&self, track_id: &str) -> Option<Vec<u8>> {
        let index = self.index.read().unwrap_or_else(|e| e.into_inner());
        let rel_path = index.get(track_id)?;
        let full_path = self.cache_dir.join(rel_path);
        fs::read(&full_path).ok()
    }

    /// Check if a track is in the cache index (without reading the file).
    pub fn has(&self, track_id: &str) -> bool {
        self.index
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(track_id)
    }

    /// Store audio data. Returns true on success, false on I/O failure.
    pub fn put(&self, track_id: &str, data: &[u8], meta: &TrackMeta) -> bool {
        let artist = sanitize(meta.artist);
        let album = sanitize(meta.album);
        let num = meta.track_number.unwrap_or(0);
        let title = sanitize(meta.title);

        let filename = if num > 0 {
            format!("{:02} - {}.flac", num, title)
        } else {
            format!("{}.flac", title)
        };

        let dir = self.cache_dir.join(&artist).join(&album);
        if fs::create_dir_all(&dir).is_err() {
            return false;
        }

        let audio_path = dir.join(&filename);
        if fs::write(&audio_path, data).is_err() {
            return false;
        }

        let id_path = audio_path.with_extension("id");
        if fs::write(&id_path, track_id).is_err() {
            return false;
        }

        // Update in-memory index
        let rel_path = PathBuf::from(&artist).join(&album).join(&filename);
        self.index
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(track_id.to_string(), rel_path);
        true
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn build_index(cache_dir: &PathBuf, index: &mut HashMap<String, PathBuf>) {
    let Ok(entries) = glob_recursive(cache_dir, "id") else {
        return;
    };
    for id_path in entries {
        if let Ok(track_id) = fs::read_to_string(&id_path) {
            let track_id = track_id.trim().to_string();
            let audio_path = id_path.with_extension("");
            if audio_path.exists()
                && let Ok(rel) = audio_path.strip_prefix(cache_dir)
            {
                index.insert(track_id, rel.to_path_buf());
            }
        }
    }
}

fn glob_recursive(dir: &PathBuf, ext: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    if !dir.is_dir() {
        return Ok(results);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            results.extend(glob_recursive(&path, ext)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            results.push(path);
        }
    }
    Ok(results)
}
