# Access Control System

This document describes the access control system implemented in the P2P AI Agents core architecture.

## Overview

The access control system provides a comprehensive security framework with:

- **Authentication**: Identity verification and token management
- **Authorization**: Permission checking and access control
- **Role-Based Access Control (RBAC)**: Role-based permission assignment
- **Policy Management**: Flexible security policies
- **Principal Management**: User/agent identity management

## Core Concepts

### Principal

A principal represents a user or agent in the system:

```rust
use p2p_ai_agents::core::{Principal, PrincipalId, RoleId};

let principal = Principal {
    id: PrincipalId::new("agent-001".to_string()),
    name: "AI Processing Agent".to_string(),
    roles: HashSet::from([RoleId::new("worker".to_string())]),
    attributes: HashMap::new(),
};
```

### Roles

Roles define sets of permissions that can be assigned to principals:

```rust
use p2p_ai_agents::core::{Role, RoleId, Resource, Permission};

let mut permissions = HashMap::new();
let mut task_permissions = HashSet::new();
task_permissions.insert(Permission::new("execute".to_string()));
permissions.insert(Resource::new("tasks".to_string()), task_permissions);

let role = Role {
    id: RoleId::new("worker".to_string()),
    name: "Task Worker".to_string(),
    description: "Can execute tasks".to_string(),
    permissions,
};
```

### Permissions

Permissions define specific actions that can be performed on resources:

```rust
// Common permissions
let read = Permission::new("read".to_string());
let write = Permission::new("write".to_string());
let execute = Permission::new("execute".to_string());
let delete = Permission::new("delete".to_string());
```

### Resources

Resources represent protected system components:

```rust
// Common resources
let tasks = Resource::new("tasks".to_string());
let storage = Resource::new("storage".to_string());
let network = Resource::new("network".to_string());
let agents = Resource::new("agents".to_string());
```

## Authentication

### Using the Authenticator

```rust
use p2p_ai_agents::core::{Authenticator, AuthResult, DefaultAuthenticator, PrincipalId};

let authenticator = DefaultAuthenticator;
let principal_id = PrincipalId::new("user-123".to_string());
let credentials = HashMap::from([
    ("password".to_string(), "secret".to_string())
]);

match authenticator.authenticate(&principal_id, &credentials).await {
    AuthResult::Success(token) => {
        println!("Authenticated: {}", token.principal_id.as_str());
    }
    AuthResult::Failed(reason) => {
        println!("Authentication failed: {}", reason);
    }
    AuthResult::Expired => {
        println!("Token expired");
    }
}
```

### Token Validation

```rust
let validation_result = authenticator.validate_token(&token).await;
match validation_result {
    AuthResult::Success(valid_token) => {
        // Token is valid, proceed
    }
    AuthResult::Expired => {
        // Token expired, re-authenticate
    }
    AuthResult::Failed(reason) => {
        // Token invalid
    }
}
```

## Authorization

### Using the Authorizer

```rust
use p2p_ai_agents::core::{Authorizer, AuthzResult, DefaultAuthorizer, Principal, Resource, Permission};

let authorizer = DefaultAuthorizer::new();

// Check permission
let result = authorizer.check_permission(
    &principal,
    &Resource::new("tasks".to_string()),
    &Permission::new("execute".to_string())
).await;

match result {
    AuthzResult::Allow => {
        // Access granted
        execute_task().await;
    }
    AuthzResult::Deny(reason) => {
        // Access denied
        println!("Access denied: {}", reason);
    }
}
```

### Checking Multiple Permissions

```rust
// Get all permissions for a resource
let permissions = authorizer.get_permissions(
    &principal,
    &Resource::new("storage".to_string())
).await;

if permissions.contains(&Permission::new("write".to_string())) {
    // Can write to storage
}

// Check role membership
let has_admin_role = authorizer.has_role(
    &principal,
    &RoleId::new("admin".to_string())
).await;
```

## Access Control Manager

The `AccessControlManager` combines authentication and authorization:

