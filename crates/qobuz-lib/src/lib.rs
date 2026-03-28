// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
#![deny(unsafe_code)]

pub mod api;
pub mod cache;
pub mod config;
pub mod error;
pub mod player;
pub mod queue;
pub mod session;
pub mod stream;
pub mod streaming;

pub use api::{
    Album, AlbumImage, Artist, ArtistDetail, Genre, Playlist, PlaylistOwner, QobuzClient,
    SearchResults, Track, format_fallback_chain,
};
pub use cache::{AudioCache, TrackMeta};
pub use config::Config;
pub use error::QobuzError;
pub use player::{AudioQuality, Player};
pub use queue::shuffle;
pub use stream::{StreamWriter, StreamingBuffer, new_streaming_pair};
pub use streaming::{StreamListener, stream_track};
