// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Typed error types for qobuz-lib. Frontends can match on these to provide
//! context-specific error handling (e.g. retry on network, re-login on auth).

use std::fmt;

/// Main error type for qobuz-lib operations.
#[derive(Debug)]
pub enum QobuzError {
    /// Not authenticated — user needs to login.
    NotAuthenticated,
    /// Login failed (wrong credentials or API error).
    LoginFailed(String),
    /// HTTP request failed (network issue, timeout).
    Network(String),
    /// API returned a non-success HTTP status.
    HttpStatus(u16, String),
    /// API returned an error message in the JSON body.
    ApiError(String),
    /// Failed to parse API response.
    ParseError(String),
    /// No audio format available for this track.
    NoFormatAvailable,
    /// Audio download failed after retries.
    DownloadFailed(String),
    /// Invalid request signature (wrong app_secret).
    InvalidSignature,
    /// Credential extraction from Qobuz web player failed.
    CredentialExtraction(String),
    /// Audio decoder error.
    DecoderError(String),
    /// Config I/O error.
    ConfigError(String),
}

impl fmt::Display for QobuzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAuthenticated => write!(f, "Not authenticated"),
            Self::LoginFailed(msg) => write!(f, "Login failed: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::HttpStatus(code, msg) => write!(f, "HTTP {}: {}", code, msg),
            Self::ApiError(msg) => write!(f, "API error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NoFormatAvailable => write!(f, "No audio format available"),
            Self::DownloadFailed(msg) => write!(f, "Download failed: {}", msg),
            Self::InvalidSignature => write!(f, "Invalid request signature — check app_secret"),
            Self::CredentialExtraction(msg) => write!(f, "Credential extraction failed: {}", msg),
            Self::DecoderError(msg) => write!(f, "Decoder error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for QobuzError {}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, QobuzError>;

// Conversion from common error types
impl From<reqwest::Error> for QobuzError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<serde_json::Error> for QobuzError {
    fn from(e: serde_json::Error) -> Self {
        Self::ParseError(e.to_string())
    }
}

impl From<rodio::decoder::DecoderError> for QobuzError {
    fn from(e: rodio::decoder::DecoderError) -> Self {
        Self::DecoderError(format!("{:?}", e))
    }
}

impl From<std::io::Error> for QobuzError {
    fn from(e: std::io::Error) -> Self {
        Self::ConfigError(e.to_string())
    }
}

impl From<toml::ser::Error> for QobuzError {
    fn from(e: toml::ser::Error) -> Self {
        Self::ConfigError(e.to_string())
    }
}

impl From<std::time::SystemTimeError> for QobuzError {
    fn from(e: std::time::SystemTimeError) -> Self {
        Self::Network(format!("SystemTime: {}", e))
    }
}

impl From<regex::Error> for QobuzError {
    fn from(e: regex::Error) -> Self {
        Self::ParseError(format!("Regex: {}", e))
    }
}
