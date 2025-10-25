//! Event-driven architecture for the P2P AI Agents system
//!
//! This module provides event handling capabilities for decoupled
//! communication between system components.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Type alias for event handler storage to reduce complexity
/// Wrapper for type-erased event handlers
#[async_trait]
trait ErasedEventHandler: Send + Sync {
    async fn handle_erased(&self, event: &(dyn Any + Send + Sync)) -> EventResult;
    fn name(&self) -> &'static str;
}

/// Wrapper to convert a concrete event handler to a type-erased one
struct EventHandlerWrapper<E: Event, H: EventHandler<E>> {
    handler: H,
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E: Event + 'static, H: EventHandler<E>> ErasedEventHandler for EventHandlerWrapper<E, H> {
    async fn handle_erased(&self, event: &(dyn Any + Send + Sync)) -> EventResult {
        if let Some(typed_event) = event.downcast_ref::<E>() {
            self.handler.handle(typed_event).await
        } else {
            EventResult::Error("Event type mismatch".to_string())
        }
    }

    fn name(&self) -> &'static str {
        self.handler.name()
    }
}

/// Type alias for event handler map
type EventHandlerMap = Arc<RwLock<HashMap<TypeId, Vec<Arc<dyn ErasedEventHandler>>>>>;

/// Event identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub Uuid);

impl EventId {
    /// Create a new event identifier
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EventId {
    fn default() -> Self {
        Self::new()
    }
}

/// Event priority for ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Low priority events
    Low = 0,
    /// Normal priority events
    Normal = 1,
    /// High priority events
    High = 2,
    /// Critical priority events
    Critical = 3,
}

/// Base event trait
pub trait Event: Send + Sync {
    /// Get the event type identifier
    fn event_type(&self) -> &'static str;

    /// Get the event priority
    fn priority(&self) -> EventPriority {
        EventPriority::Normal
    }

    /// Get the event ID
    fn id(&self) -> EventId;

    /// Get the event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;

    /// Get the event source
    fn source(&self) -> Option<String>;

    /// Clone the event
    fn clone_event(&self) -> Box<dyn Event>;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// Handle the event
    async fn handle(&self, event: &E) -> EventResult;

    /// Get the handler name for debugging
    fn name(&self) -> &'static str;

    /// Check if this handler can handle the event
    fn can_handle(&self, _event: &E) -> bool {
        true
    }
}

/// Event result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventResult {
    /// Event handled successfully
    Success,
    /// Event handled with a warning
    Warning(String),
    /// Event handling failed
    Error(String),
    /// Event should be retried
    Retry,
    /// Event should be ignored
    Ignore,
}

/// Event bus for managing event publishing and subscription
#[derive(Clone)]
pub struct EventBus {
    handlers: EventHandlerMap,
}

/// Error types for event operations
#[derive(Debug, Error)]
pub enum EventError {
    /// Event handler not found
    #[error("Event handler not found for event type: {0}")]
    HandlerNotFound(String),

    /// Event publishing failed
    #[error("Event publishing failed: {0}")]
    PublishingFailed(String),

    /// Event subscription failed
    #[error("Event subscription failed: {0}")]
    SubscriptionFailed(String),

    /// Event handling failed
    #[error("Event handling failed: {0}")]
    HandlingFailed(String),
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to events of a specific type
    pub async fn subscribe<E, H>(&self, handler: H) -> Result<(), EventError>
    where
        E: Event + 'static,
        H: EventHandler<E> + 'static,
    {
        let type_id = TypeId::of::<E>();
        let mut handlers = self.handlers.write().await;

        let handler_entry = handlers.entry(type_id).or_insert_with(Vec::new);
        let wrapped_handler = EventHandlerWrapper {
            handler,
            _phantom: std::marker::PhantomData,
        };
        handler_entry.push(Arc::new(wrapped_handler));

        Ok(())
    }

