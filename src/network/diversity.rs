//! Connection diversity enforcement to prevent subnet-based Sybil attacks.
//!
//! This module limits the percentage of connections from any single subnet
//! to ensure network resilience against coordinated attacks.

use std::collections::HashMap;
use std::net::IpAddr;
use thiserror::Error;

/// Maximum percentage of connections allowed from a single /24 subnet
pub const MAX_SUBNET_PERCENTAGE: f32 = 0.20; // 20%

#[derive(Debug, Error)]
pub enum DiversityError {
    #[error("Subnet limit exceeded: {subnet} ({current}/{max} connections)")]
    SubnetLimitExceeded {
        subnet: String,
        current: usize,
        max: usize,
    },
    #[error("No connections available")]
    NoConnections,
}

/// Connection diversity manager
#[derive(Debug, Clone)]
pub struct DiversityManager {
    /// Map of subnet prefix -> count of connections
    subnet_counts: HashMap<String, usize>,
    /// Total number of connections
    total_connections: usize,
}

impl Default for DiversityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DiversityManager {
    /// Create a new diversity manager.
    pub fn new() -> Self {
        Self {
            subnet_counts: HashMap::new(),
            total_connections: 0,
        }
    }

    /// Check if a new connection from the given IP would exceed diversity limits.
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address of the potential connection
    ///
    /// # Returns
    ///
    /// `Ok(())` if connection is allowed, `Err` if it would exceed limits
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::IpAddr;
    /// use p2p_ai_agents::network::diversity::DiversityManager;
    ///
    /// let mut manager = DiversityManager::new();
    /// let ip: IpAddr = "192.168.1.1".parse().unwrap();
    /// assert!(manager.can_connect(&ip).is_ok());
    /// ```
    pub fn can_connect(&self, ip: &IpAddr) -> Result<(), DiversityError> {
        if self.total_connections == 0 {
            return Ok(());
        }

        let subnet = get_subnet_prefix(ip);
        let current_count = self.subnet_counts.get(&subnet).copied().unwrap_or(0);
        let max_allowed = self.max_connections_per_subnet();

        if current_count >= max_allowed {
            return Err(DiversityError::SubnetLimitExceeded {
                subnet,
                current: current_count,
                max: max_allowed,
            });
        }

        Ok(())
    }

    /// Add a new connection from the given IP.
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address of the new connection
    ///
    /// # Errors
    ///
    /// Returns error if connection would exceed diversity limits
    pub fn add_connection(&mut self, ip: &IpAddr) -> Result<(), DiversityError> {
        self.can_connect(ip)?;

        let subnet = get_subnet_prefix(ip);
        *self.subnet_counts.entry(subnet).or_insert(0) += 1;
        self.total_connections += 1;

        Ok(())
    }

    /// Remove a connection from the given IP.
    ///
    /// # Arguments
    ///
    /// * `ip` - IP address of the connection to remove
    pub fn remove_connection(&mut self, ip: &IpAddr) {
        let subnet = get_subnet_prefix(ip);
        
        if let Some(count) = self.subnet_counts.get_mut(&subnet) {
            if *count > 0 {
                *count -= 1;
                self.total_connections = self.total_connections.saturating_sub(1);
                
                if *count == 0 {
                    self.subnet_counts.remove(&subnet);
                }
            }
        }
    }

    /// Get the current number of connections from a subnet.
    pub fn subnet_connection_count(&self, ip: &IpAddr) -> usize {
        let subnet = get_subnet_prefix(ip);
        self.subnet_counts.get(&subnet).copied().unwrap_or(0)
    }

    /// Get total number of connections.
    pub fn total_connections(&self) -> usize {
        self.total_connections
    }

    /// Get number of unique subnets.
    pub fn unique_subnets(&self) -> usize {
        self.subnet_counts.len()
    }

    /// Calculate maximum allowed connections per subnet based on total.
    fn max_connections_per_subnet(&self) -> usize {
        if self.total_connections == 0 {
            return 0;
        }
        ((self.total_connections as f32) * MAX_SUBNET_PERCENTAGE).ceil() as usize
    }

    /// Get subnet statistics.
    pub fn subnet_stats(&self) -> HashMap<String, usize> {
        self.subnet_counts.clone()
    }
}

