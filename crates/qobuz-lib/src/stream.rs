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
        StreamWriter {
            inner: inner.clone(),
        },
        StreamingBuffer {
            inner,
            position: 0,
            total_size,
        },
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
                buf[..to_read].copy_from_slice(&state.data[self.position..self.position + to_read]);
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
        let base: i64 = match pos {
            SeekFrom::Start(n) => i64::try_from(n)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "position overflow"))?,
            SeekFrom::Current(n) => {
                let p = i64::try_from(self.position).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "position overflow")
                })?;
                p.checked_add(n)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
            }
            SeekFrom::End(n) => {
                let len = if self.total_size > 0 {
                    self.total_size
                } else {
                    let (mutex, _) = &*self.inner;
                    let state = mutex.lock().unwrap_or_else(|e| e.into_inner());
                    state.data.len() as u64
                };
                let l = i64::try_from(len)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "length overflow"))?;
                l.checked_add(n)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
            }
        };

        if base < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }

        self.position = usize::try_from(base)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "position overflow"))?;
        Ok(self.position as u64)
    }
}

// Safety: StreamingBuffer must be Send to pass it from the async download
// task to the audio thread. All shared data is behind Arc<(Mutex, Condvar)>.
// The `position` field is only accessed via &mut self (Read/Seek), so no
// concurrent access is possible once ownership is transferred.
// Sync is intentionally NOT implemented — StreamingBuffer must not be shared.
#[allow(unsafe_code)]
unsafe impl Send for StreamingBuffer {}
