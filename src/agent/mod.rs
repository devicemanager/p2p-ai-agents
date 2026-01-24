//! Agent Module
//!
//! This module contains the core agent logic, including identity management,
//! messaging, and task execution.

/// AI integration and model management.
pub mod ai;
pub mod executors;
pub mod identity;
pub mod messaging;
pub mod resource;
pub mod task;

use crate::agent::ai::ModelManager;
use crate::agent::executors::{
    ExecutorRegistry, TextProcessingExecutor, VectorComputationExecutor,
};
use crate::agent::identity::AgentIdentity;
use crate::agent::messaging::{Message, MessageType};
use crate::agent::task::{Task, TaskExecutor, TaskId, TaskManager, TaskStatus, TaskType};
use crate::core::identity::IdentityError;
use crate::network::{NetworkConfig, NetworkManager, NetworkMessage, PeerId as NetworkPeerId};
use futures::future::{AbortHandle, Abortable};
use serde_json::json;
use tokio::sync::{broadcast, mpsc, Mutex};

/// Unique identifier for an Agent.
pub type AgentId = String;

/// Configuration for an Agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// The unique name or identifier for the agent.
    pub name: String,
    /// List of capabilities this agent supports.
    pub capabilities: Vec<TaskType>,
    /// List of AI models this agent has available.
    #[serde(default)]
    pub models: Vec<String>,
    // Add other config fields if necessary, currently minimal for compilation
}

/// Core Agent structure representing a node in the network.
pub struct Agent {
    /// The agent's identity and trust components.
    pub identity: AgentIdentity,
    /// The agent's configuration.
    pub config: AgentConfig,
    /// The agent's task manager.
    pub task_manager: TaskManager,
    /// The agent's task executor registry.
    pub executor_registry: ExecutorRegistry,
    /// The agent's AI model manager.
    pub model_manager: Arc<ModelManager>,
    /// Network manager (protected by mutex for mutable access during start/stop).
    pub network_manager: Arc<Mutex<NetworkManager>>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
}

impl Agent {
    /// Get the listening address(es) of the agent
    pub async fn listen_addresses(&self) -> Vec<String> {
        let addrs = self
            .network_manager
            .lock()
            .await
            .get_listen_addresses()
            .await;
        addrs.iter().map(|a| a.to_string()).collect()
    }

    /// Creates a new Agent instance.
    pub fn new(
        identity: AgentIdentity,
        config: AgentConfig,
        network_config: NetworkConfig,
        task_manager: TaskManager,
        model_manager: Arc<ModelManager>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let network_manager = NetworkManager::new(network_config);

        let executor_registry = ExecutorRegistry::new();
        Self {
            identity,
            config,
            task_manager,
            executor_registry,
            model_manager,
            network_manager: Arc::new(Mutex::new(network_manager)),
            shutdown_tx,
        }
    }

    /// Returns the Agent's ID.
    pub fn id(&self) -> AgentId {
        // Fallback to name for now, should be DID
        self.config.name.clone()
    }

    /// Finds peers that support a specific task capability and optionally a specific model.
    pub async fn find_peers_with_capability(
        &self,
        task_type: TaskType,
        required_model: Option<&str>,
    ) -> Vec<NetworkPeerId> {
        let network = self.network_manager.lock().await;
        let connected_peers = network.peer_cache.get_connected_peers().await;

        connected_peers
            .into_iter()
            .filter(|p| {
                // Check task capability
                let has_task = p.capabilities.supported_tasks.contains(&task_type);

                // Check model capability if required
                let has_model = if let Some(model) = required_model {
                    p.capabilities.supported_models.contains(&model.to_string())
                } else {
                    true
                };

                has_task && has_model
            })
            .map(|p| p.peer_id)
            .collect()
    }

