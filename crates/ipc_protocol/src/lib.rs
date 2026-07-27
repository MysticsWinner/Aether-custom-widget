//! IPC Protocol and Shared Memory Ring Buffer Definitions
//!
//! Provides zero-copy ring buffer structures and serialization primitives
//! for inter-process communication between the core host engine, sandboxed plugins,
//! and the WinUI 3 management GUI.

pub mod messages;
pub mod ring_buffer;

pub use messages::*;
pub use ring_buffer::*;
