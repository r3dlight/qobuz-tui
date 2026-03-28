// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Play queue utilities (shuffle, etc.).

use crate::api::Track;
use rand::seq::SliceRandom;

/// Shuffle the queue while keeping the currently playing track at `current_index`.
/// Returns the new index of the current track (always 0 after shuffle).
pub fn shuffle(queue: &mut [Track], current_index: usize) -> usize {
    if queue.len() <= 1 {
        return current_index;
    }
    // Move current track to front, shuffle the rest
    if current_index < queue.len() {
        queue.swap(0, current_index);
    }
    let mut rng = rand::rng();
    queue[1..].shuffle(&mut rng);
    0
}
