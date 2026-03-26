// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 r3dlight
//! Shared streaming buffer for progressive audio playback.
//!
//! A `StreamWriter` feeds data from the download task.
//! A `StreamingBuffer` implements `Read + Seek` for the audio decoder,
//! blocking on read when data is not yet available (using a Condvar
//! instead of busy-waiting).

use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Condvar, Mutex};

/// Shared state between writer and reader.
struct Inner {
    data: Vec<u8>,
    complete: bool,
}

/// Writer side — fed by the download task.
pub struct StreamWriter {
    inner: Arc<(Mutex<Inner>, Condvar)>,
}

/// Reader side — consumed by the audio decoder.
pub struct StreamingBuffer {
    inner: Arc<(Mutex<Inner>, Condvar)>,
    position: usize,
    total_size: u64,
}

/// Create a linked (writer, reader) pair for streaming audio.
/// `total_size` should be the Content-Length (used for `SeekFrom::End`).
pub fn new_streaming_pair(total_size: u64) -> (StreamWriter, StreamingBuffer) {
    let inner = Arc::new((
        Mutex::new(Inner {
            data: Vec::with_capacity(total_size.min(50 * 1024 * 1024) as usize),
            complete: false,
        }),
        Condvar::new(),
    ));

    (
        StreamWriter { inner: inner.clone() },
        StreamingBuffer { inner, position: 0, total_size },
    )
}

impl StreamWriter {
    /// Append a chunk of downloaded data.
    pub fn write(&self, chunk: &[u8]) {
        let (mutex, cvar) = &*self.inner;
        let mut state = mutex.lock().unwrap_or_else(|e| e.into_inner());
        state.data.extend_from_slice(chunk);
        cvar.notify_all();
    }

    /// Mark the download as complete (signals EOF to the reader).
    pub fn finish(&self) {
        let (mutex, cvar) = &*self.inner;
        let mut state = mutex.lock().unwrap_or_else(|e| e.into_inner());
        state.complete = true;
        cvar.notify_all();
    }

    /// How many bytes have been downloaded so far.
    pub fn downloaded(&self) -> usize {
        let (mutex, _) = &*self.inner;
        let state = mutex.lock().unwrap_or_else(|e| e.into_inner());
        state.data.len()
    }

    /// Clone all downloaded data (for caching after completion).
    pub fn get_data(&self) -> Vec<u8> {
        let (mutex, _) = &*self.inner;
        let state = mutex.lock().unwrap_or_else(|e| e.into_inner());
        state.data.clone()
    }
}

impl Read for StreamingBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (mutex, cvar) = &*self.inner;
        let mut state = mutex.lock().unwrap_or_else(|e| e.into_inner());

        loop {
            let available = state.data.len().saturating_sub(self.position);
            if available > 0 {
                let to_read = buf.len().min(available);
                buf[..to_read]
                    .copy_from_slice(&state.data[self.position..self.position + to_read]);
                self.position += to_read;
                return Ok(to_read);
            }
            if state.complete {
                return Ok(0); // True EOF
            }
            // Wait for writer to append data or finish (no busy-wait)
            state = cvar.wait(state).unwrap_or_else(|e| e.into_inner());
        }
    }
}

impl Seek for StreamingBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::Current(n) => self.position as i64 + n,
            SeekFrom::End(n) => {
                if self.total_size > 0 {
                    self.total_size as i64 + n
                } else {
                    let (mutex, _) = &*self.inner;
                    let state = mutex.lock().unwrap_or_else(|e| e.into_inner());
                    state.data.len() as i64 + n
                }
            }
        };

        if new_pos < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "seek before start"));
        }

        self.position = new_pos as usize;
        Ok(self.position as u64)
    }
}

// Safety: all shared state is behind Arc<(Mutex, Condvar)>
unsafe impl Send for StreamingBuffer {}
unsafe impl Sync for StreamingBuffer {}
