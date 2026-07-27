# Encrypted Cloud Sync & CRDT Specification

The **Phase 13 Cloud Sync Engine** (`crates/cloud_sync`) provides seamless, end-to-end encrypted synchronization across multiple workstations and displays.

---

## 🌐 The 6 Synchronized Cloud Entities

1. **Layouts**: Screen positions, multi-monitor display bounds (`LayoutEntity`).
2. **Themes**: Active `theme.json` schemas, color tokens, font mappings (`ThemeEntity`).
3. **Settings**: Host daemon and widget configuration key-value pairs (`SettingsEntity`).
4. **Plugins**: Installed package IDs, version pins, enabled states (`PluginEntity`).
5. **Devices**: Registered hardware workstation profiles (`DeviceEntity`).
6. **Accounts**: User credentials, session tokens (`AccountEntity`).

---

## ⚔️ CRDT Vector Clock Conflict Resolution

Sync conflicts between multiple devices (e.g. laptop vs desktop editing layout bounds concurrently) resolve deterministically via state-based **CRDTs** (Conflict-Free Replicated Data Types) and Lamport **Vector Clocks**.

```
Device A (Workstation)                      Device B (Laptop)
[ VectorClock: {A: 1} ]                     [ VectorClock: {B: 1} ]
Edit Layout -> Bounds X=100                 Edit Layout -> Bounds X=200
        |                                           |
        +---------------------+---------------------+
                              |
                              v
                     [ CrdtResolver ]
                              |
                Check Causality & Vector Clocks
                              |
       +----------------------+----------------------+
       |                                             |
Vector Clock Dominates                   Concurrent Edit (Tie)
    |                                                |
Apply Dominant State                     Apply Last-Write-Wins
                                         (Lamport Timestamp)
```

---

## ✈️ Offline-First Local Cache & Queueing

1. **Local Primary Truth**: All state mutations write instantly to local SQLite WAL storage. Network connectivity is never on the critical path.
2. **Offline Transaction Buffer**: When offline, changes append to `OfflineSyncQueue`.
3. **Reconnection Flush**: Upon network reconnect, `OfflineSyncQueue` flushes pending operations to the cloud, executes CRDT resolution, and updates local state.
4. **AES-256-GCM Encryption**: All cloud sync payloads are encrypted on the client device prior to transmission.
