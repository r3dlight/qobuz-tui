// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Progressive streaming download with quality fallback.
//!
//! Provides [`stream_track`] which downloads audio progressively, emitting
//! a [`StreamingBuffer`] for immediate playback via a [`StreamListener`] trait.
//! Frontends (TUI, Tauri, etc.) implement `StreamListener` to receive events.

use crate::api::{QobuzClient, format_fallback_chain};
use crate::error::{QobuzError, Result};
use crate::player::AudioQuality;
use crate::stream::{self, StreamingBuffer};

/// Minimum data before starting playback: 10% of file, clamped to 64-256KB.
const STREAM_MIN_BYTES: u64 = 64 * 1024;
const STREAM_MAX_BYTES: u64 = 256 * 1024;

/// Callback trait for streaming events. Frontends implement this to receive
/// buffers for playback and status notifications.
pub trait StreamListener: Send + Sync + 'static {
    /// Called when enough data is buffered to start playback.
    fn on_stream_ready(
        &self,
        buffer: StreamingBuffer,
        title: String,
        artist: String,
        duration: u64,
        format_id: u32,
    );
    /// Called when quality falls back to a lower format.
    fn on_quality_fallback(&self, quality_label: &str);
    /// Called when the full download completes (data for caching).
    fn on_stream_complete(&self, data: Vec<u8>, track_id: String);
    /// Called on error.
    fn on_stream_error(&self, err: String);
}

/// Stream a track with progressive download and quality fallback.
/// Returns the full audio data on success (for caching).
pub async fn stream_track(
    api: &QobuzClient,
    track_id: &str,
    preferred_format: u32,
    title: &str,
    artist: &str,
    duration: u64,
    listener: &dyn StreamListener,
) -> Result<Vec<u8>> {
    let formats = format_fallback_chain(preferred_format);
    let mut last_err = QobuzError::NoFormatAvailable;
    let mut tried_first = false;

    for &fmt in formats {
        let url = match api.get_track_url(track_id, fmt).await {
            Ok(u) => u,
            Err(_) => {
                tried_first = true;
                continue;
            }
        };

        if tried_first && fmt != preferred_format {
            let label = AudioQuality::from_format_id(fmt)
                .map(|q| q.label())
                .unwrap_or("?");
            listener.on_quality_fallback(label);
        }
        tried_first = true;

        let resp = match api.raw_client().get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                last_err = QobuzError::HttpStatus(r.status().as_u16(), String::new());
                continue;
            }
            Err(e) => {
                last_err = QobuzError::Network(e.to_string());
                continue;
            }
        };

        let total_size = resp.content_length().unwrap_or(0);
        let (writer, buffer) = stream::new_streaming_pair(total_size);
        let mut sent_buffer = false;
        let mut buffer_opt = Some(buffer);
        let mut resp = resp;
        let mut download_ok = true;
        let threshold = (total_size / 10).clamp(STREAM_MIN_BYTES, STREAM_MAX_BYTES) as usize;

        loop {
            match resp.chunk().await {
                Ok(Some(chunk)) => {
                    writer.write(&chunk);
                    if !sent_buffer
                        && writer.downloaded() >= threshold
                        && let Some(buf) = buffer_opt.take()
                    {
                        listener.on_stream_ready(
                            buf,
                            title.to_string(),
                            artist.to_string(),
                            duration,
                            fmt,
                        );
                        sent_buffer = true;
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    last_err = QobuzError::DownloadFailed(e.to_string());
                    download_ok = false;
                    break;
                }
            }
        }
        writer.finish();

        if !sent_buffer && let Some(buf) = buffer_opt.take() {
            listener.on_stream_ready(buf, title.to_string(), artist.to_string(), duration, fmt);
        }

        if download_ok {
            return Ok(writer.get_data());
        }
    }

    Err(last_err)
}
