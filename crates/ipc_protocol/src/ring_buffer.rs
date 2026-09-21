//! High-Performance Shared Memory IPC Protocol
//!
//! Provides two specialized shared memory communication models for inter-process telemetry:
//! 1. `MultiReaderSnapshotBuffer` / `MultiReaderRingBuffer`: Single-Producer Multi-Consumer (SPMC)
//!    lock-free buffers using seqlock atomic sequence validation and per-reader independent cursors.
//!    Supports arbitrary concurrent readers (TUI dashboard, WinUI 3 C# GUI, sandboxed widgets)
//!    with ZERO reader-reader contention and zero head/tail pointer corruption.
//! 2. `SharedMemoryRingBuffer`: Single-Producer Single-Consumer (SPSC) queue with 64-byte
//!    cacheline alignment (`#[repr(align(64))]`) to prevent false sharing.
//!
//! ## Realistic Performance Profile:
//! Shared memory eliminates Win32 Named Pipe IPC serialization and kernel context switches for
//! steady-state data reads. However, memory-mapped I/O remains subject to OS paging, memory
//! bus synchronization, CPU cache invalidations, and Win32 named event signaling overhead.

use crate::messages::MetricPayload;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use thiserror::Error;

pub const SHM_MAGIC: u32 = 0x41455448; // "AETH" in ASCII
pub const SHM_PROTOCOL_VERSION: u32 = 1;
pub const RING_BUFFER_CAPACITY: usize = 256;
pub const MULTI_READER_SLOTS: usize = 32;

/// Standard Win32 SDDL string for AppContainer read-only shared memory access.
/// - Grants Read-Only (`FR`) to `ALL APPLICATION PACKAGES` (`AC` / `S-1-15-2-1`) and `Everyone` (`WD`).
/// - Grants Full Control (`FA`) to `Built-in Administrators` (`BA`) and `Local System` (`SY`).
/// - Sets Mandatory Integrity Label to Low (`LW`) with No-Write-Up policy (`NW`).
/// Ensures sandboxed low-integrity AppContainer plugins can map and read telemetry without tampering.
pub const SHM_SECURITY_DESCRIPTOR_SDDL: &str =
    "D:(A;;FR;;;AC)(A;;FR;;;WD)(A;;FA;;;BA)(A;;FA;;;SY)S:(ML;;NW;;;LW)";

/// Standard Win32 memory-mapped file name in local session namespace.
pub const SHM_TELEMETRY_MAP_NAME: &str = r"Local\AetherTelemetrySharedMemory";

/// Errors that can occur when reading from multi-reader shared memory regions.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ShmReadError {
    #[error("Invalid SHM magic header: expected 0x{expected:X}, found 0x{found:X}")]
    InvalidMagic { expected: u32, found: u32 },
    #[error("Unsupported SHM protocol version: {version}")]
    UnsupportedVersion { version: u32 },
    #[error("Struct layout size mismatch: expected {expected_size} bytes, found {actual_size} bytes")]
    LayoutMismatch { expected_size: u32, actual_size: u32 },
    #[error("Torn read detected: writer modified payload concurrently after {retries} attempts")]
    TornRead { retries: u32 },
    #[error("Writer process crashed or terminated mid-write at sequence {sequence}")]
    WriterCrashedMidWrite { sequence: u64 },
    #[error("Writer process has terminated or is inactive")]
    WriterInactive,
    #[error("No payload has been published yet")]
    NoDataAvailable,
}

/// 64-byte aligned header placed at the start of memory-mapped IPC segments.
#[repr(C, align(64))]
pub struct ShmHeader {
    pub magic: u32,
    pub protocol_version: u32,
    pub payload_size: u32,
    pub reserved: u32,
    pub sequence: AtomicU64,
    pub timestamp_ms: AtomicU64,
    pub producer_pid: AtomicU32,
    pub is_producer_alive: AtomicBool,
}

impl ShmHeader {
    pub fn new(producer_pid: u32) -> Self {
        Self {
            magic: SHM_MAGIC,
            protocol_version: SHM_PROTOCOL_VERSION,
            payload_size: std::mem::size_of::<MetricPayload>() as u32,
            reserved: 0,
            sequence: AtomicU64::new(0),
            timestamp_ms: AtomicU64::new(0),
            producer_pid: AtomicU32::new(producer_pid),
            is_producer_alive: AtomicBool::new(true),
        }
    }

