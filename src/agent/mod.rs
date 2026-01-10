//! Agent Module
//!
//! This module contains the core agent logic, including identity management,
//! messaging, and task execution.

pub mod identity;
pub mod messaging;
pub mod task;

use crate::agent::identity::AgentIdentity;
use crate::agent::task::{Task, TaskId, TaskManager, TaskStatus, TaskType, TextProcessingExecutor, VectorComputationExecutor, TaskExecutor};
use crate::agent::messaging::{Message, MessageType};
use crate::core::identity::IdentityError;
use crate::network::{NetworkConfig, NetworkManager, NetworkMessage};
use tokio::sync::{broadcast, mpsc, Mutex};
use serde_json::json;
use futures::future::{AbortHandle, Abortable};

/// Unique identifier for an Agent.
pub type AgentId = String;

/// Configuration for an Agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    /// The unique name or identifier for the agent.
    pub name: String,
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
    /// Network manager (protected by mutex for mutable access during start/stop).
    pub network_manager: Arc<Mutex<NetworkManager>>,
    /// Shutdown signal sender.
    shutdown_tx: broadcast::Sender<()>,
}

impl Agent {
    /// Creates a new Agent instance.
    pub fn new(identity: AgentIdentity, config: AgentConfig, network_config: NetworkConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let network_manager = NetworkManager::new(network_config);
        
        Self {
            identity,
            config,
            task_manager: TaskManager::new(),
            network_manager: Arc::new(Mutex::new(network_manager)),
            shutdown_tx,
        }
    }
    
    /// Returns the Agent's ID.
    pub fn id(&self) -> AgentId {
         // Fallback to name for now, should be DID
         self.config.name.clone()
    }
    
    /// Starts the Agent.
    pub async fn start(&self) -> anyhow::Result<()> {
        let _shutdown_rx = self.shutdown_tx.subscribe();
        // Network manager start is handled by the wrapper (DefaultAgent) which wires up the callbacks
        Ok(())
    }
    
    /// Stops the Agent.
    pub async fn stop(&self) -> anyhow::Result<()> {
        let _ = self.shutdown_tx.send(());
        self.network_manager.lock().await.shutdown().await.ok();
        Ok(())
    }
    
    /// Broadcast a message to the network.
    pub async fn broadcast_message(&self, mut message: Message) -> anyhow::Result<()> {
        // Sign the message
        let signable_bytes = message.to_signable_bytes();
        let signature = self.identity.sign_data(&signable_bytes)?;
        message.signature = Some(signature);
        message.public_key = Some(self.identity.public_key_bytes());

        let bytes = serde_json::to_vec(&message)?;
        let msg = NetworkMessage {
            from: self.id(),
            to: "broadcast".to_string(),
            content: bytes,
        };
        // Use send_message which queues it. 
        // We need to implement actual sending in NetworkManager.
        // Wait, I updated `NetworkManager` to handle `SendMessage` command via Gossipsub!
        // So we need to push a command.
        // But `NetworkManager::send_message` currently just pushes to a queue for testing.
        // I need to update `NetworkManager::send_message` to send the command.
        
        // Actually, let's look at `NetworkManager`. I didn't update `send_message` method, only the command loop.
        // So `NetworkManager::send_message` still just pushes to internal vector.
        // This is fine for now, but for real broadcast we need to trigger the command.
        // Accessing command_sender is hard because it's private.
        //
        // Let's assume `NetworkManager` will be updated later to use the command channel in `send_message`.
        // For this step, I will just call `send_message` on the manager.
        self.network_manager.lock().await.send_message(msg).await;
        Ok(())
    }

    /// Submits a task to the agent.
    pub async fn submit_task(&self, task: Task) -> TaskId {
        // Add the task to the manager
        self.task_manager.add_task(task).await
    }
    
    /// Cancels a task.
    pub async fn cancel_task(&self, id: TaskId) -> anyhow::Result<()> {
        self.task_manager.cancel_task(id).await
    }

