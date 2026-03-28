// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Application configuration stored in `~/.config/qobuz-tui/config.toml`.
//!
//! Holds Qobuz API credentials, user auth token, and audio quality preference.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Persistent application configuration (credentials, audio quality).
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub app_secret: String,
    #[serde(default)]
    pub user_auth_token: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub format_id: Option<u32>,
}

impl Config {
    /// Path to the config file (`~/.config/qobuz-tui/config.toml`).
    pub fn path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("qobuz-tui");
        config_dir.join("config.toml")
    }

    /// Load config from disk, or return defaults if the file doesn't exist.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save config to disk (creates parent directories if needed).
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Preferred audio format_id (default: 27 = Hi-Res 24-bit/192kHz).
    pub fn format_id(&self) -> u32 {
        self.format_id.unwrap_or(27) // Default: Hi-Res 24-bit/192kHz
    }

    /// Whether both `app_id` and `app_secret` are set.
    pub fn has_app_credentials(&self) -> bool {
        !self.app_id.is_empty() && !self.app_secret.is_empty()
    }

    /// Whether the user has valid credentials and an auth token.
    pub fn is_logged_in(&self) -> bool {
        self.user_auth_token.is_some() && self.has_app_credentials()
    }
}