```rust
use p2p_ai_agents::core::{AccessControlManager, DefaultAuthenticator, DefaultAuthorizer};

let authenticator = Arc::new(DefaultAuthenticator);
let authorizer = Arc::new(DefaultAuthorizer::new());
let acm = AccessControlManager::new(authenticator, authorizer);

// Add roles and principals
acm.add_role(worker_role).await;
acm.add_principal(principal).await;

// Authenticate and authorize in one call
let allowed = acm.is_allowed(
    &PrincipalId::new("user-123".to_string()),
    &Resource::new("tasks".to_string()),
    &Permission::new("execute".to_string())
).await?;

if allowed {
    // Proceed with operation
}
```

## Policies

Policies define security rules at a higher level:

```rust
use p2p_ai_agents::core::{Policy, PolicyEffect, RoleId, Resource, Permission};

let policy = Policy {
    id: "worker-policy".to_string(),
    name: "Worker Access Policy".to_string(),
    roles: HashSet::from([RoleId::new("worker".to_string())]),
    permissions: {
        let mut perms = HashMap::new();
        let mut task_perms = HashSet::new();
        task_perms.insert(Permission::new("execute".to_string()));
        perms.insert(Resource::new("tasks".to_string()), task_perms);
        perms
    },
    effect: PolicyEffect::Allow,
};

acm.add_policy(policy).await;
```

## Security Best Practices

### 1. Principle of Least Privilege

Assign only the minimum permissions required:

```rust
// Good: Specific permissions
let read_only_role = Role {
    id: RoleId::new("reader".to_string()),
    name: "Read Only".to_string(),
    description: "Can only read resources".to_string(),
    permissions: HashMap::from([
        (Resource::new("tasks".to_string()),
         HashSet::from([Permission::new("read".to_string())]))
    ]),
};

// Avoid: Overly broad permissions
```

### 2. Role-Based Assignment

Use roles to manage permissions consistently:

```rust
// Define roles once
let admin_role = Role { /* admin permissions */ };
let user_role = Role { /* user permissions */ };

// Assign roles to principals
let admin_principal = Principal {
    roles: HashSet::from([RoleId::new("admin".to_string())]),
    // ...
};
```

### 3. Token Expiration

Always set reasonable token expiration times:

```rust
let token = AuthToken {
    principal_id: principal_id,
    expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)), // 1 hour
    claims: HashMap::new(),
};
```

### 4. Secure Credential Storage

Never store credentials in code:

```rust
// Good: Use environment variables or secure vaults
let password = std::env::var("AGENT_PASSWORD")
    .expect("AGENT_PASSWORD must be set");

// Avoid: Hardcoded credentials
// let password = "secret123"; // NEVER DO THIS
```

## Integration Examples

### Agent Task Execution

```rust
use p2p_ai_agents::core::AccessControlManager;

pub struct TaskExecutor {
    acm: AccessControlManager,
}

impl TaskExecutor {
    pub async fn execute_task(
        &self,
        principal_id: &PrincipalId,
        task: Task
    ) -> Result<TaskResult, String> {
        // Check if principal can execute tasks
        let allowed = self.acm.is_allowed(
            principal_id,
            &Resource::new("tasks".to_string()),
            &Permission::new("execute".to_string())
        ).await?;

        if !allowed {
            return Err("Insufficient permissions to execute tasks".to_string());
        }

        // Execute task...
        Ok(execute_task_impl(task).await)
    }
}
```

### Storage Access Control

```rust
pub struct SecureStorage {
    storage: Arc<dyn Storage>,
    acm: AccessControlManager,
}

impl SecureStorage {
    pub async fn secure_put(
        &self,
        principal_id: &PrincipalId,
        key: &str,
        value: Vec<u8>
    ) -> Result<(), String> {
        let allowed = self.acm.is_allowed(
            principal_id,
            &Resource::new("storage".to_string()),
            &Permission::new("write".to_string())
        ).await?;

        if !allowed {
            return Err("Insufficient permissions for storage write".to_string());
        }

        self.storage.put(key, value).await
            .map_err(|e| format!("Storage error: {}", e))
    }
}
```

## Testing Access Control