    /// Publish an event
    pub async fn publish<E>(&self, event: E) -> Result<(), EventError>
    where
        E: Event + 'static,
    {
        // Handle with registered handlers
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().await;

        if let Some(handler_list) = handlers.get(&type_id) {
            for handler in handler_list {
                let result = handler
                    .handle_erased(&event as &(dyn Any + Send + Sync))
                    .await;
                match result {
                    EventResult::Error(err) => {
                        tracing::error!("Event handler {} failed: {}", handler.name(), err);
                    }
                    EventResult::Warning(warn) => {
                        tracing::warn!("Event handler {} warning: {}", handler.name(), warn);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Get the number of handlers for an event type
    pub async fn handler_count<E>(&self) -> usize
    where
        E: Event + 'static,
    {
        let type_id = TypeId::of::<E>();
        let handlers = self.handlers.read().await;
        handlers.get(&type_id).map_or(0, |list| list.len())
    }

    /// Clear all handlers for an event type
    pub async fn clear_handlers<E>(&self)
    where
        E: Event + 'static,
    {
        let type_id = TypeId::of::<E>();
        let mut handlers = self.handlers.write().await;
        handlers.remove(&type_id);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for easier event creation
#[macro_export]
macro_rules! define_event {
    ($name:ident, $payload:ty) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        /// Event struct generated by define_event macro
        pub struct $name {
            /// Event identifier
            pub id: EventId,
            /// Event timestamp
            pub timestamp: chrono::DateTime<chrono::Utc>,
            /// Event source
            pub source: Option<String>,
            /// Event payload
            pub payload: $payload,
        }

        impl Event for $name {
            fn event_type(&self) -> &'static str {
                stringify!($name)
            }

            fn id(&self) -> EventId {
                self.id.clone()
            }

            fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
                self.timestamp
            }

            fn source(&self) -> Option<String> {
                self.source.clone()
            }

            fn clone_event(&self) -> Box<dyn Event> {
                Box::new(self.clone())
            }
        }

        impl $name {
            /// Create a new event
            pub fn new(payload: $payload, source: Option<String>) -> Self {
                Self {
                    id: EventId::new(),
                    timestamp: chrono::Utc::now(),
                    source,
                    payload,
                }
            }
        }
    };
}

// Define some common events
define_event!(AgentStarted, String);
define_event!(AgentStopped, String);
define_event!(TaskCompleted, String);
define_event!(TaskFailed, String);
define_event!(PeerConnected, String);
define_event!(PeerDisconnected, String);
define_event!(ResourceLimitExceeded, String);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct TestEventHandler {
        #[allow(dead_code)]
        name: String,
        handled_events: Arc<RwLock<Vec<EventId>>>,
    }

    #[async_trait]
    impl EventHandler<AgentStarted> for TestEventHandler {
        async fn handle(&self, event: &AgentStarted) -> EventResult {
            let mut events = self.handled_events.write().await;
            events.push(event.id().clone());
            EventResult::Success
        }

        fn name(&self) -> &'static str {
            "TestEventHandler"
        }
    }

    #[tokio::test]
    async fn test_event_bus_subscription_and_publishing() {
        let event_bus = EventBus::new();
        let handled_events = Arc::new(RwLock::new(Vec::new()));

        let handler = TestEventHandler {
            name: "test".to_string(),
            handled_events: handled_events.clone(),
        };

        // Subscribe to events
        event_bus
            .subscribe::<AgentStarted, _>(handler)
            .await
            .unwrap();

        // Publish an event
        let event = AgentStarted::new("agent-1".to_string(), Some("test-source".to_string()));
        event_bus.publish(event.clone()).await.unwrap();

        // Wait a bit for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Check if event was handled
        let events = handled_events.read().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event.id());
    }

    #[tokio::test]
    async fn test_event_bus_handler_count() {
        let event_bus = EventBus::new();

        let handler1 = TestEventHandler {
            name: "handler1".to_string(),
            handled_events: Arc::new(RwLock::new(Vec::new())),
        };

        let handler2 = TestEventHandler {
            name: "handler2".to_string(),
            handled_events: Arc::new(RwLock::new(Vec::new())),
        };

        // Subscribe two handlers
        event_bus
            .subscribe::<AgentStarted, _>(handler1)
            .await
            .unwrap();
        event_bus
            .subscribe::<AgentStarted, _>(handler2)
            .await
            .unwrap();

        // Check handler count
        assert_eq!(event_bus.handler_count::<AgentStarted>().await, 2);
    }
}
