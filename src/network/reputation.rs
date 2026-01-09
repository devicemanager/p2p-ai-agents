//! Reputation system for agents in the network.
//!
//! This module implements a reputation scoring system that limits task quotas
//! based on agent behavior and history.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Maximum reputation score
pub const MAX_REPUTATION: i32 = 1000;
/// Minimum reputation score
pub const MIN_REPUTATION: i32 = 0;
/// Starting reputation for new agents
pub const STARTING_REPUTATION: i32 = 100;

/// Reputation tier thresholds and quotas
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReputationTier {
    /// 0-249: Limited access
    Newcomer,
    /// 250-499: Basic access
    Established,
    /// 500-749: Enhanced access
    Trusted,
    /// 750-1000: Full access
    Elite,
}

impl ReputationTier {
    /// Get the tier for a given reputation score.
    pub fn from_score(score: i32) -> Self {
        match score {
            0..=249 => ReputationTier::Newcomer,
            250..=499 => ReputationTier::Established,
            500..=749 => ReputationTier::Trusted,
            _ => ReputationTier::Elite,
        }
    }

    /// Get the task quota for this tier (tasks per hour).
    pub fn task_quota(&self) -> u32 {
        match self {
            ReputationTier::Newcomer => 10,
            ReputationTier::Established => 50,
            ReputationTier::Trusted => 200,
            ReputationTier::Elite => 1000,
        }
    }

    /// Get the connection quota for this tier (max concurrent connections).
    pub fn connection_quota(&self) -> u32 {
        match self {
            ReputationTier::Newcomer => 5,
            ReputationTier::Established => 20,
            ReputationTier::Trusted => 50,
            ReputationTier::Elite => 100,
        }
    }
}

#[derive(Debug, Error)]
pub enum ReputationError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Reputation score out of bounds: {0}")]
    ScoreOutOfBounds(i32),
}

/// Reputation manager for tracking agent scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationManager {
    scores: HashMap<String, i32>,
}

