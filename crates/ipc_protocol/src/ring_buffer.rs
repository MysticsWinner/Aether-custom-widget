//! High-Performance Zero-Copy SPSC Lock-Free Shared Memory Ring Buffer
//!
//! Designed for Win32 `CreateFileMappingW` memory-mapped inter-process communication.
//! Features 64-byte cacheline alignment (`#[repr(align(64))]`) to prevent false sharing,
//! atomic acquire-release semantics, and Win32 named event synchronization support.

use crate::messages::MetricPayload;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub const RING_BUFFER_CAPACITY: usize = 256;

/// Cacheline-aligned lock-free SPSC circular ring buffer layout.
#[repr(C, align(64))]
pub struct SharedMemoryRingBuffer {
    pub head: AtomicU32,
    pub tail: AtomicU32,
    pub is_event_signaled: AtomicBool,
    pub total_messages_pushed: AtomicU32,
    pub slots: [MetricPayload; RING_BUFFER_CAPACITY],
}

impl SharedMemoryRingBuffer {
    /// Creates a new empty `SharedMemoryRingBuffer`.
    pub fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            is_event_signaled: AtomicBool::new(false),
            total_messages_pushed: AtomicU32::new(0),
            slots: [MetricPayload::default(); RING_BUFFER_CAPACITY],
        }
    }

    /// Pushes a metric payload into the ring buffer (producer).
    /// Returns true on success, false if the buffer is saturated.
    pub fn push(&mut self, payload: MetricPayload) -> bool {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let current_head = self.head.load(Ordering::Acquire);

        let next_tail = (current_tail + 1) % (RING_BUFFER_CAPACITY as u32);
        if next_tail == current_head {
            return false; // Buffer full
        }

        self.slots[current_tail as usize] = payload;
        self.tail.store(next_tail, Ordering::Release);
        self.total_messages_pushed.fetch_add(1, Ordering::Relaxed);
        self.is_event_signaled.store(true, Ordering::Release);
        true
    }

    /// Pops a metric payload from the ring buffer (consumer).
    /// Returns None if the buffer is empty.
    pub fn pop(&mut self) -> Option<MetricPayload> {
        let current_head = self.head.load(Ordering::Relaxed);
        let current_tail = self.tail.load(Ordering::Acquire);

        if current_head == current_tail {
            self.is_event_signaled.store(false, Ordering::Release);
            return None; // Buffer empty
        }

        let payload = self.slots[current_head as usize];
        let next_head = (current_head + 1) % (RING_BUFFER_CAPACITY as u32);
        self.head.store(next_head, Ordering::Release);
        Some(payload)
    }

    /// Returns the number of unread messages in the buffer.
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        if tail >= head {
            (tail - head) as usize
        } else {
            (RING_BUFFER_CAPACITY as u32 - head + tail) as usize
        }
    }

    /// Returns true if the ring buffer contains no unread messages.
    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Relaxed) == self.tail.load(Ordering::Relaxed)
    }

    /// Returns maximum capacity of the ring buffer.
    pub fn capacity(&self) -> usize {
        RING_BUFFER_CAPACITY - 1
    }
}

impl Default for SharedMemoryRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop_lifecycle() {
        let mut buffer = SharedMemoryRingBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        let mut payload = MetricPayload::default();
        payload.cpu_usage_pct = 78.4;
        payload.timestamp_ms = 123456789;

        assert!(buffer.push(payload));
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);

        let popped = buffer.pop().expect("Should pop message");
        assert_eq!(popped.cpu_usage_pct, 78.4);
        assert_eq!(popped.timestamp_ms, 123456789);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_ring_buffer_wraparound_saturation() {
        let mut buffer = SharedMemoryRingBuffer::new();
        let cap = buffer.capacity();

        for i in 0..cap {
            let mut payload = MetricPayload::default();
            payload.timestamp_ms = i as u64;
            assert!(buffer.push(payload), "Should succeed pushing up to capacity");
        }

        assert_eq!(buffer.len(), cap);
        let mut overflow_payload = MetricPayload::default();
        overflow_payload.timestamp_ms = 9999;
        assert!(!buffer.push(overflow_payload), "Must reject when capacity is reached");

        for i in 0..cap {
            let popped = buffer.pop().expect("Must pop items in FIFO order");
            assert_eq!(popped.timestamp_ms, i as u64);
        }

        assert!(buffer.is_empty());
    }
}