    /// Validates magic, protocol version, and payload struct size.
    pub fn validate(&self) -> Result<(), ShmReadError> {
        if self.magic != SHM_MAGIC {
            return Err(ShmReadError::InvalidMagic {
                expected: SHM_MAGIC,
                found: self.magic,
            });
        }
        if self.protocol_version != SHM_PROTOCOL_VERSION {
            return Err(ShmReadError::UnsupportedVersion {
                version: self.protocol_version,
            });
        }
        let expected_size = std::mem::size_of::<MetricPayload>() as u32;
        if self.payload_size != expected_size {
            return Err(ShmReadError::LayoutMismatch {
                expected_size,
                actual_size: self.payload_size,
            });
        }
        Ok(())
    }
}

/// Single-Producer Multi-Consumer (SPMC) Seqlock Snapshot Buffer.
/// Allows any number of concurrent readers to read the latest `MetricPayload` without
/// acquiring locks and without reader-reader contention.
#[repr(C, align(64))]
pub struct MultiReaderSnapshotBuffer {
    pub header: ShmHeader,
    pub primary_slot: MetricPayload,
    pub secondary_slot: MetricPayload,
    pub active_slot_index: AtomicU32,
}

impl MultiReaderSnapshotBuffer {
    pub fn new(producer_pid: u32) -> Self {
        Self {
            header: ShmHeader::new(producer_pid),
            primary_slot: MetricPayload::default(),
            secondary_slot: MetricPayload::default(),
            active_slot_index: AtomicU32::new(0),
        }
    }

    /// Publishes a new metric payload (Producer only).
    /// Uses seqlock protocol: odd sequence = write in progress, even = stable.
    pub fn publish(&mut self, payload: MetricPayload) {
        let current_seq = self.header.sequence.load(Ordering::Relaxed);
        // Odd sequence signals writer in progress
        self.header.sequence.store(current_seq.wrapping_add(1), Ordering::Release);

        let active_slot = self.active_slot_index.load(Ordering::Relaxed);
        let target_slot = (active_slot + 1) % 2;

        if target_slot == 0 {
            self.primary_slot = payload;
        } else {
            self.secondary_slot = payload;
        }

        self.active_slot_index.store(target_slot, Ordering::Release);
        self.header.timestamp_ms.store(payload.timestamp_ms, Ordering::Relaxed);

        let final_seq = current_seq.wrapping_add(2);
        // Even sequence signals stable commit
        self.header.sequence.store(final_seq, Ordering::Release);
        tracing::trace!(target: "ipc::shm", seq = final_seq, ts = payload.timestamp_ms, "Committed SHM telemetry payload");
    }

    /// Reads the latest stable payload (Safe for arbitrary concurrent Readers).
    /// Enforces lock-free seqlock verification with bounded retry limits.
    pub fn read_latest(&self) -> Result<MetricPayload, ShmReadError> {
        self.header.validate()?;

        if !self.is_producer_alive() {
            tracing::warn!(target: "ipc::shm", "Producer process is inactive or terminated");
            return Err(ShmReadError::WriterInactive);
        }

        const MAX_RETRIES: u32 = 10;
        let mut last_observed_seq = 0;
        for retry in 0..MAX_RETRIES {
            let seq1 = self.header.sequence.load(Ordering::Acquire);
            last_observed_seq = seq1;
            if seq1 == 0 {
                return Err(ShmReadError::NoDataAvailable);
            }
            if seq1 % 2 != 0 {
                // Writer currently modifying
                if !self.is_producer_alive() {
                    // Writer died or crashed while sequence was odd!
                    tracing::error!(target: "ipc::shm", sequence = seq1, "Writer crashed mid-write");
                    return Err(ShmReadError::WriterCrashedMidWrite { sequence: seq1 });
                }
                std::hint::spin_loop();
                continue;
            }

            let slot_idx = self.active_slot_index.load(Ordering::Acquire);
            let payload = if slot_idx == 0 {
                self.primary_slot
            } else {
                self.secondary_slot
            };

            let seq2 = self.header.sequence.load(Ordering::Acquire);
            if seq1 == seq2 {
                if retry > 0 {
                    tracing::debug!(target: "ipc::shm", retries = retry, "SHM seqlock read succeeded after retry");
                }
                return Ok(payload);
            }
            std::hint::spin_loop();
        }

        if last_observed_seq % 2 != 0 {
            tracing::error!(target: "ipc::shm", sequence = last_observed_seq, "Writer terminated during active write");
            Err(ShmReadError::WriterCrashedMidWrite {
                sequence: last_observed_seq,
            })
        } else {
            tracing::warn!(target: "ipc::shm", retries = MAX_RETRIES, "SHM torn read detected after max retries");
            Err(ShmReadError::TornRead { retries: MAX_RETRIES })
        }
    }

