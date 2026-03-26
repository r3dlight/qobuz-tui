use crate::stream::StreamingBuffer;
use anyhow::Result;
use rodio::{Decoder, Sink, Source};
use std::io::Cursor;
use std::time::{Duration, Instant};

pub struct Player {
    sink: Sink,
    pub volume: f32,
    pub is_playing: bool,
    pub is_loading: bool,
    pub current_track_title: Option<String>,
    pub current_track_artist: Option<String>,
    pub current_track_duration: u64,
    /// Cached audio data for seek support (only when playing from cache)
    cached_data: Option<Vec<u8>>,
    // Timing
    play_started_at: Option<Instant>,
    paused_duration: Duration,
    pause_started_at: Option<Instant>,
    position_offset: Duration,
    last_seek_err: Option<String>,
}

impl Player {
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
        self.play_started_at = Some(Instant::now());
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
        self.position_offset = Duration::ZERO;
    }

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

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    pub fn volume_up(&mut self) {
        self.set_volume(self.volume + 0.05);
    }

    pub fn volume_down(&mut self) {
        self.set_volume(self.volume - 0.05);
    }

    pub fn set_loading(&mut self, title: &str, artist: &str) {
        self.is_loading = true;
        self.is_playing = false;
        self.current_track_title = Some(title.to_string());
        self.current_track_artist = Some(artist.to_string());
    }

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

    pub fn progress(&self) -> f64 {
        if self.current_track_duration == 0 {
            return 0.0;
        }
        (self.elapsed_secs() as f64 / self.current_track_duration as f64).min(1.0)
    }

    pub fn clear(&mut self) {
        self.is_playing = false;
        self.is_loading = false;
        self.cached_data = None;
        self.current_track_title = None;
        self.current_track_artist = None;
        self.current_track_duration = 0;
        self.play_started_at = None;
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
        self.position_offset = Duration::ZERO;
    }
}