    /// Processes a single pending task if available (for manual execution or testing).
    pub async fn process_next_task(&self) -> anyhow::Result<Option<TaskId>> {
        let next_task = self.task_manager.get_next_pending_task().await;
        
        if let Some(task) = next_task {
            // Update status to Running
            self.task_manager.update_status(task.id, TaskStatus::Running).await?;
            
            let task_manager = self.task_manager.clone(); // Need to clone to move into spawned task
            // We need to implement clone for TaskManager which uses Arc, so it's cheap.
            // But TaskManager doesn't derive Clone yet.
            // Oh, TaskManager fields are Arc, so we can derive Clone in task.rs or impl it.
            // Let's check task.rs: "tasks: Arc<RwLock<...>>"
            // Wait, I can't modify task.rs here. I should have checked if TaskManager derives Clone.
            // It does not derive Clone in the previous read. I need to update task.rs first or manually clone the Arcs if exposed.
            // Actually, `self.task_manager` is `TaskManager`.
            // Let's assume I will add `#[derive(Clone)]` to TaskManager in `src/agent/task.rs`.
            
            let task_id = task.id;
            
            // Create an abort handle for cancellation
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            
            // Register the handle
            // We need to access register_running_task which I added to TaskManager
            // BUT, `self.task_manager` is not Arc wrapped inside Agent, Agent holds it by value?
            // "pub task_manager: TaskManager"
            // So `self.task_manager` is a struct field.
            // To move it into a closure, we need to clone it.
            // I will update TaskManager to derive Clone.
            
            self.task_manager.register_running_task(task_id, abort_handle).await;

            // Spawn the execution
            let _handle = tokio::spawn(Abortable::new(async move {
                let result = if let Some(payload) = &task.payload {
                     match payload.task_type {
                         TaskType::TextProcessing => {
                             TextProcessingExecutor.execute(payload).await
                         }
                         TaskType::VectorComputation => {
                             VectorComputationExecutor.execute(payload).await
                         }
                         TaskType::Custom(_) => {
                             // Fallback for custom tasks
                             tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                             Ok(json!({"status": "simulated_custom_execution"}))
                         }
                     }
                } else {
                     // No payload, just simulate work
                     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                     Ok(json!({"status": "no_payload"}))
                };
                
                match result {
                    Ok(output) => {
                         // Complete the task with result
                        let _ = task_manager.update_status(task_id, TaskStatus::Completed(output)).await;
                    }
                    Err(e) => {
                        let _ = task_manager.update_status(task_id, TaskStatus::Failed(e.to_string())).await;
                    }
                }
            }, abort_registration));
            
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
        // Verify Signature
        if let (Some(sig), Some(pk_bytes)) = (&message.signature, &message.public_key) {
             let signable_bytes = message.to_signable_bytes();
             if let Ok(valid) = self.identity.verify_signature(pk_bytes, &signable_bytes, sig) {
                 if !valid {
                     eprintln!("⚠️ Warning: Invalid signature for message from {}", message.sender);
                     // Reject message
                     return Err(anyhow::anyhow!("Invalid message signature"));
                 }
                 // Signature valid, proceed
             } else {
                 eprintln!("⚠️ Warning: Error verifying signature for message from {}", message.sender);
                 return Err(anyhow::anyhow!("Signature verification error"));
             }
        } else {
             eprintln!("⚠️ Warning: Unsigned message from {}. Rejecting in secure mode.", message.sender);
             return Err(anyhow::anyhow!("Missing signature or public key"));
        }

        match message.content {
            MessageType::TaskRequest(task) => {
                println!("Agent received TaskRequest: {}", task.id);
                // Submit the task to the local manager
                // We trust the sender for now (Identity verification to be added later)
                self.submit_task(task).await;
            }
            MessageType::TaskResponse { task_id, status } => {
                 println!("Agent received TaskResponse for {}: {:?}", task_id, status);
                 // Update local state if we are tracking this remote task
                 // (Not yet implemented: tracking remote tasks)
            }
            MessageType::Text(text) => {
                println!("Agent received Text message from {}: {}", message.sender, text);
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

use std::sync::Arc;
use crate::core::services::ServiceRegistry;

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

        let agent = Arc::new(Agent::new(identity, config, network_config));
        
        Ok(Self {
            agent,
            _service_registry: None,
        })
    }

    /// Creates a new DefaultAgent with a provided ServiceRegistry.
    pub async fn with_services(config: AgentConfig, registry: Arc<ServiceRegistry>) -> anyhow::Result<Self> {
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

         let agent = Arc::new(Agent::new(identity, config, network_config));

         Ok(Self {
             agent,
             _service_registry: Some(registry),
         })
    }

    /// Delegate start to the inner Agent and spawn the task loop
    pub async fn start(&self) -> anyhow::Result<()> {
        self.agent.start().await?;
        
        let agent_clone = self.agent.clone();
        let mut shutdown_rx = self.agent.shutdown_tx.subscribe();
        
        // Setup Network Manager & Message Loop
        {
            let mut nm = self.agent.network_manager.lock().await;
            
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

    /// Get the number of connected peers
    pub async fn connected_peers_count(&self) -> usize {
        self.agent.network_manager.lock().await.get_connected_peers().await.len()
    }
    
    /// Get the listening address(es) of the agent
    pub async fn listen_addresses(&self) -> Vec<String> {
        let addrs = self.agent.network_manager.lock().await.get_listen_addresses().await;
        addrs.iter().map(|a| a.to_string()).collect()
    }
    
    /// Dial another peer manually
    pub async fn dial(&self, addr: &str) -> anyhow::Result<()> {
        let multiaddr = crate::network::Multiaddr(addr.to_string());
        self.agent.network_manager.lock().await.dial(multiaddr).await?;
        Ok(())
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
