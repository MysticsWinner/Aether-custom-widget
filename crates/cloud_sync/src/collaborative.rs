//! Live Collaborative Desktop Layouts over CRDT WebSocket Sync (Phase 26)
//!
//! Enables multiple workstation peers to share and co-edit a desktop widget layout
//! in real time using Lamport vector clocks, conflict-free CRDT resolution, and live
//! peer cursor/selection presence tracking.

use crate::crdt::{CrdtItem, CrdtResolver, VectorClock};
use crate::entities::LayoutEntity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Real-time cursor and widget selection state of a remote collaborator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerPresence {
    pub peer_id: String,
    pub display_name: String,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub selected_widget_id: Option<String>,
    pub last_heartbeat_ms: u64,
}

/// Incremental collaborative delta broadcast over the WebSocket sync channel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollaborativeDelta {
    LayoutUpdated(CrdtItem<LayoutEntity>),
    PresenceUpdated(PeerPresence),
    PeerLeft { peer_id: String },
}

/// Multi-peer real-time collaborative layout session.
#[derive(Debug, Clone)]
pub struct CollaborativeSession {
    session_id: String,
    local_peer_id: String,
    vector_clock: VectorClock,
    shared_layouts: HashMap<String, CrdtItem<LayoutEntity>>,
    peers: HashMap<String, PeerPresence>,
    outbound_deltas: Vec<CollaborativeDelta>,
}

impl CollaborativeSession {
    pub fn new(session_id: impl Into<String>, local_peer_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            local_peer_id: local_peer_id.into(),
            vector_clock: VectorClock::new(),
            shared_layouts: HashMap::new(),
            peers: HashMap::new(),
            outbound_deltas: Vec::new(),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Publishes a local widget layout move/resize and queues a `CollaborativeDelta::LayoutUpdated` frame.
    pub fn update_local_layout(&mut self, layout: LayoutEntity, timestamp_ms: u64) {
        self.vector_clock.increment(&self.local_peer_id);
        let item = CrdtItem {
            item_id: layout.layout_id.clone(),
            payload: layout.clone(),
            vector_clock: self.vector_clock.clone(),
            timestamp_ms,
        };

        let merged = match self.shared_layouts.get(&layout.layout_id) {
            Some(existing) => CrdtResolver::resolve(existing.clone(), item),
            None => item,
        };

        self.shared_layouts
            .insert(layout.layout_id.clone(), merged.clone());
        self.outbound_deltas
            .push(CollaborativeDelta::LayoutUpdated(merged));
    }

    /// Applies an incoming `CollaborativeDelta` from a remote peer deterministically via CRDT resolution.
    pub fn apply_remote_delta(&mut self, delta: CollaborativeDelta) {
        match delta {
            CollaborativeDelta::LayoutUpdated(remote_item) => {
                self.vector_clock.merge(&remote_item.vector_clock);
                let id = remote_item.item_id.clone();
                let resolved = match self.shared_layouts.get(&id) {
                    Some(local_item) => CrdtResolver::resolve(local_item.clone(), remote_item),
                    None => remote_item,
                };
                self.shared_layouts.insert(id, resolved);
            }
            CollaborativeDelta::PresenceUpdated(presence) => {
                self.peers.insert(presence.peer_id.clone(), presence);
            }
            CollaborativeDelta::PeerLeft { peer_id } => {
                self.peers.remove(&peer_id);
            }
        }
    }

    /// Drains queued outbound deltas ready for WebSocket transmission.
    pub fn drain_outbound_deltas(&mut self) -> Vec<CollaborativeDelta> {
        std::mem::take(&mut self.outbound_deltas)
    }

    /// Looks up the current CRDT-resolved layout for a widget ID.
    pub fn get_layout(&self, widget_id: &str) -> Option<&LayoutEntity> {
        self.shared_layouts.get(widget_id).map(|item| &item.payload)
    }

    /// Returns all active remote collaborators.
    pub fn active_peers(&self) -> usize {
        self.peers.len()
    }
}

/// Wire envelope for a real-time WebSocket collaborative sync frame (`wss://sync.aether.local/v1/rooms`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CollaborativeWsFrame {
    pub session_id: String,
    pub sender_peer_id: String,
    pub sequence_no: u64,
    pub delta: CollaborativeDelta,
}

