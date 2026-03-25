use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AudioCache {
    cache_dir: PathBuf,
    /// Maps track_id → relative file path for quick lookup
    index: HashMap<String, PathBuf>,
}

/// Metadata needed to build a nice file path.
pub struct TrackMeta<'a> {
    pub artist: &'a str,
    pub album: &'a str,
    pub track_number: Option<u32>,
    pub title: &'a str,
}

impl AudioCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qobuz-tui");
        let mut cache = Self {
            cache_dir,
            index: HashMap::new(),
        };
        cache.build_index();
        cache
    }

    /// Scan the cache directory and build an index from track_id to file path.
    /// We store the track_id in a sidecar `.id` file next to each audio file.
    fn build_index(&mut self) {
        let Ok(entries) = glob_recursive(&self.cache_dir, "id") else {
            return;
        };
        for id_path in entries {
            if let Ok(track_id) = fs::read_to_string(&id_path) {
                let track_id = track_id.trim().to_string();
                let audio_path = id_path.with_extension("");
                if audio_path.exists()
                    && let Ok(rel) = audio_path.strip_prefix(&self.cache_dir)
                {
                    self.index.insert(track_id, rel.to_path_buf());
                }
            }
        }
    }

    /// Get cached audio data by track_id.
    pub fn get(&self, track_id: &str) -> Option<Vec<u8>> {
        let rel_path = self.index.get(track_id)?;
        let full_path = self.cache_dir.join(rel_path);
        fs::read(&full_path).ok()
    }

    /// Store audio data with nice file naming: `Artist/Album/01 - Title.flac`
    pub fn put(&self, track_id: &str, data: &[u8], meta: &TrackMeta) {
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
        let _ = fs::create_dir_all(&dir);

        let audio_path = dir.join(&filename);
        let _ = fs::write(&audio_path, data);

        // Write sidecar .id file for index lookup
        let id_path = audio_path.with_extension("id");
        let _ = fs::write(&id_path, track_id);
    }

    /// Check if a track is cached (without reading the data).
    pub fn has(&self, track_id: &str) -> bool {
        self.index.contains_key(track_id)
    }

    /// Register a newly cached track in the in-memory index.
    pub fn register(&mut self, track_id: &str, meta: &TrackMeta) {
        let artist = sanitize(meta.artist);
        let album = sanitize(meta.album);
        let num = meta.track_number.unwrap_or(0);
        let title = sanitize(meta.title);

        let filename = if num > 0 {
            format!("{:02} - {}.flac", num, title)
        } else {
            format!("{}.flac", title)
        };

        let rel_path = PathBuf::from(&artist).join(&album).join(&filename);
        self.index.insert(track_id.to_string(), rel_path);
    }
}

/// Sanitize a string for use as a file/directory name.
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

/// Recursively find all files with a given extension in a directory.
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