    pub fn is_producer_alive(&self) -> bool {
        self.header.is_producer_alive.load(Ordering::Relaxed)
    }

    pub fn set_producer_alive(&self, alive: bool) {
        self.header.is_producer_alive.store(alive, Ordering::Release);
    }
}

/// Multi-reader circular buffer slot with its own monotonic sequence tag.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TaggedSlot {
    pub sequence: u64,
    pub payload: MetricPayload,
}

impl Default for TaggedSlot {
    fn default() -> Self {
        Self {
            sequence: 0,
            payload: MetricPayload::default(),
        }
    }
}

/// SPMC ring buffer where the producer advances `write_sequence` and multiple readers
/// track their own independent local `read_cursor` without modifying shared memory.
#[repr(C, align(64))]
pub struct MultiReaderRingBuffer {
    pub header: ShmHeader,
    pub write_sequence: AtomicU64,
    pub slots: [TaggedSlot; MULTI_READER_SLOTS],
}

impl MultiReaderRingBuffer {
    pub fn new(producer_pid: u32) -> Self {
        Self {
            header: ShmHeader::new(producer_pid),
            write_sequence: AtomicU64::new(0),
            slots: [TaggedSlot::default(); MULTI_READER_SLOTS],
        }
    }

    /// Appends a new payload into the multi-reader ring buffer (Producer only).
    pub fn push(&mut self, payload: MetricPayload) -> u64 {
        let current_seq = self.write_sequence.load(Ordering::Relaxed);
        let next_seq = current_seq.wrapping_add(1);
        let slot_idx = (current_seq as usize) % MULTI_READER_SLOTS;

        self.slots[slot_idx] = TaggedSlot {
            sequence: next_seq,
            payload,
        };

        self.write_sequence.store(next_seq, Ordering::Release);
        self.header.sequence.store(next_seq, Ordering::Release);
        self.header.timestamp_ms.store(payload.timestamp_ms, Ordering::Relaxed);
        next_seq
    }

    /// Returns the current producer write sequence.
    pub fn current_write_sequence(&self) -> u64 {
        self.write_sequence.load(Ordering::Acquire)
    }

    /// Reads a specific sequence slot from shared memory without mutating shared state.
    pub fn read_at_sequence(&self, seq: u64) -> Option<MetricPayload> {
        let slot_idx = ((seq.saturating_sub(1)) as usize) % MULTI_READER_SLOTS;
        let slot = self.slots[slot_idx];
        if slot.sequence == seq {
            Some(slot.payload)
        } else {
            None // Overwritten by newer writes or not yet written
        }
    }
}

/// Independent consumer cursor for `MultiReaderRingBuffer`.
/// Each reader process maintains its own local cursor; no reader contention or synchronization on shared memory.
#[derive(Debug, Clone)]
pub struct MultiReaderCursor {
    local_read_seq: u64,
}

impl MultiReaderCursor {
    pub fn new() -> Self {
        Self { local_read_seq: 0 }
    }

    /// Polls for the next available metric payload. Automatically fast-forwards if overrun.
    pub fn poll_next(&mut self, buffer: &MultiReaderRingBuffer) -> Option<MetricPayload> {
        let latest_seq = buffer.current_write_sequence();
        if latest_seq == 0 || self.local_read_seq >= latest_seq {
            return None;
        }

        // Check if cursor fell behind capacity
        let max_lag = MULTI_READER_SLOTS as u64;
        if latest_seq - self.local_read_seq > max_lag {
            // Overrun: fast-forward to earliest available slot
            self.local_read_seq = latest_seq - max_lag;
        }

        self.local_read_seq += 1;
        match buffer.read_at_sequence(self.local_read_seq) {
            Some(payload) => Some(payload),
            None => {
                // If overwritten during read, jump to latest
                self.local_read_seq = latest_seq;
                buffer.read_at_sequence(latest_seq)
            }
        }
    }
}