    /// Starts the Agent.
    pub async fn start(self: Arc<Self>) -> anyhow::Result<()> {
        let _shutdown_rx = self.shutdown_tx.subscribe();

        // Load persisted tasks
        match self.task_manager.load_tasks().await {
            Ok(count) => {
                if count > 0 {
                    println!("Recovered {} tasks from storage", count);
                    tracing::info!("Recovered {} tasks from storage", count);
                }
            }
            Err(e) => {
                eprintln!("Failed to load tasks from storage: {}", e);
                tracing::error!("Failed to load tasks from storage: {}", e);
                // Continue startup even if load fails
            }
        }

        let agent_clone = self.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Setup Network Manager & Message Loop
        {
            let mut nm = self.network_manager.lock().await;

            // Construct agent version string with capabilities
            // e.g. "/p2p-ai-agents/1.0.0/TextProcessor,VectorComputer"
            let mut version = "/p2p-ai-agents/1.0.0".to_string();
            if !self.config.capabilities.is_empty() {
                version.push('/');
                let caps: Vec<String> = self
                    .config
                    .capabilities
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                version.push_str(&caps.join(","));
            }
            nm.set_agent_version(version);

            // Channel for network messages -> agent
            let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
            nm.set_message_callback(tx);

            // Start the network manager
            if let Err(e) = nm.start().await {
                eprintln!("Failed to start network manager: {:?}", e);
            }

            // Announce Capabilities (if any)
            // We spawn a task to do this shortly after startup to ensure peers are connected
            let agent_announce = self.clone();
            tokio::spawn(async move {
                if !agent_announce.config.capabilities.is_empty() {
                    // Wait a bit for initial connections
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    let msg = Message::new_capability_announcement(
                        agent_announce.id(),
                        agent_announce.config.capabilities.clone(),
                        agent_announce.config.models.clone(),
                    );
                    if let Err(e) = agent_announce.broadcast_message(msg).await {
                        tracing::error!("Failed to broadcast capability announcement: {:?}", e);
                    } else {
                        tracing::info!("Broadcasted capability announcement");
                    }
                }
            });

            // Spawn message handler loop
            let agent_msg_clone = self.clone();
            let mut msg_shutdown_rx = self.shutdown_tx.subscribe();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = msg_shutdown_rx.recv() => break,
                        msg = rx.recv() => {
                            if let Some(bytes) = msg {
                                // Deserialize and handle
                                if let Ok(message) = serde_json::from_slice::<Message>(&bytes) {
                                     if let Err(e) = agent_msg_clone.handle_message(message).await {
                                         eprintln!("Error handling message: {:?}", e);
                                     }
                                } else {
                                     eprintln!("Failed to deserialize network message");
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
        }

        // Spawn background task processing loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        // Received shutdown signal
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                        // Poll for tasks
                        if let Err(e) = agent_clone.process_next_task().await {
                             eprintln!("Error processing task: {:?}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stops the Agent.
    pub async fn stop(&self) -> anyhow::Result<()> {
        let _ = self.shutdown_tx.send(());
        self.network_manager.lock().await.shutdown().await.ok();
        Ok(())
    }

    /// Sends a signed message to the network (direct or broadcast).
    /// The actual transport is selected based on the recipient (broadcast or direct).
    async fn send_network_message(&self, mut message: Message) -> anyhow::Result<()> {
        let signable_bytes = message.to_signable_bytes();
        let signature = self.identity.sign_data(&signable_bytes)?;
        message.signature = Some(signature);
        message.public_key = Some(self.identity.public_key_bytes());

        let bytes = serde_json::to_vec(&message)?;

        if message.recipient == "broadcast" {
            let msg = NetworkMessage {
                from: self.id(),
                to: message.recipient.clone(),
                content: bytes,
            };
            self.network_manager.lock().await.send_message(msg).await;
        } else {
            // Direct message
            // Try to send directly. If it fails (e.g. invalid PeerID), we propagate the error
            // so the caller can decide (e.g. fallback).
            self.network_manager
                .lock()
                .await
                .send_request(&message.recipient, bytes)
                .await?;
        }

        Ok(())
    }

    /// Broadcast a message to the network.
    pub async fn broadcast_message(&self, message: Message) -> anyhow::Result<()> {
        self.send_network_message(message).await
    }

    /// Dispatches a task to a capable peer.
    pub async fn dispatch_task(&self, task_id: TaskId) -> anyhow::Result<()> {
        let task = self
            .task_manager
            .get_task(task_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // 1. Identify TaskType and Model Requirement
        let (task_type, required_model) = if let Some(payload) = &task.payload {
            let model = match payload.task_type {
                TaskType::AiInference => payload.data.get("model").and_then(|v| v.as_str()),
                TaskType::TextProcessing => {
                    // Check if it's an embedding task which requires a model
                    let op = payload
                        .data
                        .get("operation")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if op == "embed" {
                        payload.data.get("model").and_then(|v| v.as_str())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            (payload.task_type.clone(), model)
        } else {
            return Err(anyhow::anyhow!("Task has no payload to determine type"));
        };

        // 2. Find Peers
        let peers = self
            .find_peers_with_capability(task_type.clone(), required_model)
            .await;
        if peers.is_empty() {
            return Err(anyhow::anyhow!(
                "No peers found with capability {} (model: {:?})",
                task_type,
                required_model
            ));
        }

        // 3. Select Peer (Simple: First one)
        let target_peer = peers[0].clone();
        println!("Dispatching task {} to peer {}", task_id, target_peer);

        // 4. Update Task with Assignment
        self.task_manager
            .assign_task(task_id, target_peer.to_string())
            .await?;

        // 5. Construct Message
        let message = Message::new_task_request(self.id(), target_peer.clone(), task.clone());

        // 6. Sign and Send
        if let Err(e) = self.send_network_message(message).await {
            // Fallback to broadcast if direct send fails (e.g. invalid peer ID format for libp2p)
            // This is a robustness feature to keep tests passing while we transition.
            println!("Direct send failed ({}), falling back to broadcast", e);

            // Recreate message because original was moved
            let mut message_clone = Message::new_task_request(self.id(), target_peer, task);

            // Manually sign and broadcast to bypass `send_network_message`'s transport selection
            // (which would try direct send again because recipient is not "broadcast")
            let signable_bytes = message_clone.to_signable_bytes();
            let signature = self.identity.sign_data(&signable_bytes)?;
            message_clone.signature = Some(signature);
            message_clone.public_key = Some(self.identity.public_key_bytes());

            let bytes = serde_json::to_vec(&message_clone)?;

            let msg = NetworkMessage {
                from: self.id(),
                to: "broadcast".to_string(), // Transport-level destination
                content: bytes,
            };

            self.network_manager.lock().await.send_message(msg).await;
        }

        Ok(())
    }

    /// Submits a task to the agent.
    pub async fn submit_task(&self, task: Task) -> TaskId {
        // Add the task to the manager
        self.task_manager.add_task(task).await
    }

    /// Cancels a task.
    pub async fn cancel_task(&self, id: TaskId) -> anyhow::Result<()> {
        // 1. Check if the task is remote
        if let Some(task) = self.task_manager.get_task(id).await {
            if let Some(assigned_peer) = task.assigned_to {
                // Task was dispatched to a remote peer. Send cancellation request.
                // Only send if not already completed/failed/cancelled
                match task.status {
                    TaskStatus::Queued | TaskStatus::Running => {
                        let message =
                            Message::new_task_cancellation(self.id(), assigned_peer.clone(), id);
                        // Best effort send with fallback
                        if let Err(e) = self.send_network_message(message).await {
                            eprintln!("Failed to send cancellation to {}: {:?}. Falling back to broadcast.", assigned_peer, e);

                            // Recreate message for broadcast fallback
                            let mut message_clone = Message::new_task_cancellation(
                                self.id(),
                                assigned_peer.clone(),
                                id,
                            );

                            // Manually sign and broadcast
                            let signable_bytes = message_clone.to_signable_bytes();
                            if let Ok(signature) = self.identity.sign_data(&signable_bytes) {
                                message_clone.signature = Some(signature);
                                message_clone.public_key = Some(self.identity.public_key_bytes());

                                if let Ok(bytes) = serde_json::to_vec(&message_clone) {
                                    let msg = NetworkMessage {
                                        from: self.id(),
                                        to: "broadcast".to_string(),
                                        content: bytes,
                                    };
                                    self.network_manager.lock().await.send_message(msg).await;
                                }
                            }
                        }
                    }
                    _ => {} // Already done, no need to send cancel
                }
            }
        }

        // 2. Cancel locally (stops local execution or marks as cancelled)
        self.task_manager.cancel_task(id).await
    }

    /// Processes a single pending task if available (for manual execution or testing).
    pub async fn process_next_task(&self) -> anyhow::Result<Option<TaskId>> {
        let next_task = self.task_manager.get_next_pending_task().await;

        if let Some(task) = next_task {
            // Update status to Running
            self.task_manager
                .update_status(task.id, TaskStatus::Running)
                .await?;

            let _task_manager = self.task_manager.clone();
            let _identity = self.identity.clone();
            let _network_manager = self.network_manager.clone();
            let _agent_id = self.id();

            let task_id = task.id;
            let _executor_registry = self.executor_registry.clone();

            // Create an abort handle for cancellation
            let (abort_handle, abort_registration) = AbortHandle::new_pair();

            // Register the handle
            self.task_manager
                .register_running_task(task_id, abort_handle)
                .await;

            let model_manager = self.model_manager.clone();

            // Spawn the execution
            let _handle = tokio::spawn(Abortable::new(
                async move {
                    let result = if let Some(payload) = &task.payload {
                        match payload.task_type {
                            TaskType::TextProcessing => {
                                let executor = TextProcessingExecutor::new(model_manager);
                                executor.execute(payload).await
                            }
                            TaskType::VectorComputation => {
                                VectorComputationExecutor.execute(payload).await
                            }
                            TaskType::AiModelDownload => {
                                let model_name = payload
                                    .data
                                    .get("model")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("prajjwal1/bert-tiny");

                                match model_manager.ensure_model(model_name).await {
                                    Ok(path) => Ok(json!({
                                        "status": "downloaded",
                                        "model": model_name,
                                        "path": path.to_string_lossy()
                                    })),
                                    Err(e) => Err(e),
                                }
                            }
                            TaskType::AiInference => {
                                // Delegate to TextProcessingExecutor which handles "embed" and other AI ops
                                // We might want to refactor this later to be more distinct, but for now
                                // TextProcessingExecutor knows how to use the ModelManager and InferenceEngine.
                                let executor = TextProcessingExecutor::new(model_manager.clone());
                                executor.execute(payload).await
                            }
                            TaskType::Custom(_) => {
                                // Check for requested duration in payload for testing/simulation
                                let duration = payload
                                    .data
                                    .get("duration_ms")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(100);

                                tokio::time::sleep(std::time::Duration::from_millis(duration))
                                    .await;
                                Ok(json!({"status": "simulated_custom_execution"}))
                            }
                        }
                    } else {
                        // No payload, just simulate work
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        Ok::<serde_json::Value, anyhow::Error>(json!({"status": "no_payload"}))
                    };

                    let status = match result {
                        Ok(output) => TaskStatus::Completed(output),
                        Err(e) => TaskStatus::Failed(e.to_string()),
                    };

                    // Update local state
                    let _ = _task_manager.update_status(task_id, status.clone()).await;

                    // Broadcast update
                    let mut message =
                        Message::new_task_response(_agent_id.clone(), "broadcast", task_id, status);

                    // Sign and send
                    // Note: We duplicate logic from broadcast_message here because we can't easily
                    // call methods on 'self' inside this moved async block without cloning the whole Agent
                    // (which has other non-clonable fields like shutdown_tx).
                    let signable_bytes = message.to_signable_bytes();
                    if let Ok(signature) = _identity.sign_data(&signable_bytes) {
                        message.signature = Some(signature);
                        message.public_key = Some(_identity.public_key_bytes());

                        if let Ok(bytes) = serde_json::to_vec(&message) {
                            let msg = NetworkMessage {
                                from: _agent_id,
                                to: "broadcast".to_string(),
                                content: bytes,
                            };
                            _network_manager.lock().await.send_message(msg).await;
                        }
                    } else {
                        eprintln!("Failed to sign task completion message");
                    }
                },
                abort_registration,
            ));

            Ok(Some(task_id))
        } else {
            Ok(None)
        }
    }

    /// Gets the status of a task.
    pub async fn task_status(&self, id: &TaskId) -> anyhow::Result<TaskStatus> {
        let task = self.task_manager.get_task(*id).await;
        match task {
            Some(t) => Ok(t.status),
            None => Err(anyhow::anyhow!("Task not found")),
        }
    }

    /// Handles an incoming incoming network message.
    pub async fn handle_message(&self, message: Message) -> anyhow::Result<()> {
        // 0. Filter by Recipient
        // Since we currently use Gossipsub (broadcast) for transport, we must filter messages
        // intended for others.
        if message.recipient != "broadcast" && message.recipient != self.id() {
            // Message is not for us. Ignore it.
            // In a future direct-transport implementation, we wouldn't receive this.
            return Ok(());
        }

        // Verify Signature
        if let (Some(sig), Some(pk_bytes)) = (&message.signature, &message.public_key) {
            let signable_bytes = message.to_signable_bytes();
            if let Ok(valid) = self
                .identity
                .verify_signature(pk_bytes, &signable_bytes, sig)
            {
                if !valid {
                    eprintln!(
                        "⚠️ Warning: Invalid signature for message from {}",
                        message.sender
                    );
                    // Reject message
                    return Err(anyhow::anyhow!("Invalid message signature"));
                }
                // Signature valid.

                // Authorization Check: Is the sender trusted?
                if !self.identity.is_peer_trusted(pk_bytes) {
                    eprintln!(
                        "⚠️ Warning: Trusted peer check failed for message from {}",
                        message.sender
                    );
                    return Err(anyhow::anyhow!("Peer not in trust registry"));
                }
            } else {
                eprintln!(
                    "⚠️ Warning: Error verifying signature for message from {}",
                    message.sender
                );
                return Err(anyhow::anyhow!("Signature verification error"));
            }
        } else {
            eprintln!(
                "⚠️ Warning: Unsigned message from {}. Rejecting in secure mode.",
                message.sender
            );
            return Err(anyhow::anyhow!("Missing signature or public key"));
        }

        match message.content {
            MessageType::TaskRequest(task) => {
                println!("Agent received TaskRequest: {}", task.id);
                // Submit the task to the local manager
                // We trust the sender for now (Identity verification to be added later)
                self.submit_task(*task).await;
            }
            MessageType::TaskResponse { task_id, status } => {
                println!("Agent received TaskResponse for {}: {:?}", task_id, status);
                // Update local state if we are tracking this remote task
                if let Some(_task) = self.task_manager.get_task(task_id).await {
                    // Only update if the task is not already in a final state locally
                    // or if we decide to trust the remote update.
                    // For now, we update it if we have it.

                    // Note: We might want to check if the status transition is valid
                    // e.g. from Queued/Running -> Completed/Failed.

                    let _ = self.task_manager.update_status(task_id, status).await;
                } else {
                    tracing::warn!("Received response for unknown task: {}", task_id);
                }
            }
            MessageType::Text(text) => {
                println!(
                    "Agent received Text message from {}: {}",
                    message.sender, text
                );
            }
            MessageType::CapabilityAnnouncement {
                capabilities,
                models,
            } => {
                println!(
                    "Agent received CapabilityAnnouncement from {}: {:?} (Models: {:?})",
                    message.sender, capabilities, models
                );

                // Update peer cache in NetworkManager
                // Note: NetworkManager tracks peers by PeerId (string), which matches message.sender
                // But we need to be careful if message.sender is a human-readable name vs a proper DID/Key.
                // For now, our implementation uses human-readable names or UUIDs as IDs uniformly.
                // However, `PeerCache` uses `PeerId` struct.

                // We need to resolve the sender ID to a network PeerId if they differ,
                // but in our current mock setup, they are likely the same or we treat them as such.

                // IMPORTANT: `NetworkManager` manages `PeerCache`. We need to access it.
                // But `NetworkManager` is locked behind a Mutex.

                let peer_id_str = message.sender.clone();
                let supported_tasks = capabilities;
                let supported_models = models;

                let nm = self.network_manager.lock().await;

                // Create or update PeerInfo
                // We don't have the full PeerInfo (like address) here if we haven't seen them before,
                // but usually we would have established a connection.
                // If we don't know them, we can't really "connect" to them just from a broadcast message
                // unless the message contained their multiaddr (which it doesn't currently).
                // However, for `find_peers_with_capability` to work, we need them in the cache.

                use crate::network::{ConnectionStatus, PeerCapabilities, PeerId, PeerInfo};

                // Try to find existing entry to preserve address/reputation
                let mut peer_info = if let Some(existing) =
                    nm.peer_cache.get_peer(&PeerId(peer_id_str.clone())).await
                {
                    existing
                } else {
                    // New peer (or at least new to cache).
                    // We might not know their address yet.
                    PeerInfo {
                        peer_id: PeerId(peer_id_str.clone()),
                        addresses: vec![],
                        last_seen: chrono::Utc::now(),
                        reputation: 50,
                        capabilities: PeerCapabilities {
                            supported_tasks: vec![],
                            supported_models: vec![],
                        },
                        status: ConnectionStatus::Connected, // Assume connected if we heard them via gossipsub
                    }
                };

                // Update capabilities
                peer_info.capabilities.supported_tasks = supported_tasks;
                peer_info.capabilities.supported_models = supported_models;
                peer_info.last_seen = chrono::Utc::now();

                // Write back to cache
                nm.peer_cache.upsert_peer(peer_info).await;
            }
            MessageType::TaskCancellation { task_id } => {
                println!(
                    "Agent received TaskCancellation from {} for task {}",
                    message.sender, task_id
                );
                // We should verify that the sender is actually the one who requested the task
                // For now, we trust the sender (assuming verified signature & trust checks passed above)

                if let Err(e) = self.task_manager.cancel_task(task_id).await {
                    eprintln!("Error cancelling task {}: {:?}", task_id, e);
                } else {
                    println!("Successfully cancelled task {}", task_id);
                    // Optionally send a status update back confirming cancellation?
                    // The task execution loop will eventually stop and might send a Failed/Cancelled update,
                    // but `cancel_task` sets status to Cancelled immediately.

                    // Let's send a confirmation
                    let confirm_msg = Message::new_task_response(
                        self.id(),
                        "broadcast", // or specifically to sender? Broadcast for now to be safe with current setup
                        task_id,
                        TaskStatus::Cancelled,
                    );
                    let _ = self.broadcast_message(confirm_msg).await;
                }
            }
        }
        Ok(())
    }
}

/// Errors specific to the Agent module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Identity-related error.
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),
    // Add other agent errors here
}

use crate::core::services::ServiceRegistry;
use std::sync::Arc;

/// A default implementation of an Agent for testing and examples.
pub struct DefaultAgent {
    agent: Arc<Agent>,
    // You might want to hold the ServiceRegistry here if needed for lifecycle management
    _service_registry: Option<Arc<ServiceRegistry>>,
}

impl DefaultAgent {
    /// Creates a new DefaultAgent with the given configuration.
    pub async fn new(config: AgentConfig) -> anyhow::Result<Self> {
        // Initialize identity with default depth 20 and initial root 0
        let identity = AgentIdentity::new(20, semaphore::Field::from(0)).await?;

        // Create default network config
        let network_config = NetworkConfig {
            listen_addr: "0.0.0.0:0".parse().unwrap(), // Bind to random port
            bootstrap_peers: vec![],
            max_peers: 50,
            protocol_config: crate::network::ProtocolConfig {},
            resource_limits: crate::network::ResourceLimits {
                max_bandwidth: 1024 * 1024,
                max_memory: 512 * 1024 * 1024,
                max_connections: 50,
            },
            security_config: crate::network::SecurityConfig {
                trusted_authorities: vec![],
                local_certificate: None,
            },
        };

        // TODO: Make this configurable properly, maybe pass storage path in config
        let storage_path = std::path::PathBuf::from(".p2p-ai-agents");
        let model_manager = Arc::new(ModelManager::new(&storage_path));

        let task_manager = TaskManager::default();
        let agent = Arc::new(Agent::new(
            identity,
            config,
            network_config,
            task_manager,
            model_manager,
        ));

        Ok(Self {
            agent,
            _service_registry: None,
        })
    }

    /// Creates a new DefaultAgent with a provided ServiceRegistry.
    pub async fn with_services(
        config: AgentConfig,
        registry: Arc<ServiceRegistry>,
    ) -> anyhow::Result<Self> {
        let identity = AgentIdentity::new(20, semaphore::Field::from(0)).await?;

        // Default network config (same as above)
        let network_config = NetworkConfig {
            listen_addr: "0.0.0.0:0".parse().unwrap(),
            bootstrap_peers: vec![],
            max_peers: 50,
            protocol_config: crate::network::ProtocolConfig {},
            resource_limits: crate::network::ResourceLimits {
                max_bandwidth: 1024 * 1024,
                max_memory: 512 * 1024 * 1024,
                max_connections: 50,
            },
            security_config: crate::network::SecurityConfig {
                trusted_authorities: vec![],
                local_certificate: None,
            },
        };

        // TODO: Make this configurable properly
        let storage_path = std::path::PathBuf::from(".p2p-ai-agents");
        let model_manager = Arc::new(ModelManager::new(&storage_path));

        let task_manager = TaskManager::default();
        let agent = Arc::new(Agent::new(
            identity,
            config,
            network_config,
            task_manager,
            model_manager,
        ));

        Ok(Self {
            agent,
            _service_registry: Some(registry),
        })
    }

    /// Delegate start to the inner Agent and spawn the task loop
    pub async fn start(&self) -> anyhow::Result<()> {
        self.agent.clone().start().await?;

        let agent_clone = self.agent.clone();
        let mut shutdown_rx = self.agent.shutdown_tx.subscribe();

        // Setup Network Manager & Message Loop
        {
            let mut nm = self.agent.network_manager.lock().await;

            // Construct agent version string with capabilities
            // e.g. "/p2p-ai-agents/1.0.0/TextProcessor,VectorComputer"
            let mut version = "/p2p-ai-agents/1.0.0".to_string();
            if !self.agent.config.capabilities.is_empty() {
                version.push('/');
                let caps: Vec<String> = self
                    .agent
                    .config
                    .capabilities
                    .iter()
                    .map(|c| c.to_string())
                    .collect();
                version.push_str(&caps.join(","));
            }
            nm.set_agent_version(version);

            // Channel for network messages -> agent
            let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
            nm.set_message_callback(tx);

            // Start the network manager
            if let Err(e) = nm.start().await {
                eprintln!("Failed to start network manager: {:?}", e);
            }

            // Spawn message handler loop
            let agent_msg_clone = agent_clone.clone();
            let mut msg_shutdown_rx = self.agent.shutdown_tx.subscribe();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = msg_shutdown_rx.recv() => break,
                        msg = rx.recv() => {
                            if let Some(bytes) = msg {
                                // Deserialize and handle
                                if let Ok(message) = serde_json::from_slice::<Message>(&bytes) {
                                     if let Err(e) = agent_msg_clone.handle_message(message).await {
                                         eprintln!("Error handling message: {:?}", e);
                                     }
                                } else {
                                     eprintln!("Failed to deserialize network message");
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
        }

        // Spawn background task processing loop
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        // Received shutdown signal
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {
                        // Poll for tasks
                        if let Err(e) = agent_clone.process_next_task().await {
                             eprintln!("Error processing task: {:?}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Delegate stop to the inner Agent
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.agent.stop().await
    }

    /// Delegate id to the inner Agent
    pub fn id(&self) -> AgentId {
        self.agent.id()
    }

    /// Get status (Mock implementation for example)
    pub async fn status(&self) -> anyhow::Result<AgentStatus> {
        Ok(AgentStatus { is_running: true })
    }

    /// Submit a task to the agent
    pub async fn submit_task(&self, task: Task) -> anyhow::Result<TaskId> {
        Ok(self.agent.submit_task(task).await)
    }

    /// Cancels a submitted task
    pub async fn cancel_task(&self, id: &TaskId) -> anyhow::Result<()> {
        self.agent.cancel_task(*id).await
    }

    /// Get the status of a submitted task
    pub async fn task_status(&self, id: &TaskId) -> anyhow::Result<TaskStatus> {
        self.agent.task_status(id).await
    }

    /// Broadcast a message to the network
    pub async fn broadcast_message(&self, message: Message) -> anyhow::Result<()> {
        self.agent.broadcast_message(message).await
    }

    /// Process a received message (Exposed for testing/networking)
    pub async fn handle_message(&self, message: Message) -> anyhow::Result<()> {
        self.agent.handle_message(message).await
    }

    /// Get the local PeerId string
    pub async fn local_peer_id(&self) -> String {
        self.agent
            .network_manager
            .lock()
            .await
            .local_peer_id()
            .map(|p| p.to_string())
            .unwrap_or_default()
    }

    /// Get the number of connected peers
    pub async fn connected_peers_count(&self) -> usize {
        self.agent
            .network_manager
            .lock()
            .await
            .get_connected_peers()
            .await
            .len()
    }

    /// Get the listening address(es) of the agent
    pub async fn listen_addresses(&self) -> Vec<String> {
        let addrs = self
            .agent
            .network_manager
            .lock()
            .await
            .get_listen_addresses()
            .await;
        addrs.iter().map(|a| a.to_string()).collect()
    }

    /// Dial another peer manually (and bootstrap Kademlia if peer ID is present)
    pub async fn dial(&self, addr: &str) -> anyhow::Result<()> {
        let multiaddr = crate::network::Multiaddr(addr.to_string());

        // Check if address has a peer ID
        if let Some(pos) = addr.rfind("/p2p/") {
            let peer_id = &addr[pos + 5..];
            self.agent
                .network_manager
                .lock()
                .await
                .bootstrap(peer_id, multiaddr.clone())
                .await?;
        }

        // Always perform standard dial
        self.agent
            .network_manager
            .lock()
            .await
            .dial(multiaddr)
            .await?;
        Ok(())
    }

    /// Expose internal agent for advanced testing (e.g. Identity management)
    pub fn internal_agent(&self) -> &Arc<Agent> {
        &self.agent
    }
}

/// Simple status struct for the example
#[derive(Debug)]
pub struct AgentStatus {
    /// Whether the agent is currently running.
    pub is_running: bool,
}

/// Resource limits placeholder for the example
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage.
    pub max_cpu: f32,
    /// Maximum memory usage in bytes.
    pub max_memory: u64,
    /// Maximum storage usage in bytes.
    pub max_storage: u64,
    /// Maximum bandwidth usage in bytes/sec.
    pub max_bandwidth: u64,
    /// Maximum number of concurrent connections.
    pub max_connections: u32,
}
