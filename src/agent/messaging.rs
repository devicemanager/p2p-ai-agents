//! Messaging module for Agent communication.

use crate::agent::task::{Task, TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a message.
pub type MessageId = Uuid;

/// Types of messages in the Agent Protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request to execute a task.
    TaskRequest(Task),
    /// Response with task status/result.
    TaskResponse {
        /// ID of the task.
        task_id: TaskId,
        /// Current status/result of the task.
        status: TaskStatus,
    },
    /// Generic text message (legacy/chat).
    Text(String),
}

/// A message exchanged between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique ID of the message.
    pub id: MessageId,
    /// ID of the sender agent.
    pub sender: String,
    /// ID of the recipient agent (or "broadcast").
    pub recipient: String,
    /// The content of the message.
    pub content: MessageType,
    /// Timestamp of creation.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Message {
    /// Creates a new generic text message.
    pub fn new_text(sender: impl Into<String>, recipient: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::Text(text.into()),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Creates a task request message.
    pub fn new_task_request(sender: impl Into<String>, recipient: impl Into<String>, task: Task) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::TaskRequest(task),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Creates a task response message.
    pub fn new_task_response(sender: impl Into<String>, recipient: impl Into<String>, task_id: TaskId, status: TaskStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::TaskResponse { task_id, status },
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Manages messaging operations for an agent.
pub struct Messenger;

impl Messenger {
    /// Creates a new Messenger.
    pub fn new() -> Self {
        Self
    }

    /// Sends a message.
    /// 
    /// In a real implementation, this would hand off to the NetworkManager.
    pub async fn send(&self, message: &Message) -> anyhow::Result<()> {
        // Placeholder: Log the message sending
        tracing::info!(
            "Messenger: Sending message {} from {} to {} [Type: {:?}]",
            message.id,
            message.sender,
            message.recipient,
            message.content
        );
        Ok(())
    }

    /// Simulates receiving a message.
    pub async fn receive(&self) -> Option<Message> {
        // Placeholder: currently does not actually receive anything
        None
    }
}
