use anyhow::Result;
use rodio::{Decoder, Sink};
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
    // Timing for progress bar
    play_started_at: Option<Instant>,
    paused_duration: Duration,
    pause_started_at: Option<Instant>,
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
            play_started_at: None,
            paused_duration: Duration::ZERO,
            pause_started_at: None,
        }
    }

    pub fn play_audio(&mut self, data: Vec<u8>, title: &str, artist: &str, duration: u64) -> Result<()> {
        self.sink.stop();

        let cursor = Cursor::new(data);
        let source = Decoder::new(cursor)?;

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

        Ok(())
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
        self.is_playing = false; // Prevent is_finished() from re-triggering
        self.current_track_title = Some(title.to_string());
        self.current_track_artist = Some(artist.to_string());
    }

    pub fn is_finished(&self) -> bool {
        self.sink.empty() && self.is_playing && !self.is_loading
    }

    /// Elapsed playback time in seconds (accounts for pauses).
    pub fn elapsed_secs(&self) -> u64 {
        if let Some(started) = self.play_started_at {
            let total = started.elapsed();
            let paused = self.paused_duration
                + self
                    .pause_started_at
                    .map(|p| p.elapsed())
                    .unwrap_or_default();
            let played = total.saturating_sub(paused);
            played.as_secs().min(self.current_track_duration)
        } else {
            0
        }
    }

    /// Progress as a fraction 0.0..1.0
    pub fn progress(&self) -> f64 {
        if self.current_track_duration == 0 {
            return 0.0;
        }
        (self.elapsed_secs() as f64 / self.current_track_duration as f64).min(1.0)
    }

    pub fn clear(&mut self) {
        self.is_playing = false;
        self.is_loading = false;
        self.current_track_title = None;
        self.current_track_artist = None;
        self.current_track_duration = 0;
        self.play_started_at = None;
        self.paused_duration = Duration::ZERO;
        self.pause_started_at = None;
    }
}
