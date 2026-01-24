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
    TaskRequest(Box<Task>),
    /// Response with task status/result.
    TaskResponse {
        /// ID of the task.
        task_id: TaskId,
        /// Current status/result of the task.
        status: TaskStatus,
    },
    /// Generic text message (legacy/chat).
    Text(String),
    /// Announcement of agent capabilities.
    CapabilityAnnouncement {
        /// List of supported task types.
        capabilities: Vec<crate::agent::task::TaskType>,
    },
    /// Request to cancel a task.
    TaskCancellation {
        /// ID of the task to cancel.
        task_id: TaskId,
    },
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
    /// Digital signature of the message content and metadata.
    pub signature: Option<Vec<u8>>,
    /// Public key of the sender (protobuf encoded) to verify the signature.
    pub public_key: Option<Vec<u8>>,
}

impl Message {
    /// Creates a new generic text message.
    pub fn new_text(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::Text(text.into()),
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    /// Creates a task request message.
    pub fn new_task_request(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        task: Task,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::TaskRequest(Box::new(task)),
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    /// Creates a task response message.
    pub fn new_task_response(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        task_id: TaskId,
        status: TaskStatus,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::TaskResponse { task_id, status },
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    /// Creates a capability announcement message.
    pub fn new_capability_announcement(
        sender: impl Into<String>,
        capabilities: Vec<crate::agent::task::TaskType>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: "broadcast".to_string(),
            content: MessageType::CapabilityAnnouncement { capabilities },
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    /// Creates a task cancellation message.
    pub fn new_task_cancellation(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        task_id: TaskId,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            sender: sender.into(),
            recipient: recipient.into(),
            content: MessageType::TaskCancellation { task_id },
            timestamp: chrono::Utc::now(),
            signature: None,
            public_key: None,
        }
    }

    /// Serializes the core message data for signing.
    /// Excludes the signature field itself.
    pub fn to_signable_bytes(&self) -> Vec<u8> {
        // We structure the data to ensure consistent serialization for signing
        // Format: sender + recipient + content_bytes + timestamp_millis
        let content_bytes = serde_json::to_vec(&self.content).unwrap_or_default();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.sender.as_bytes());
        bytes.extend_from_slice(self.recipient.as_bytes());
        bytes.extend_from_slice(&content_bytes);
        bytes.extend_from_slice(&self.timestamp.timestamp_millis().to_be_bytes());
        bytes
    }
}

/// Manages messaging operations for an agent.
pub struct Messenger;

impl Default for Messenger {
    fn default() -> Self {
        Self
    }
}

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