/// Extract /24 subnet prefix from IP address.
fn get_subnet_prefix(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
        }
        IpAddr::V6(ipv6) => {
            let segments = ipv6.segments();
            format!("{:x}:{:x}:{:x}::/48", segments[0], segments[1], segments[2])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn test_get_subnet_prefix_ipv4() {
        assert_eq!(get_subnet_prefix(&ip("192.168.1.1")), "192.168.1.0/24");
        assert_eq!(get_subnet_prefix(&ip("192.168.1.254")), "192.168.1.0/24");
        assert_eq!(get_subnet_prefix(&ip("10.0.0.1")), "10.0.0.0/24");
    }

    #[test]
    fn test_get_subnet_prefix_ipv6() {
        let ipv6 = ip("2001:0db8:85a3::1");
        let prefix = get_subnet_prefix(&ipv6);
        assert!(prefix.starts_with("2001:db8:85a3::/48"));
    }

    #[test]
    fn test_new_manager() {
        let manager = DiversityManager::new();
        assert_eq!(manager.total_connections(), 0);
        assert_eq!(manager.unique_subnets(), 0);
    }

    #[test]
    fn test_add_connection() {
        let mut manager = DiversityManager::new();
        
        assert!(manager.add_connection(&ip("192.168.1.1")).is_ok());
        assert_eq!(manager.total_connections(), 1);
        assert_eq!(manager.subnet_connection_count(&ip("192.168.1.1")), 1);
    }

    #[test]
    fn test_remove_connection() {
        let mut manager = DiversityManager::new();
        let test_ip = ip("192.168.1.1");
        
        manager.add_connection(&test_ip).unwrap();
        assert_eq!(manager.total_connections(), 1);
        
        manager.remove_connection(&test_ip);
        assert_eq!(manager.total_connections(), 0);
        assert_eq!(manager.subnet_connection_count(&test_ip), 0);
    }

    #[test]
    fn test_multiple_connections_same_subnet() {
        let mut manager = DiversityManager::new();
        
        manager.add_connection(&ip("192.168.1.1")).unwrap();
        manager.add_connection(&ip("192.168.1.2")).unwrap();
        manager.add_connection(&ip("192.168.1.3")).unwrap();
        
        assert_eq!(manager.total_connections(), 3);
        assert_eq!(manager.subnet_connection_count(&ip("192.168.1.1")), 3);
        assert_eq!(manager.unique_subnets(), 1);
    }

    #[test]
    fn test_multiple_subnets() {
        let mut manager = DiversityManager::new();
        
        manager.add_connection(&ip("192.168.1.1")).unwrap();
        manager.add_connection(&ip("192.168.2.1")).unwrap();
        manager.add_connection(&ip("10.0.0.1")).unwrap();
        
        assert_eq!(manager.total_connections(), 3);
        assert_eq!(manager.unique_subnets(), 3);
        assert_eq!(manager.subnet_connection_count(&ip("192.168.1.1")), 1);
        assert_eq!(manager.subnet_connection_count(&ip("192.168.2.1")), 1);
        assert_eq!(manager.subnet_connection_count(&ip("10.0.0.1")), 1);
    }

    #[test]
    fn test_subnet_limit_enforcement() {
        let mut manager = DiversityManager::new();
        
        // Add 10 connections total to trigger 20% limit (max 2 per subnet)
        for i in 1..=8 {
            manager.add_connection(&ip(&format!("10.0.{}.1", i))).unwrap();
        }
        
        // First connection to 192.168.1.x should succeed
        assert!(manager.add_connection(&ip("192.168.1.1")).is_ok());
        
        // Second connection should succeed (2 out of 9 = 22%, but rounded up to 2)
        assert!(manager.add_connection(&ip("192.168.1.2")).is_ok());
        
        // Third connection should fail (would be 3 out of 10 = 30% > 20%)
        let result = manager.add_connection(&ip("192.168.1.3"));
        assert!(matches!(result, Err(DiversityError::SubnetLimitExceeded { .. })));
    }

    #[test]
    fn test_can_connect_first_connection() {
        let manager = DiversityManager::new();
        assert!(manager.can_connect(&ip("192.168.1.1")).is_ok());
    }

    #[test]
    fn test_can_connect_under_limit() {
        let mut manager = DiversityManager::new();
        
        for i in 1..=5 {
            manager.add_connection(&ip(&format!("10.0.0.{}", i))).unwrap();
        }
        
        // Should allow 1 more from same subnet (1/6 = 16.7% < 20%)
        assert!(manager.can_connect(&ip("10.0.0.100")).is_ok());
    }

    #[test]
    fn test_subnet_stats() {
        let mut manager = DiversityManager::new();
        
        manager.add_connection(&ip("192.168.1.1")).unwrap();
        manager.add_connection(&ip("192.168.1.2")).unwrap();
        manager.add_connection(&ip("10.0.0.1")).unwrap();
        
        let stats = manager.subnet_stats();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats.get("192.168.1.0/24"), Some(&2));
        assert_eq!(stats.get("10.0.0.0/24"), Some(&1));
    }

    #[test]
    fn test_remove_nonexistent_connection() {
        let mut manager = DiversityManager::new();
        
        // Should not panic or error
        manager.remove_connection(&ip("192.168.1.1"));
        assert_eq!(manager.total_connections(), 0);
    }

    #[test]
    fn test_remove_cleans_up_empty_subnets() {
        let mut manager = DiversityManager::new();
        let test_ip = ip("192.168.1.1");
        
        manager.add_connection(&test_ip).unwrap();
        assert_eq!(manager.unique_subnets(), 1);
        
        manager.remove_connection(&test_ip);
        assert_eq!(manager.unique_subnets(), 0);
    }

    #[test]
    fn test_max_connections_per_subnet_calculation() {
        let mut manager = DiversityManager::new();
        
        // With 10 connections, max per subnet should be 2 (20%)
        for i in 1..=10 {
            manager.add_connection(&ip(&format!("10.0.{}.1", i))).unwrap();
        }
        
        assert_eq!(manager.max_connections_per_subnet(), 2);
        
        // With 5 connections, max per subnet should be 1 (20% of 5 = 1)
        let mut manager2 = DiversityManager::new();
        for i in 1..=5 {
            manager2.add_connection(&ip(&format!("10.0.{}.1", i))).unwrap();
        }
        assert_eq!(manager2.max_connections_per_subnet(), 1);
    }

    #[test]
    fn test_ipv6_connections() {
        let mut manager = DiversityManager::new();
        
        let ipv6_1 = ip("2001:0db8:85a3::1");
        let ipv6_2 = ip("2001:0db8:85a3::2");
        
        manager.add_connection(&ipv6_1).unwrap();
        manager.add_connection(&ipv6_2).unwrap();
        
        assert_eq!(manager.total_connections(), 2);
        assert_eq!(manager.subnet_connection_count(&ipv6_1), 2);
    }
}
