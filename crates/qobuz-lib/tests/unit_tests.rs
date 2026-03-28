// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight

use qobuz_lib::api::{AlbumBrief, Artist, Track, format_fallback_chain};
use qobuz_lib::config::Config;
use qobuz_lib::player::AudioQuality;
use qobuz_lib::session::{Session, SessionTrack};

#[test]
fn audio_quality_from_format_id() {
    assert_eq!(AudioQuality::from_format_id(5), Some(AudioQuality::Mp3_320));
    assert_eq!(AudioQuality::from_format_id(6), Some(AudioQuality::FlacCd));
    assert_eq!(
        AudioQuality::from_format_id(7),
        Some(AudioQuality::FlacHiRes96)
    );
    assert_eq!(
        AudioQuality::from_format_id(27),
        Some(AudioQuality::FlacHiRes192)
    );
    assert_eq!(AudioQuality::from_format_id(99), None);
    assert_eq!(AudioQuality::from_format_id(0), None);
}

#[test]
fn audio_quality_labels() {
    assert_eq!(AudioQuality::Mp3_320.label(), "MP3 320k");
    assert_eq!(AudioQuality::FlacCd.label(), "FLAC 16/44");
    assert_eq!(AudioQuality::FlacHiRes96.label(), "FLAC 24/96");
    assert_eq!(AudioQuality::FlacHiRes192.label(), "FLAC 24/192");
}

#[test]
fn format_fallback_chain_27() {
    assert_eq!(format_fallback_chain(27), &[27, 7, 6, 5]);
}

#[test]
fn format_fallback_chain_7() {
    assert_eq!(format_fallback_chain(7), &[7, 6, 5]);
}

#[test]
fn format_fallback_chain_6() {
    assert_eq!(format_fallback_chain(6), &[6, 5]);
}

#[test]
fn format_fallback_chain_unknown() {
    assert_eq!(format_fallback_chain(99), &[5]);
}

#[test]
fn shuffle_keeps_current_track() {
    let mut queue = vec![
        make_track("1", "A"),
        make_track("2", "B"),
        make_track("3", "C"),
        make_track("4", "D"),
        make_track("5", "E"),
    ];
    let new_idx = qobuz_lib::shuffle(&mut queue, 2);
    assert_eq!(new_idx, 0);
    assert_eq!(queue[0].id, "3"); // original track at index 2 is now at 0
}

#[test]
fn shuffle_single_track() {
    let mut queue = vec![make_track("1", "A")];
    let idx = qobuz_lib::shuffle(&mut queue, 0);
    assert_eq!(idx, 0);
    assert_eq!(queue[0].id, "1");
}

#[test]
fn shuffle_empty_queue() {
    let mut queue: Vec<Track> = Vec::new();
    let idx = qobuz_lib::shuffle(&mut queue, 0);
    assert_eq!(idx, 0);
}

#[test]
fn session_track_roundtrip() {
    let track = make_track("42", "Test Song");
    let session_track = SessionTrack::from_track(&track);
    let restored = session_track.to_track();

    assert_eq!(restored.id, "42");
    assert_eq!(restored.title, "Test Song");
    assert_eq!(restored.artist_name(), "Test Artist");
    assert_eq!(restored.album_title(), "Test Album");
    assert_eq!(restored.duration, 180);
}

#[test]
fn session_track_preserves_artist_id() {
    let track = Track {
        id: "1".into(),
        title: "T".into(),
        duration: 60,
        track_number: Some(1),
        performer: Some(Artist {
            id: "artist-123".into(),
            name: "A".into(),
        }),
        album: Some(AlbumBrief {
            title: "Al".into(),
            artist: None,
        }),
    };
    let st = SessionTrack::from_track(&track);
    assert_eq!(st.artist_id, "artist-123");
    let restored = st.to_track();
    assert_eq!(restored.performer.unwrap().id, "artist-123");
}

#[test]
fn session_serialization_roundtrip() {
    let session = Session {
        queue: vec![SessionTrack {
            id: "1".into(),
            title: "Song".into(),
            artist: "Artist".into(),
            artist_id: "a1".into(),
            album: "Album".into(),
            duration: 300,
            track_number: Some(5),
        }],
        queue_index: 0,
        volume: 0.75,
        loop_mode: 2,
    };

    let json = serde_json::to_string(&session).unwrap();
    let restored: Session = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.queue.len(), 1);
    assert_eq!(restored.queue[0].title, "Song");
    assert_eq!(restored.queue[0].artist_id, "a1");
    assert_eq!(restored.volume, 0.75);
    assert_eq!(restored.loop_mode, 2);
}

#[test]
fn config_defaults() {
    let config = Config::default();
    assert!(config.app_id.is_empty());
    assert!(config.app_secret.is_empty());
    assert!(config.user_auth_token.is_none());
    assert!(config.email.is_none());
    assert_eq!(config.format_id(), 27);
    assert!(!config.has_app_credentials());
    assert!(!config.is_logged_in());
}

#[test]
fn config_logged_in() {
    let config = Config {
        app_id: "123".into(),
        app_secret: "abc".into(),
        user_auth_token: Some("token".into()),
        email: Some("test@test.com".into()),
        format_id: Some(6),
    };
    assert!(config.has_app_credentials());
    assert!(config.is_logged_in());
    assert_eq!(config.format_id(), 6);
}

#[test]
fn track_format_duration() {
    let track = Track {
        duration: 185,
        ..Default::default()
    };
    assert_eq!(track.format_duration(), "3:05");

    let short = Track {
        duration: 5,
        ..Default::default()
    };
    assert_eq!(short.format_duration(), "0:05");

    let zero = Track {
        duration: 0,
        ..Default::default()
    };
    assert_eq!(zero.format_duration(), "0:00");
}

#[test]
fn track_artist_name_fallback() {
    let no_performer = Track::default();
    assert_eq!(no_performer.artist_name(), "Unknown");

    let with_performer = Track {
        performer: Some(Artist {
            id: String::new(),
            name: "Queen".into(),
        }),
        ..Default::default()
    };
    assert_eq!(with_performer.artist_name(), "Queen");
}

#[test]
fn track_album_title_fallback() {
    let no_album = Track::default();
    assert_eq!(no_album.album_title(), "Unknown");

    let with_album = Track {
        album: Some(AlbumBrief {
            title: "Jazz".into(),
            artist: None,
        }),
        ..Default::default()
    };
    assert_eq!(with_album.album_title(), "Jazz");
}

// Helper
fn make_track(id: &str, title: &str) -> Track {
    Track {
        id: id.into(),
        title: title.into(),
        duration: 180,
        track_number: Some(1),
        performer: Some(Artist {
            id: "a1".into(),
            name: "Test Artist".into(),
        }),
        album: Some(AlbumBrief {
            title: "Test Album".into(),
            artist: None,
        }),
    }
}
