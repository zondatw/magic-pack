//! Dependency-free byte-progress plumbing.
//!
//! The CLI shows a progress bar by polling a shared `AtomicU64` that the
//! codecs increment as bytes flow. The library never depends on the
//! progress UI (`indicatif` lives in the binary); it only bumps a
//! counter. A `None` counter makes everything a no-op, so the
//! non-instrumented paths (and the MCP server) pay nothing.

use std::io::{self, Read};
use std::sync::atomic::{AtomicU64, Ordering};

/// Borrowed handle to the shared byte counter passed down into codecs.
pub type Progress<'a> = Option<&'a AtomicU64>;

/// Wraps a reader and adds every byte read to a shared counter. Used to
/// measure how much input (compress) or how much of the archive
/// (decompress) has been consumed.
pub struct CountingReader<'a, R: Read> {
    inner: R,
    counter: Progress<'a>,
}

impl<'a, R: Read> CountingReader<'a, R> {
    pub fn new(inner: R, counter: Progress<'a>) -> Self {
        CountingReader { inner, counter }
    }
}

impl<R: Read> Read for CountingReader<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 {
            if let Some(counter) = self.counter {
                counter.fetch_add(n as u64, Ordering::Relaxed);
            }
        }
        Ok(n)
    }
}

/// Add `n` bytes to the counter if present (for codecs that report
/// per-file rather than per-read, e.g. tar/zip).
pub fn add(counter: Progress, n: u64) {
    if let Some(counter) = counter {
        counter.fetch_add(n, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn counts_all_bytes_read() {
        let data = [7u8; 5000];
        let counter = AtomicU64::new(0);
        let mut r = CountingReader::new(&data[..], Some(&counter));
        let mut sink = Vec::new();
        let n = r.read_to_end(&mut sink).unwrap();
        assert_eq!(n, 5000);
        assert_eq!(counter.load(Ordering::Relaxed), 5000);
    }

    #[test]
    fn none_counter_is_noop() {
        let data = [1u8; 100];
        let mut r = CountingReader::new(&data[..], None);
        let mut sink = Vec::new();
        assert_eq!(r.read_to_end(&mut sink).unwrap(), 100);
        // nothing to assert beyond "doesn't panic"
    }

    #[test]
    fn add_helper_increments() {
        let counter = AtomicU64::new(10);
        add(Some(&counter), 32);
        assert_eq!(counter.load(Ordering::Relaxed), 42);
        add(None, 99); // no-op
    }
}
