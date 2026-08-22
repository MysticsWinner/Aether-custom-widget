# Aether IPC — Inter-Process Communication Specification

## Overview

Inter-process communication between **Aether Runtime**, **Aether Studio**, and sandboxed plugin processes is split into two specialized channels:

---

## IPC Transport Channels

### 1. Control Channel (Win32 Named Pipes)
- **Protocol**: Binary RPC / Bincode frames.
- **Purpose**: Subsystem management, lifecycle control, theme swap commands, and sandbox initialization.

### 2. High-Frequency Telemetry Channel (Shared Memory Ring Buffer)
- **Protocol**: Zero-copy memory-mapped file ring buffer (`SharedTelemetryCache`).
- **Purpose**: System hardware metrics (CPU, RAM, GPU, Network) published by `system_providers` once per tick and read by sandboxed plugins with zero kernel context switches.
