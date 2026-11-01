//! WebSocket signaling relay for WebRTC P2P direct mesh.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use tracing::{info, warn};

#[derive(Deserialize)]
pub struct WsQuery {
    pub peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalingPacket {
    #[serde(rename = "type")]
    pub packet_type: String, // "offer" | "answer" | "candidate" | "join"
    pub sender: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
}

type PeerSender = mpsc::UnboundedSender<Message>;

#[derive(Clone, Default)]
pub struct SignalingRelayState {
    peers: Arc<Mutex<HashMap<String, PeerSender>>>,
}

pub fn ws_router(state: SignalingRelayState) -> Router {
    Router::new()
        .route("/ws/signaling", get(ws_handler))
        .with_state(state)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(relay_state): State<SignalingRelayState>,
) -> impl IntoResponse {
    let peer_id = query
        .peer_id
        .unwrap_or_else(|| format!("anon_{}", uuid::Uuid::new_v4().simple()));
    ws.on_upgrade(move |socket| handle_socket(socket, peer_id, relay_state))
}

async fn handle_socket(socket: WebSocket, peer_id: String, relay_state: SignalingRelayState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Register peer
    {
        let mut map = relay_state.peers.lock().unwrap();
        map.insert(peer_id.clone(), tx);
        info!("Peer connected to signaling relay: {}", peer_id);
    }

    // Task forwarding from mpsc channel to WebSocket sender
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from WebSocket receiver
    let relay_clone = relay_state.clone();
    let peer_id_clone = peer_id.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(packet) = serde_json::from_str::<SignalingPacket>(&text) {
                    if let Some(target) = packet.target {
                        let map = relay_clone.peers.lock().unwrap();
                        if let Some(target_tx) = map.get(&target) {
                            let _ = target_tx.send(Message::Text(text));
                        } else {
                            warn!("Target peer not found: {}", target);
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    // Unregister peer
    {
        let mut map = relay_state.peers.lock().unwrap();
        map.remove(&peer_id_clone);
        info!("Peer disconnected from signaling relay: {}", peer_id_clone);
    }
}
