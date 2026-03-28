// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Audio playback engine with progress tracking, seek, and quality reporting.
//!
//! Supports two playback modes:
//! - **Cached**: from a `Vec<u8>` in memory (seekable immediately)
//! - **Streaming**: from a [`StreamingBuffer`] (seekable once download completes)

use crate::stream::StreamingBuffer;
use crate::error::Result;
use rodio::{Decoder, Sink, Source};
use std::io::Cursor;
use std::time::{Duration, Instant};

/// Audio quality format ID used by Qobuz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioQuality {
    Mp3_320 = 5,
    FlacCd = 6,
    FlacHiRes96 = 7,
    FlacHiRes192 = 27,
}

impl AudioQuality {
    /// Convert a Qobuz format_id (5, 6, 7, 27) to an `AudioQuality` variant.
    pub fn from_format_id(id: u32) -> Option<Self> {
        match id {
            5 => Some(Self::Mp3_320),
            6 => Some(Self::FlacCd),
            7 => Some(Self::FlacHiRes96),
            27 => Some(Self::FlacHiRes192),
            _ => None,
        }
    }

    /// Human-readable label (e.g. "FLAC 24/192").
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mp3_320 => "MP3 320k",
            Self::FlacCd => "FLAC 16/44",
            Self::FlacHiRes96 => "FLAC 24/96",
            Self::FlacHiRes192 => "FLAC 24/192",
        }
    }
}

/// Audio playback engine wrapping a rodio `Sink`.
pub struct Player {
    sink: Sink,
    pub volume: f32,
    pub is_playing: bool,
    pub is_loading: bool,
    pub current_track_title: Option<String>,
    pub current_track_artist: Option<String>,
    pub current_track_duration: u64,
    current_quality: Option<AudioQuality>,
    /// Cached audio data for seek support
    cached_data: Option<Vec<u8>>,
    // Timing
    play_started_at: Option<Instant>,
    paused_duration: Duration,
    pause_started_at: Option<Instant>,
    position_offset: Duration,
    last_seek_err: Option<String>,
}

impl Player {
    /// Create a new player with the given rodio `Sink`. Default volume: 80%.
    pub fn new(sink: Sink) -> Self {
        sink.set_volume(0.8);
        Self {
            sink,
            volume: 0.8,
            is_playing: false,
            is_loading: false,
            current_track_title: None,
            current_track_artist: None,
            current_track_duration: 0,
            current_quality: None,
            cached_data: None,
            play_started_at: None,
            paused_duration: Duration::ZERO,
            pause_started_at: None,
            position_offset: Duration::ZERO,
            last_seek_err: None,
        }
    }

    /// Play from fully downloaded audio data (seekable).
    pub fn play_audio(
        &mut self,
        data: Vec<u8>,
        title: &str,
        artist: &str,
        duration: u64,
    ) -> Result<()> {
        self.sink.stop();
        let source = Decoder::new(Cursor::new(data.clone()))?;
        self.cached_data = Some(data);
        self.start_playback(source, title, artist, duration);
        Ok(())
    }

    /// Play from a streaming buffer (progressive download, seek not available).
    pub fn play_streaming(
        &mut self,
        buffer: StreamingBuffer,
        title: &str,
        artist: &str,
        duration: u64,
    ) -> Result<()> {
        self.sink.stop();
        self.cached_data = None;
        let source = Decoder::new(buffer)?;
        self.start_playback(source, title, artist, duration);
        Ok(())
    }

    fn start_playback<S>(&mut self, source: S, title: &str, artist: &str, duration: u64)
    where
        S: Source + Send + 'static,
        f32: rodio::cpal::FromSample<S::Item>,
        S::Item: rodio::Sample + Send,
    {
        self.sink.append(source);
        self.sink.set_volume(self.volume);
        self.sink.play();

        self.is_playing = true;
        self.is_loading = false;
        self.current_track_title = Some(title.to_string());
        self.current_track_artist = Some(artist.to_string());
        self.current_track_duration = duration;
        self.current_quality = None;
        self.play_started_at = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
        self.position_offset = Duration::ZERO;
        self.last_seek_err = None;
    }

    /// Toggle between playing and paused. No-op if nothing is loaded.
    pub fn toggle_pause(&mut self) {
        if self.is_playing {
            self.sink.pause();
            self.is_playing = false;
            self.pause_started_at = Some(Instant::now());
        } else if self.current_track_title.is_some() {
            self.sink.play();
            self.is_playing = true;
            if let Some(pause_start) = self.pause_started_at.take() {
                self.paused_duration += pause_start.elapsed();
            }
        }
    }

