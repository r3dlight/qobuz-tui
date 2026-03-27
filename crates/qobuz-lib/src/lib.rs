// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
#![deny(unsafe_code)]

pub mod api;
pub mod cache;
pub mod config;
pub mod player;
pub mod session;
pub mod stream;

pub use api::{Album, AlbumImage, Artist, Playlist, PlaylistOwner, QobuzClient, SearchResults, Track};
pub use cache::{AudioCache, TrackMeta};
pub use config::Config;
pub use player::{AudioQuality, Player};
pub use stream::{new_streaming_pair, StreamingBuffer, StreamWriter};
