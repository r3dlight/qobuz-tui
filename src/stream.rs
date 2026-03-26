//! Shared streaming buffer for progressive audio playback.
//!
//! A `StreamWriter` feeds data from the download task.
//! A `StreamingBuffer` implements `Read + Seek` for the audio decoder,
//! blocking when data is not yet available.

use std::io::{self, Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// Writer side — fed by the download task.
pub struct StreamWriter {
    data: Arc<RwLock<Vec<u8>>>,
    complete: Arc<AtomicBool>,
}

/// Reader side — consumed by the audio decoder.
/// Blocks on read/seek when requested data is not yet downloaded.
pub struct StreamingBuffer {
    data: Arc<RwLock<Vec<u8>>>,
    position: usize,
    total_size: u64,
    complete: Arc<AtomicBool>,
}

/// Create a linked (writer, reader) pair for streaming audio.
/// `total_size` should be the Content-Length (used for `SeekFrom::End`).
pub fn new_streaming_pair(total_size: u64) -> (StreamWriter, StreamingBuffer) {
    let data = Arc::new(RwLock::new(Vec::with_capacity(
        total_size.min(50 * 1024 * 1024) as usize,
    )));
    let complete = Arc::new(AtomicBool::new(false));

    (
        StreamWriter {
            data: data.clone(),
            complete: complete.clone(),
        },
        StreamingBuffer {
            data,
            position: 0,
            total_size,
            complete,
        },
    )
}

impl StreamWriter {
    /// Append a chunk of downloaded data.
    pub fn write(&self, chunk: &[u8]) {
        self.data.write().unwrap().extend_from_slice(chunk);
    }

    /// Mark the download as complete (signals EOF to the reader).
    pub fn finish(&self) {
        self.complete.store(true, Ordering::Release);
    }

    /// How many bytes have been downloaded so far.
    pub fn downloaded(&self) -> usize {
        self.data.read().unwrap().len()
    }

    /// Clone all downloaded data (for caching after completion).
    pub fn get_data(&self) -> Vec<u8> {
        self.data.read().unwrap().clone()
    }
}

impl Read for StreamingBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            {
                let data = self.data.read().unwrap();
                let available = data.len().saturating_sub(self.position);
                if available > 0 {
                    let to_read = buf.len().min(available);
                    buf[..to_read]
                        .copy_from_slice(&data[self.position..self.position + to_read]);
                    self.position += to_read;
                    return Ok(to_read);
                }
                if self.complete.load(Ordering::Acquire) {
                    return Ok(0); // True EOF
                }
            }
            // Data not available yet — wait for more
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}

impl Seek for StreamingBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::Current(n) => self.position as i64 + n,
            SeekFrom::End(n) => {
                // Return total_size for End-relative seeks. The decoder uses this
                // to learn the file size, then seeks back to the start. We must NOT
                // block here — only Read should block when data isn't available yet.
                if self.total_size > 0 {
                    self.total_size as i64 + n
                } else {
                    // Unknown size: use what we have so far
                    let len = self.data.read().unwrap().len();
                    len as i64 + n
                }
            }
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }

        // Just set the position — don't wait for data.
        // If the decoder reads from this position, Read will block until
        // the data is available (or return EOF if the download is complete).
        self.position = new_pos as usize;
        Ok(self.position as u64)
    }
}

// Safety: StreamingBuffer can be sent between threads (Arc internals are Send+Sync)
unsafe impl Send for StreamingBuffer {}
unsafe impl Sync for StreamingBuffer {}