    /// Mark playback as failed (e.g. download error).
    pub fn set_error(&mut self) {
        self.is_loading = false;
    }

    /// Set the audio quality of the current track.
    pub fn set_quality(&mut self, quality: Option<AudioQuality>) {
        self.current_quality = quality;
    }

    /// Store cached audio data to enable seek on the current track.
    pub fn enable_seek(&mut self, data: Vec<u8>) {
        self.cached_data = Some(data);
    }

    /// Current audio quality, if known.
    pub fn quality(&self) -> Option<AudioQuality> {
        self.current_quality
    }

    /// Whether seek is available (track data is cached).
    pub fn is_seekable(&self) -> bool {
        self.cached_data.is_some()
    }

    /// Set volume (clamped to 0.0..1.0).
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    /// Increase volume by 5%.
    pub fn volume_up(&mut self) {
        self.set_volume(self.volume + 0.05);
    }

    /// Decrease volume by 5%.
    pub fn volume_down(&mut self) {
        self.set_volume(self.volume - 0.05);
    }

    /// Mark the player as loading a new track (shows "Loading..." in the UI).
    /// Resets quality, seek state, and timing.
    pub fn set_loading(&mut self, title: &str, artist: &str) {
        self.is_loading = true;
        self.is_playing = false;
        self.current_quality = None;
        self.last_seek_err = None;
        self.current_track_title = Some(title.to_string());
        self.current_track_artist = Some(artist.to_string());
    }

    /// True if the sink is empty and we were playing (track ended naturally).
    pub fn is_finished(&self) -> bool {
        self.sink.empty() && self.is_playing && !self.is_loading
    }

    /// Seek forward by `secs` seconds.
    pub fn seek_forward(&mut self, secs: u64) -> bool {
        let current = self.elapsed_secs();
        let target = (current + secs).min(self.current_track_duration);
        self.seek_to(target)
    }

    /// Seek backward by `secs` seconds.
    pub fn seek_backward(&mut self, secs: u64) -> bool {
        let current = self.elapsed_secs();
        let target = current.saturating_sub(secs);
        self.seek_to(target)
    }

    /// Last seek error message, if any.
    pub fn last_seek_error(&self) -> Option<&str> {
        self.last_seek_err.as_deref()
    }

    /// Seek by recreating the decoder and skipping to the target position.
    /// Only works for cached tracks (we need the raw data to recreate the decoder).
    fn seek_to(&mut self, position_secs: u64) -> bool {
        self.last_seek_err = None;

        let Some(data) = &self.cached_data else {
            self.last_seek_err = Some("Seek available after track is cached".to_string());
            return false;
        };
        let data = data.clone();

        let source = match Decoder::new(Cursor::new(data)) {
            Ok(s) => s,
            Err(e) => {
                self.last_seek_err = Some(format!("Decoder error: {:?}", e));
                return false;
            }
        };

        // Skip to the target position
        let skip_dur = Duration::from_secs(position_secs);
        let skipped = source.skip_duration(skip_dur);

        self.sink.stop();
        self.sink.append(skipped);
        self.sink.set_volume(self.volume);
        self.sink.play();

        self.is_playing = true;
        self.position_offset = Duration::from_secs(position_secs);
        self.play_started_at = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
        true
    }

    /// Elapsed playback time in seconds (accounts for pauses and seeks).
    pub fn elapsed_secs(&self) -> u64 {
        if let Some(started) = self.play_started_at {
            let wall_time = started.elapsed();
            let paused = self.paused_duration
                + self
                    .pause_started_at
                    .map(|p| p.elapsed())
                    .unwrap_or_default();
            let played = wall_time.saturating_sub(paused);
            (self.position_offset + played)
                .as_secs()
                .min(self.current_track_duration)
        } else {
            0
        }
    }

    /// Playback progress as a fraction (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        if self.current_track_duration == 0 {
            return 0.0;
        }
        (self.elapsed_secs() as f64 / self.current_track_duration as f64).min(1.0)
    }

    /// Reset all player state (stop playback, clear track info).
    pub fn clear(&mut self) {
        self.is_playing = false;
        self.is_loading = false;
        self.cached_data = None;
        self.current_quality = None;
        self.current_track_title = None;
        self.current_track_artist = None;
        self.current_track_duration = 0;
        self.play_started_at = None;
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
        self.position_offset = Duration::ZERO;
    }
}