impl Default for ReputationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReputationManager {
    /// Create a new reputation manager.
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
        }
    }

    /// Register a new agent with starting reputation.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Unique identifier for the agent
    ///
    /// # Examples
    ///
    /// ```
    /// use p2p_ai_agents::network::reputation::ReputationManager;
    ///
    /// let mut manager = ReputationManager::new();
    /// manager.register_agent("agent1".to_string());
    /// assert_eq!(manager.get_score("agent1").unwrap(), 100);
    /// ```
    pub fn register_agent(&mut self, agent_id: String) {
        self.scores.insert(agent_id, STARTING_REPUTATION);
    }

    /// Get the reputation score for an agent.
    pub fn get_score(&self, agent_id: &str) -> Result<i32, ReputationError> {
        self.scores
            .get(agent_id)
            .copied()
            .ok_or_else(|| ReputationError::AgentNotFound(agent_id.to_string()))
    }

    /// Get the reputation tier for an agent.
    pub fn get_tier(&self, agent_id: &str) -> Result<ReputationTier, ReputationError> {
        let score = self.get_score(agent_id)?;
        Ok(ReputationTier::from_score(score))
    }

    /// Increase reputation for successful task completion.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `amount` - Amount to increase (default: 10 for successful task)
    pub fn increase_reputation(
        &mut self,
        agent_id: &str,
        amount: i32,
    ) -> Result<i32, ReputationError> {
        let current = self.get_score(agent_id)?;
        let new_score = (current + amount).min(MAX_REPUTATION);
        self.scores.insert(agent_id.to_string(), new_score);
        Ok(new_score)
    }

    /// Decrease reputation for failed task or malicious behavior.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `amount` - Amount to decrease (default: 5 for failed task, 50 for malicious behavior)
    pub fn decrease_reputation(
        &mut self,
        agent_id: &str,
        amount: i32,
    ) -> Result<i32, ReputationError> {
        let current = self.get_score(agent_id)?;
        let new_score = (current - amount).max(MIN_REPUTATION);
        self.scores.insert(agent_id.to_string(), new_score);
        Ok(new_score)
    }

    /// Check if an agent can accept a new task based on their tier quota.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - Agent identifier
    /// * `current_tasks` - Number of tasks currently assigned to agent in the last hour
    pub fn can_accept_task(
        &self,
        agent_id: &str,
        current_tasks: u32,
    ) -> Result<bool, ReputationError> {
        let tier = self.get_tier(agent_id)?;
        Ok(current_tasks < tier.task_quota())
    }

    /// Get all agents and their scores.
    pub fn all_scores(&self) -> &HashMap<String, i32> {
        &self.scores
    }

    /// Get number of registered agents.
    pub fn agent_count(&self) -> usize {
        self.scores.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_tier_from_score() {
        assert_eq!(ReputationTier::from_score(0), ReputationTier::Newcomer);
        assert_eq!(ReputationTier::from_score(100), ReputationTier::Newcomer);
        assert_eq!(ReputationTier::from_score(249), ReputationTier::Newcomer);
        assert_eq!(ReputationTier::from_score(250), ReputationTier::Established);
        assert_eq!(ReputationTier::from_score(499), ReputationTier::Established);
        assert_eq!(ReputationTier::from_score(500), ReputationTier::Trusted);
        assert_eq!(ReputationTier::from_score(749), ReputationTier::Trusted);
        assert_eq!(ReputationTier::from_score(750), ReputationTier::Elite);
        assert_eq!(ReputationTier::from_score(1000), ReputationTier::Elite);
    }

    #[test]
    fn test_reputation_tier_quotas() {
        assert_eq!(ReputationTier::Newcomer.task_quota(), 10);
        assert_eq!(ReputationTier::Established.task_quota(), 50);
        assert_eq!(ReputationTier::Trusted.task_quota(), 200);
        assert_eq!(ReputationTier::Elite.task_quota(), 1000);

        assert_eq!(ReputationTier::Newcomer.connection_quota(), 5);
        assert_eq!(ReputationTier::Established.connection_quota(), 20);
        assert_eq!(ReputationTier::Trusted.connection_quota(), 50);
        assert_eq!(ReputationTier::Elite.connection_quota(), 100);
    }

    #[test]
    fn test_register_agent() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        assert_eq!(manager.get_score("agent1").unwrap(), STARTING_REPUTATION);
        assert_eq!(manager.agent_count(), 1);
    }

    #[test]
    fn test_get_score_nonexistent_agent() {
        let manager = ReputationManager::new();
        assert!(matches!(
            manager.get_score("nonexistent"),
            Err(ReputationError::AgentNotFound(_))
        ));
    }

    #[test]
    fn test_get_tier() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        assert_eq!(
            manager.get_tier("agent1").unwrap(),
            ReputationTier::Newcomer
        );
    }

    #[test]
    fn test_increase_reputation() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        let new_score = manager.increase_reputation("agent1", 50).unwrap();
        assert_eq!(new_score, 150);
        assert_eq!(manager.get_score("agent1").unwrap(), 150);
    }

    #[test]
    fn test_increase_reputation_max_cap() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        manager.increase_reputation("agent1", 2000).unwrap();
        assert_eq!(manager.get_score("agent1").unwrap(), MAX_REPUTATION);
    }

    #[test]
    fn test_decrease_reputation() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        let new_score = manager.decrease_reputation("agent1", 30).unwrap();
        assert_eq!(new_score, 70);
        assert_eq!(manager.get_score("agent1").unwrap(), 70);
    }

    #[test]
    fn test_decrease_reputation_min_cap() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        manager.decrease_reputation("agent1", 2000).unwrap();
        assert_eq!(manager.get_score("agent1").unwrap(), MIN_REPUTATION);
    }

    #[test]
    fn test_can_accept_task() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        
        // Newcomer tier has quota of 10
        assert!(manager.can_accept_task("agent1", 5).unwrap());
        assert!(manager.can_accept_task("agent1", 9).unwrap());
        assert!(!manager.can_accept_task("agent1", 10).unwrap());
        assert!(!manager.can_accept_task("agent1", 15).unwrap());
    }

    #[test]
    fn test_can_accept_task_different_tiers() {
        let mut manager = ReputationManager::new();
        manager.register_agent("newcomer".to_string());
        manager.register_agent("established".to_string());
        manager.register_agent("trusted".to_string());
        manager.register_agent("elite".to_string());
        
        manager.increase_reputation("established", 150).unwrap(); // 250
        manager.increase_reputation("trusted", 400).unwrap(); // 500
        manager.increase_reputation("elite", 650).unwrap(); // 750
        
        assert!(!manager.can_accept_task("newcomer", 10).unwrap());
        assert!(manager.can_accept_task("established", 10).unwrap());
        assert!(!manager.can_accept_task("established", 50).unwrap());
        assert!(manager.can_accept_task("trusted", 50).unwrap());
        assert!(!manager.can_accept_task("trusted", 200).unwrap());
        assert!(manager.can_accept_task("elite", 500).unwrap());
    }

    #[test]
    fn test_multiple_agents() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        manager.register_agent("agent2".to_string());
        manager.register_agent("agent3".to_string());
        
        manager.increase_reputation("agent1", 50).unwrap();
        manager.decrease_reputation("agent2", 20).unwrap();
        
        assert_eq!(manager.get_score("agent1").unwrap(), 150);
        assert_eq!(manager.get_score("agent2").unwrap(), 80);
        assert_eq!(manager.get_score("agent3").unwrap(), 100);
        assert_eq!(manager.agent_count(), 3);
    }

    #[test]
    fn test_all_scores() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        manager.register_agent("agent2".to_string());
        
        let scores = manager.all_scores();
        assert_eq!(scores.len(), 2);
        assert_eq!(scores.get("agent1"), Some(&100));
        assert_eq!(scores.get("agent2"), Some(&100));
    }

    #[test]
    fn test_serialization() {
        let mut manager = ReputationManager::new();
        manager.register_agent("agent1".to_string());
        manager.increase_reputation("agent1", 50).unwrap();
        
        let json = serde_json::to_string(&manager).unwrap();
        let deserialized: ReputationManager = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.get_score("agent1").unwrap(), 150);
    }
}