impl CollaborativeWsFrame {
    /// Serializes the WebSocket frame into a compact JSON text frame payload.
    pub fn to_ws_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parses an incoming WebSocket text frame payload.
    pub fn from_ws_text(payload: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(payload)
    }
}

/// Multi-peer WebSocket broadcast relay hub that routes `CollaborativeWsFrame` envelopes
/// between connected workstation peers in the same `session_id` room.
#[derive(Debug, Default)]
pub struct CollaborativeRelayHub {
    next_seq: u64,
    frame_log: Vec<CollaborativeWsFrame>,
}

impl CollaborativeRelayHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drains outbound deltas from `sender`, encodes them as `CollaborativeWsFrame` wire packets,
    /// and broadcasts them to all `recipients` in the same `session_id` room.
    pub fn broadcast_from_peer(
        &mut self,
        sender: &mut CollaborativeSession,
        recipients: &mut [&mut CollaborativeSession],
    ) -> usize {
        let deltas = sender.drain_outbound_deltas();
        let mut delivered = 0usize;

        for delta in deltas {
            self.next_seq += 1;
            let frame = CollaborativeWsFrame {
                session_id: sender.session_id().to_string(),
                sender_peer_id: sender.local_peer_id().to_string(),
                sequence_no: self.next_seq,
                delta,
            };

            if let Ok(wire_json) = frame.to_ws_text() {
                if let Ok(decoded) = CollaborativeWsFrame::from_ws_text(&wire_json) {
                    for peer in recipients.iter_mut() {
                        if peer.session_id() == decoded.session_id
                            && peer.local_peer_id() != decoded.sender_peer_id
                        {
                            peer.apply_remote_delta(decoded.delta.clone());
                            delivered += 1;
                        }
                    }
                    self.frame_log.push(decoded);
                }
            }
        }

        delivered
    }

    /// Returns the total number of WebSocket frames relayed by the hub.
    pub fn relayed_frame_count(&self) -> usize {
        self.frame_log.len()
    }

    /// Binds a live TCP listener for real-time collaborative sync traffic (e.g. `127.0.0.1:8765`).
    pub async fn bind_listener(addr: &str) -> std::io::Result<TcpListener> {
        TcpListener::bind(addr).await
    }

    /// Transmits a length-prefixed `CollaborativeWsFrame` over a live network TCP stream.
    pub async fn send_frame_stream(
        stream: &mut TcpStream,
        frame: &CollaborativeWsFrame,
    ) -> std::io::Result<()> {
        let json_bytes = serde_json::to_vec(frame)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let len = json_bytes.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&json_bytes).await?;
        stream.flush().await?;
        Ok(())
    }

    /// Reads and deserializes a length-prefixed `CollaborativeWsFrame` from a live network TCP stream.
    pub async fn recv_frame_stream(stream: &mut TcpStream) -> std::io::Result<CollaborativeWsFrame> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > 10 * 1024 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Frame exceeds maximum permitted collaborative payload size (10 MB)",
            ));
        }

        let mut body_buf = vec![0u8; len];
        stream.read_exact(&mut body_buf).await?;
        let frame = serde_json::from_slice(&body_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collaborative_session_crdt_convergence_and_presence() {
        let mut alice = CollaborativeSession::new("room-alpha", "peer_alice");
        let mut bob = CollaborativeSession::new("room-alpha", "peer_bob");

        alice.update_local_layout(
            LayoutEntity {
                layout_id: "perf_monitor_widget".to_string(),
                display_id: "PRIMARY".to_string(),
                bounds_x: 120.0,
                bounds_y: 80.0,
                width: 340.0,
                height: 250.0,
            },
            1000,
        );

        bob.update_local_layout(
            LayoutEntity {
                layout_id: "perf_monitor_widget".to_string(),
                display_id: "PRIMARY".to_string(),
                bounds_x: 640.0,
                bounds_y: 320.0,
                width: 340.0,
                height: 250.0,
            },
            2000,
        );

        for delta in alice.drain_outbound_deltas() {
            bob.apply_remote_delta(delta);
        }
        for delta in bob.drain_outbound_deltas() {
            alice.apply_remote_delta(delta);
        }

        // Both peers must converge to the exact same coordinates (Bob's t=2000 wins LWW tie-break)
        let alice_pos = alice.get_layout("perf_monitor_widget").unwrap();
        let bob_pos = bob.get_layout("perf_monitor_widget").unwrap();
        assert_eq!(alice_pos, bob_pos);
        assert_eq!(alice_pos.bounds_x, 640.0);
        assert_eq!(alice_pos.bounds_y, 320.0);

        alice.apply_remote_delta(CollaborativeDelta::PresenceUpdated(PeerPresence {
            peer_id: "peer_bob".to_string(),
            display_name: "Bob".to_string(),
            cursor_x: 650.0,
            cursor_y: 330.0,
            selected_widget_id: Some("perf_monitor_widget".to_string()),
            last_heartbeat_ms: 2050,
        }));
        assert_eq!(alice.active_peers(), 1);

        alice.apply_remote_delta(CollaborativeDelta::PeerLeft {
            peer_id: "peer_bob".to_string(),
        });
        assert_eq!(alice.active_peers(), 0);
    }

    #[test]
    fn test_collaborative_websocket_relay_hub_multi_peer_broadcast() {
        let mut hub = CollaborativeRelayHub::new();
        let mut ws_host = CollaborativeSession::new("studio-room-1", "peer_host");
        let mut ws_client1 = CollaborativeSession::new("studio-room-1", "peer_client_1");
        let mut ws_client2 = CollaborativeSession::new("studio-room-1", "peer_client_2");

        ws_host.update_local_layout(
            LayoutEntity {
                layout_id: "weather_widget".to_string(),
                display_id: "PRIMARY".to_string(),
                bounds_x: 420.0,
                bounds_y: 180.0,
                width: 300.0,
                height: 220.0,
            },
            5000,
        );

        let delivered =
            hub.broadcast_from_peer(&mut ws_host, &mut [&mut ws_client1, &mut ws_client2]);
        assert_eq!(delivered, 2);
        assert_eq!(hub.relayed_frame_count(), 1);

        assert_eq!(
            ws_client1.get_layout("weather_widget").unwrap().bounds_x,
            420.0
        );
        assert_eq!(
            ws_client2.get_layout("weather_widget").unwrap().bounds_y,
            180.0
        );
    }

    #[tokio::test]
    async fn test_collaborative_relay_hub_tcp_network_transport() {
        let listener = CollaborativeRelayHub::bind_listener("127.0.0.1:0")
            .await
            .expect("Must bind ephemeral TCP port");
        let local_addr = listener.local_addr().expect("Must have local addr");

        let send_frame = CollaborativeWsFrame {
            session_id: "tcp-test-session".to_string(),
            sender_peer_id: "peer_net_a".to_string(),
            sequence_no: 42,
            delta: CollaborativeDelta::PresenceUpdated(PeerPresence {
                peer_id: "peer_net_a".to_string(),
                display_name: "Network Peer A".to_string(),
                cursor_x: 250.0,
                cursor_y: 180.0,
                selected_widget_id: Some("dock_launcher".to_string()),
                last_heartbeat_ms: 123456,
            }),
        };

        let frame_clone = send_frame.clone();
        let server_task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("Accept must succeed");
            let received = CollaborativeRelayHub::recv_frame_stream(&mut socket)
                .await
                .expect("Recv frame must succeed");
            received
        });

        let mut client_stream = TcpStream::connect(local_addr)
            .await
            .expect("Connect to listener must succeed");
        CollaborativeRelayHub::send_frame_stream(&mut client_stream, &frame_clone)
            .await
            .expect("Send frame must succeed");

        let received_frame = server_task.await.expect("Server task should complete");
        assert_eq!(received_frame, send_frame);
        assert_eq!(received_frame.sender_peer_id, "peer_net_a");
        assert_eq!(received_frame.sequence_no, 42);
    }
}