impl Default for MultiReaderCursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Cacheline-aligned lock-free Single-Producer Single-Consumer (SPSC) circular ring buffer.
/// Note: This structure is strictly SPSC. For multi-reader scenarios, use `MultiReaderSnapshotBuffer`
/// or `MultiReaderRingBuffer` to avoid corrupting shared head/tail pointers.
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

    #[test]
    fn test_multi_reader_snapshot_buffer_publish_and_read() {
        let mut buf = MultiReaderSnapshotBuffer::new(1234);
        assert_eq!(buf.read_latest(), Err(ShmReadError::NoDataAvailable));

        let mut p1 = MetricPayload::default();
        p1.cpu_usage_pct = 14.8;
        p1.memory_used_mb = 6144.0;
        p1.memory_total_mb = 16384.0;
        p1.timestamp_ms = 1000;
        buf.publish(p1);

        let r1 = buf.read_latest().expect("Reader 1 should succeed");
        assert_eq!(r1.cpu_usage_pct, 14.8);
        assert_eq!(r1.memory_used_mb, 6144.0);

        // Multiple readers can read simultaneously without modifying shared buffer
        let r2 = buf.read_latest().expect("Reader 2 should succeed");
        assert_eq!(r2.cpu_usage_pct, 14.8);
        assert_eq!(r2.memory_used_mb, 6144.0);

        let mut p2 = MetricPayload::default();
        p2.cpu_usage_pct = 28.4;
        p2.memory_used_mb = 7168.0;
        p2.memory_total_mb = 16384.0;
        p2.timestamp_ms = 2000;
        buf.publish(p2);

        let r3 = buf.read_latest().expect("Reader 3 should read updated payload");
        assert_eq!(r3.cpu_usage_pct, 28.4);
        assert_eq!(r3.memory_used_mb, 7168.0);
    }

    #[test]
    fn test_multi_reader_ring_buffer_independent_cursors() {
        let mut ring = MultiReaderRingBuffer::new(5678);
        let mut cursor_a = MultiReaderCursor::new();
        let mut cursor_b = MultiReaderCursor::new();

        assert!(cursor_a.poll_next(&ring).is_none());
        assert!(cursor_b.poll_next(&ring).is_none());

        for i in 1..=5 {
            let mut payload = MetricPayload::default();
            payload.timestamp_ms = i * 100;
            payload.cpu_usage_pct = i as f32 * 10.0;
            ring.push(payload);
        }

        // Reader A consumes 3 messages
        for i in 1..=3 {
            let item = cursor_a.poll_next(&ring).expect("Cursor A should read item");
            assert_eq!(item.timestamp_ms, i * 100);
        }

        // Reader B starts late, consumes all 5 messages independently
        for i in 1..=5 {
            let item = cursor_b.poll_next(&ring).expect("Cursor B should read independently");
            assert_eq!(item.timestamp_ms, i * 100);
        }

        // Reader A consumes remaining 2
        for i in 4..=5 {
            let item = cursor_a.poll_next(&ring).expect("Cursor A should catch up");
            assert_eq!(item.timestamp_ms, i * 100);
        }

        assert!(cursor_a.poll_next(&ring).is_none());
        assert!(cursor_b.poll_next(&ring).is_none());
    }

    #[test]
    fn test_shm_header_validation() {
        let header = ShmHeader::new(999);
        assert!(header.validate().is_ok());

        let mut bad_header = ShmHeader::new(999);
        bad_header.magic = 0xDEADBEEF;
        assert!(matches!(bad_header.validate(), Err(ShmReadError::InvalidMagic { .. })));
    }

    #[test]
    fn test_shm_writer_crashed_mid_write_detected() {
        let mut buf = MultiReaderSnapshotBuffer::new(1234);
        let mut p = MetricPayload::default();
        p.cpu_usage_pct = 50.0;
        buf.publish(p);

        // Simulate writer crashing mid-write (sequence becomes odd, producer marked dead)
        buf.header.sequence.store(3, Ordering::SeqCst);
        buf.set_producer_alive(false);

        let err = buf.read_latest().unwrap_err();
        assert!(matches!(
            err,
            ShmReadError::WriterInactive | ShmReadError::WriterCrashedMidWrite { sequence: 3 }
        ));
    }

    #[test]
    fn test_shm_layout_size_mismatch() {
        let mut header = ShmHeader::new(1234);
        header.payload_size = 8; // Simulate older layout mapping
        assert!(matches!(
            header.validate(),
            Err(ShmReadError::LayoutMismatch { .. })
        ));
    }

    #[test]
    fn test_shm_alignment_and_fixed_width_compatibility() {
        assert_eq!(std::mem::align_of::<ShmHeader>(), 64);
        assert_eq!(std::mem::align_of::<MultiReaderSnapshotBuffer>(), 64);
        assert_eq!(std::mem::align_of::<SharedMemoryRingBuffer>(), 64);
        assert!(SHM_SECURITY_DESCRIPTOR_SDDL.contains("S-1-15-2-1") || SHM_SECURITY_DESCRIPTOR_SDDL.contains("AC"));
    }
}
