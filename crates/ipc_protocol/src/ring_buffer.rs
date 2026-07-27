use crate::messages::MetricPayload;
use std::sync::atomic::{AtomicU32, Ordering};

pub const RING_BUFFER_CAPACITY: usize = 128;

/// Lock-free, zero-copy single-producer single-consumer ring buffer layout
/// designed for Win32 `CreateFileMapping` memory mapping.
#[repr(C)]
pub struct SharedMemoryRingBuffer {
    pub head: AtomicU32,
    pub tail: AtomicU32,
    pub slots: [MetricPayload; RING_BUFFER_CAPACITY],
}

impl SharedMemoryRingBuffer {
    pub fn new() -> Self {
        Self {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            slots: [MetricPayload::default(); RING_BUFFER_CAPACITY],
        }
    }

    /// Push a metric payload into the ring buffer (producer)
    pub fn push(&mut self, payload: MetricPayload) -> bool {
        let current_tail = self.tail.load(Ordering::Relaxed);
        let current_head = self.head.load(Ordering::Acquire);

        if (current_tail + 1) % (RING_BUFFER_CAPACITY as u32) == current_head {
            return false; // Buffer full
        }

        self.slots[current_tail as usize] = payload;
        self.tail.store(
            (current_tail + 1) % (RING_BUFFER_CAPACITY as u32),
            Ordering::Release,
        );
        true
    }

    /// Pop a metric payload from the ring buffer (consumer)
    pub fn pop(&mut self) -> Option<MetricPayload> {
        let current_head = self.head.load(Ordering::Relaxed);
        let current_tail = self.tail.load(Ordering::Acquire);

        if current_head == current_tail {
            return None; // Buffer empty
        }

        let payload = self.slots[current_head as usize];
        self.head.store(
            (current_head + 1) % (RING_BUFFER_CAPACITY as u32),
            Ordering::Release,
        );
        Some(payload)
    }
}
