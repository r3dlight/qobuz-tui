// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
use landlock::{
    ABI, Access, AccessFs, Ruleset, RulesetAttr, RulesetCreatedAttr,
    path_beneath_rules,
};

/// Apply a Landlock sandbox restricting filesystem access.
///
/// - Read-write: config dir, cache dir, /tmp
/// - Read-only: system audio libs, SSL certs, /dev (audio devices), /proc, /sys
/// - Network: unrestricted (Landlock requires per-port rules, impractical for HTTPS CDN)
///
/// Falls back gracefully on kernels without Landlock support (BestEffort mode).
pub fn apply(config_dir: &std::path::Path, cache_dir: &std::path::Path) -> Result<(), String> {
    let abi = ABI::V4;

    let fs_all = AccessFs::from_all(abi);
    let fs_read = AccessFs::from_read(abi);

    // Only restrict filesystem — network is left unrestricted because Landlock
    // requires individual port rules and we need access to arbitrary CDN ports.
    let status = Ruleset::default()
        .handle_access(fs_all)
        .map_err(|e| format!("handle_access fs: {e}"))?
        .create()
        .map_err(|e| format!("create: {e}"))?
        // Read-write: app config, cache, and temp directory
        .add_rules(path_beneath_rules(
            &[
                config_dir.to_str().unwrap_or(""),
                cache_dir.to_str().unwrap_or(""),
                "/tmp",
            ],
            fs_all,
        ))
        .map_err(|e| format!("rw rules: {e}"))?
        // Read-only: system libraries, audio subsystem, SSL certs, DNS, devices
        .add_rules(path_beneath_rules(
            &[
                "/usr",
                "/lib",
                "/lib64",
                "/etc/alsa",
                "/etc/asound.conf",
                "/etc/pipewire",
                "/etc/pulse",
                "/etc/ld.so.cache",
                "/etc/ld.so.conf",
                "/etc/ld.so.conf.d",
                "/etc/ssl",
                "/etc/ca-certificates",
                "/etc/pki",
                "/etc/resolv.conf",
                "/etc/nsswitch.conf",
                "/etc/hosts",
                "/etc/gai.conf",
                "/dev",
                "/proc",
                "/sys",
                "/run",
            ],
            fs_read,
        ))
        .map_err(|e| format!("ro rules: {e}"))?
        .restrict_self()
        .map_err(|e| format!("restrict_self: {e}"))?;

    match status.ruleset {
        landlock::RulesetStatus::FullyEnforced => {
            eprintln!("[sandbox] Landlock fully enforced");
        }
        landlock::RulesetStatus::PartiallyEnforced => {
            eprintln!("[sandbox] Landlock partially enforced");
        }
        landlock::RulesetStatus::NotEnforced => {
            eprintln!("[sandbox] Landlock not enforced (unsupported kernel)");
        }
    }

    Ok(())
}
