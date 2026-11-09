//! Zero-Allocation Frame Arena Memory Allocator
//!
//! Provides high-performance sub-quantum memory pooling for widget render loops,
//! eliminating heap allocations (`malloc`/`free`) and preventing long-term memory fragmentation.

use std::cell::UnsafeCell;

/// Default capacity for a single widget frame arena (256 KB).
pub const DEFAULT_ARENA_CAPACITY: usize = 256 * 1024;

/// Fast, thread-local bump memory allocator for transient per-frame allocations.
pub struct FrameArena {
    buffer: Vec<u8>,
    offset: UnsafeCell<usize>,
    capacity: usize,
}

impl FrameArena {
    /// Creates a new `FrameArena` with default 256 KB pre-allocated capacity.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_ARENA_CAPACITY)
    }

    /// Creates a new `FrameArena` with custom pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity],
            offset: UnsafeCell::new(0),
            capacity,
        }
    }

    /// Allocates `size` bytes with specified alignment.
    pub fn alloc(&self, size: usize, align: usize) -> Option<*mut u8> {
        unsafe {
            let current = *self.offset.get();
            let aligned = (current + align - 1) & !(align - 1);
            let next = aligned + size;

            if next <= self.capacity {
                *self.offset.get() = next;
                let ptr = self.buffer.as_ptr().add(aligned) as *mut u8;
                Some(ptr)
            } else {
                None // Out of arena memory
            }
        }
    }

    /// Resets the allocation offset to zero in sub-nanosecond time without reallocating.
    pub fn reset(&self) {
        unsafe {
            *self.offset.get() = 0;
        }
    }

    /// Returns the currently allocated bytes in the active frame.
    pub fn used_bytes(&self) -> usize {
        unsafe { *self.offset.get() }
    }

    /// Returns the total capacity of the arena.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Trims arena memory footprint during garbage collection passes.
    /// In aggressive mode (e.g. dev mode), shrinks the pre-allocated buffer down to target minimum.
    pub fn collect_garbage(&mut self, aggressive: bool) -> usize {
        self.reset();
        let initial_cap = self.capacity;
        if aggressive && self.capacity > 64 * 1024 {
            let target_cap = 64 * 1024;
            self.buffer.truncate(target_cap);
            self.buffer.shrink_to_fit();
            self.capacity = target_cap;
            initial_cap.saturating_sub(target_cap)
        } else {
            0
        }
    }
}

impl Default for FrameArena {
    fn default() -> Self {
        Self::new()
    }
}

// FrameArena is intended for single-threaded tick loops per widget instance
unsafe impl Send for FrameArena {}
unsafe impl Sync for FrameArena {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_arena_alloc_and_reset() {
        let arena = FrameArena::with_capacity(1024);
        assert_eq!(arena.used_bytes(), 0);

        let ptr1 = arena.alloc(128, 8);
        assert!(ptr1.is_some());
        assert_eq!(arena.used_bytes(), 128);

        let ptr2 = arena.alloc(256, 8);
        assert!(ptr2.is_some());
        assert_eq!(arena.used_bytes(), 384);

        arena.reset();
        assert_eq!(arena.used_bytes(), 0);

        let ptr3 = arena.alloc(64, 8);
        assert!(ptr3.is_some());
        assert_eq!(arena.used_bytes(), 64);
    }

    #[test]
    fn test_frame_arena_out_of_memory() {
        let arena = FrameArena::with_capacity(100);
        let ptr = arena.alloc(150, 8);
        assert!(ptr.is_none(), "Should fail when requested size exceeds capacity");
    }

    #[test]
    fn test_frame_arena_garbage_collection() {
        let mut arena = FrameArena::with_capacity(512 * 1024);
        assert_eq!(arena.capacity(), 512 * 1024);
        let _ = arena.alloc(1024, 8);
        assert_eq!(arena.used_bytes(), 1024);

        // Gentle GC resets used bytes without reallocating
        let reclaimed = arena.collect_garbage(false);
        assert_eq!(reclaimed, 0);
        assert_eq!(arena.used_bytes(), 0);
        assert_eq!(arena.capacity(), 512 * 1024);

        // Aggressive GC shrinks buffer to minimum 64KB
        let reclaimed_agg = arena.collect_garbage(true);
        assert!(reclaimed_agg > 0);
        assert_eq!(arena.capacity(), 64 * 1024);
    }
}