### Unit Testing Permissions

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use p2p_ai_agents::core::{DefaultAuthorizer, Principal, RoleId};

    #[tokio::test]
    async fn test_permission_check() {
        let authorizer = DefaultAuthorizer::new();

        // Set up role with permissions
        let mut permissions = HashMap::new();
        let mut task_perms = HashSet::new();
        task_perms.insert(Permission::new("read".to_string()));
        permissions.insert(Resource::new("tasks".to_string()), task_perms);

        authorizer.update_role_permissions(
            RoleId::new("user".to_string()),
            permissions
        ).await;

        // Create principal
        let principal = Principal {
            id: PrincipalId::new("test-user".to_string()),
            name: "Test User".to_string(),
            roles: HashSet::from([RoleId::new("user".to_string())]),
            attributes: HashMap::new(),
        };

        // Test allowed permission
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("tasks".to_string()),
            &Permission::new("read".to_string())
        ).await;
        assert_eq!(result, AuthzResult::Allow);

        // Test denied permission
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("tasks".to_string()),
            &Permission::new("write".to_string())
        ).await;
        assert!(matches!(result, AuthzResult::Deny(_)));
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_access_control_flow() {
        // Set up complete access control system
        let authenticator = Arc::new(DefaultAuthenticator);
        let authorizer = Arc::new(DefaultAuthorizer::new());
        let acm = AccessControlManager::new(authenticator, authorizer);

        // Create and add role
        let role = Role {
            id: RoleId::new("test-role".to_string()),
            name: "Test Role".to_string(),
            description: "Role for testing".to_string(),
            permissions: HashMap::new(),
        };
        acm.add_role(role).await;

        // Create and add principal
        let principal = Principal {
            id: PrincipalId::new("test-principal".to_string()),
            name: "Test Principal".to_string(),
            roles: HashSet::from([RoleId::new("test-role".to_string())]),
            attributes: HashMap::new(),
        };
        acm.add_principal(principal).await;

        // Test authentication
        let auth_result = acm.authenticate(
            &PrincipalId::new("test-principal".to_string()),
            &HashMap::new()
        ).await;
        assert!(matches!(auth_result, AuthResult::Success(_)));

        // Test authorization (would need permissions set up)
        // ...
    }
}
```

## Configuration

### Environment Variables

```bash
# Authentication settings
ACCESS_CONTROL_TOKEN_EXPIRY_HOURS=24
ACCESS_CONTROL_MAX_FAILED_ATTEMPTS=5

# Default roles
ACCESS_CONTROL_DEFAULT_ROLES=guest,user

# Security policies
ACCESS_CONTROL_ENABLE_AUDIT_LOG=true
```

### Runtime Configuration

```rust
use p2p_ai_agents::core::ConfigManager;

// Load access control configuration
let config = ConfigManager::new();
config.load_from_env("ACCESS_CONTROL_").await?;

// Configure access control manager
let token_expiry = config.get("token_expiry_hours")
    .await?
    .as_integer()
    .unwrap_or(24);

let acm = AccessControlManager::new(authenticator, authorizer);
// Configure ACM with loaded settings...
```

## Monitoring and Auditing

### Audit Logging

```rust
pub struct AuditLogger {
    acm: AccessControlManager,
}

impl AuditLogger {
    pub async fn log_access_attempt(
        &self,
        principal_id: &PrincipalId,
        resource: &Resource,
        permission: &Permission,
        result: &AuthzResult,
        context: HashMap<String, String>
    ) {
        let timestamp = chrono::Utc::now();
        let log_entry = serde_json::json!({
            "timestamp": timestamp,
            "principal_id": principal_id.as_str(),
            "resource": resource.as_str(),
            "permission": permission.as_str(),
            "result": match result {
                AuthzResult::Allow => "allowed",
                AuthzResult::Deny(_) => "denied",
            },
            "context": context
        });

        // Log to audit system
        println!("AUDIT: {}", log_entry);
    }
}
```

### Metrics Collection

```rust
pub struct AccessControlMetrics {
    auth_attempts: AtomicU64,
    auth_successes: AtomicU64,
    auth_failures: AtomicU64,
    authz_checks: AtomicU64,
    authz_allowed: AtomicU64,
    authz_denied: AtomicU64,
}

impl AccessControlMetrics {
    pub fn record_auth_attempt(&self) {
        self.auth_attempts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_auth_success(&self) {
        self.auth_successes.fetch_add(1, Ordering::Relaxed);
    }

    // ... other metrics
}
```

## Troubleshooting

### Common Issues

#### Permission Denied Errors

**Symptoms**: Operations fail with "Access denied" messages

**Solutions**:
- Verify principal has required roles
- Check role permissions are correctly assigned
- Ensure resource and permission names match exactly

#### Authentication Failures

**Symptoms**: Authentication consistently fails

**Solutions**:
- Verify credentials are correct
- Check token expiration settings
- Ensure authenticator is properly configured

#### Role Assignment Issues

**Symptoms**: Users can't access expected resources

**Solutions**:
- Confirm roles are assigned to principals
- Verify role permissions include required access
- Check for typos in role, resource, or permission names

### Debug Mode

Enable debug logging for access control decisions:

```rust
use tracing::{info, warn};

#[tracing::instrument(skip(principal, resource, permission))]
async fn check_permission_debug(
    principal: &Principal,
    resource: &Resource,
    permission: &Permission,
) -> AuthzResult {
    info!("Checking permission for principal: {}", principal.id.as_str());

    let result = authorizer.check_permission(principal, resource, permission).await;

    match &result {
        AuthzResult::Allow => info!("Permission granted"),
        AuthzResult::Deny(reason) => warn!("Permission denied: {}", reason),
    }

    result
}
```

## Performance Considerations

### Caching

Implement permission caching to reduce authorization overhead:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct PermissionCache {
    cache: RwLock<HashMap<(PrincipalId, Resource, Permission), (AuthzResult, chrono::DateTime<chrono::Utc>)>>,
    ttl: chrono::Duration,
}

impl PermissionCache {
    pub async fn get_cached_permission(
        &self,
        principal_id: &PrincipalId,
        resource: &Resource,
        permission: &Permission,
    ) -> Option<AuthzResult> {
        let cache = self.cache.read().await;
        if let Some((result, timestamp)) = cache.get(&(principal_id.clone(), resource.clone(), permission.clone())) {
            if chrono::Utc::now() - *timestamp < self.ttl {
                return Some(result.clone());
            }
        }
        None
    }

    pub async fn cache_permission(
        &self,
        principal_id: &PrincipalId,
        resource: &Resource,
        permission: &Permission,
        result: AuthzResult,
    ) {
        let mut cache = self.cache.write().await;
        cache.insert(
            (principal_id.clone(), resource.clone(), permission.clone()),
            (result, chrono::Utc::now())
        );
    }
}
```

### Batch Authorization

For multiple permission checks, batch them to reduce overhead:

```rust
pub async fn check_multiple_permissions(
    &self,
    principal: &Principal,
    checks: Vec<(Resource, Permission)>,
) -> HashMap<(Resource, Permission), AuthzResult> {
    let mut results = HashMap::new();

    // Process in parallel for better performance
    let futures = checks.into_iter().map(|(resource, permission)| {
        let principal = principal.clone();
        async move {
            let result = self.authorizer.check_permission(&principal, &resource, &permission).await;
            ((resource, permission), result)
        }
    });

    let results_vec = futures_util::future::join_all(futures).await;

    for (key, result) in results_vec {
        results.insert(key, result);
    }

    results
}
```

## Future Enhancements

### Advanced Features

- **Attribute-Based Access Control (ABAC)**: Policy decisions based on attributes
- **Temporal Access Control**: Time-based permission restrictions
- **Geo-Based Access**: Location-based access restrictions
- **Risk-Based Authentication**: Dynamic authentication requirements

### Integration Points

- **External Identity Providers**: OAuth, SAML integration
- **Multi-Factor Authentication**: Additional security layers
- **Session Management**: Advanced session handling
- **Audit Integration**: Centralized audit logging

### Scalability Improvements

- **Distributed Authorization**: Authorization across multiple nodes
- **Permission Replication**: Sync permissions across cluster
- **Caching Strategies**: Advanced caching with invalidation
- **Load Balancing**: Distribute authorization load

This access control system provides a solid foundation for securing the P2P AI Agents platform while remaining flexible enough to accommodate future security requirements.