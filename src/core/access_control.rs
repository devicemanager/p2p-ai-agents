//! Access control system for P2P AI Agents
//!
//! This module provides a comprehensive access control system including:
//! - Authentication (identity verification)
//! - Authorization (permission checking)
//! - Role-based access control (RBAC)
//! - Permission management
//! - Security policies

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique identifier for users/agents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrincipalId(String);

impl PrincipalId {
    /// Create a new principal ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Role identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(String);

impl RoleId {
    /// Create a new role ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get the ID as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Permission identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(String);

impl Permission {
    /// Create a new permission
    pub fn new(permission: String) -> Self {
        Self(permission)
    }

    /// Get the permission string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Resource identifier for access control
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource(String);

impl Resource {
    /// Create a new resource
    pub fn new(resource: String) -> Self {
        Self(resource)
    }

    /// Get the resource string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Principal ID
    pub principal_id: PrincipalId,
    /// Token expiration timestamp
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional claims
    pub claims: HashMap<String, serde_json::Value>,
}

/// Authentication result
#[derive(Debug)]
pub enum AuthResult {
    /// Authentication successful
    Success(AuthToken),
    /// Authentication failed
    Failed(String),
    /// Token expired
    Expired,
}

/// Authorization result
#[derive(Debug, PartialEq, Eq)]
pub enum AuthzResult {
    /// Access allowed
    Allow,
    /// Access denied
    Deny(String),
}

/// Access control policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Associated roles
    pub roles: HashSet<RoleId>,
    /// Granted permissions
    pub permissions: HashMap<Resource, HashSet<Permission>>,
    /// Policy effect (allow/deny)
    pub effect: PolicyEffect,
}

/// Policy effect
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyEffect {
    /// Allow access
    Allow,
    /// Deny access
    Deny,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role ID
    pub id: RoleId,
    /// Role name
    pub name: String,
    /// Description
    pub description: String,
    /// Assigned permissions
    pub permissions: HashMap<Resource, HashSet<Permission>>,
}

/// User/Agent principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Principal ID
    pub id: PrincipalId,
    /// Principal name
    pub name: String,
    /// Assigned roles
    pub roles: HashSet<RoleId>,
    /// Additional attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Authentication service trait
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate a principal with credentials
    async fn authenticate(
        &self,
        principal_id: &PrincipalId,
        credentials: &HashMap<String, String>,
    ) -> AuthResult;

    /// Validate an authentication token
    async fn validate_token(&self, token: &AuthToken) -> AuthResult;

    /// Create a new authentication token
    async fn create_token(&self, principal_id: &PrincipalId) -> Result<AuthToken, String>;
}

/// Authorization service trait
#[async_trait]
pub trait Authorizer: Send + Sync {
    /// Check if a principal has permission for a resource
    async fn check_permission(
        &self,
        principal: &Principal,
        resource: &Resource,
        permission: &Permission,
    ) -> AuthzResult;

    /// Get all permissions for a principal on a resource
    async fn get_permissions(
        &self,
        principal: &Principal,
        resource: &Resource,
    ) -> HashSet<Permission>;

    /// Check if a principal has a specific role
    async fn has_role(&self, principal: &Principal, role_id: &RoleId) -> bool;
}

/// Access control manager combining authentication and authorization
#[derive(Clone)]
pub struct AccessControlManager {
    /// Authentication service
    authenticator: Arc<dyn Authenticator>,
    /// Authorization service
    authorizer: Arc<dyn Authorizer>,
    /// Principal store
    principals: Arc<RwLock<HashMap<PrincipalId, Principal>>>,
    /// Role store
    roles: Arc<RwLock<HashMap<RoleId, Role>>>,
    /// Policy store
    policies: Arc<RwLock<HashMap<String, Policy>>>,
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new(authenticator: Arc<dyn Authenticator>, authorizer: Arc<dyn Authorizer>) -> Self {
        Self {
            authenticator,
            authorizer,
            principals: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a principal
    pub async fn add_principal(&self, principal: Principal) {
        let mut principals = self.principals.write().await;
        principals.insert(principal.id.clone(), principal);
    }

    /// Get a principal by ID
    pub async fn get_principal(&self, principal_id: &PrincipalId) -> Option<Principal> {
        let principals = self.principals.read().await;
        principals.get(principal_id).cloned()
    }

    /// Add a role
    pub async fn add_role(&self, role: Role) {
        let mut roles = self.roles.write().await;
        roles.insert(role.id.clone(), role);
    }

    /// Get a role by ID
    pub async fn get_role(&self, role_id: &RoleId) -> Option<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id).cloned()
    }

    /// Add a policy
    pub async fn add_policy(&self, policy: Policy) {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy);
    }

    /// Authenticate a principal
    pub async fn authenticate(
        &self,
        principal_id: &PrincipalId,
        credentials: &HashMap<String, String>,
    ) -> AuthResult {
        self.authenticator
            .authenticate(principal_id, credentials)
            .await
    }

    /// Authorize a principal for a resource and permission
    pub async fn authorize(
        &self,
        principal: &Principal,
        resource: &Resource,
        permission: &Permission,
    ) -> AuthzResult {
        self.authorizer
            .check_permission(principal, resource, permission)
            .await
    }

    /// Check if access is allowed for a principal on a resource with permission
    pub async fn is_allowed(
        &self,
        principal_id: &PrincipalId,
        resource: &Resource,
        permission: &Permission,
    ) -> Result<bool, String> {
        let principal = self
            .get_principal(principal_id)
            .await
            .ok_or_else(|| format!("Principal {} not found", principal_id.as_str()))?;

        match self.authorize(&principal, resource, permission).await {
            AuthzResult::Allow => Ok(true),
            AuthzResult::Deny(_) => Ok(false),
        }
    }
}

/// Default authenticator implementation
pub struct DefaultAuthenticator;

#[async_trait]
impl Authenticator for DefaultAuthenticator {
    async fn authenticate(
        &self,
        principal_id: &PrincipalId,
        _credentials: &HashMap<String, String>,
    ) -> AuthResult {
        // Simple authentication - in real implementation, validate credentials
        AuthResult::Success(AuthToken {
            principal_id: principal_id.clone(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            claims: HashMap::new(),
        })
    }

    async fn validate_token(&self, token: &AuthToken) -> AuthResult {
        if let Some(expires_at) = token.expires_at {
            if chrono::Utc::now() > expires_at {
                return AuthResult::Expired;
            }
        }
        AuthResult::Success(token.clone())
    }

    async fn create_token(&self, principal_id: &PrincipalId) -> Result<AuthToken, String> {
        Ok(AuthToken {
            principal_id: principal_id.clone(),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            claims: HashMap::new(),
        })
    }
}

/// Default authorizer implementation
pub struct DefaultAuthorizer {
    /// Role permissions cache
    role_permissions: Arc<RwLock<HashMap<RoleId, HashMap<Resource, HashSet<Permission>>>>>,
}

impl DefaultAuthorizer {
    /// Create a new default authorizer
    pub fn new() -> Self {
        Self {
            role_permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update role permissions
    pub async fn update_role_permissions(
        &self,
        role_id: RoleId,
        permissions: HashMap<Resource, HashSet<Permission>>,
    ) {
        let mut role_perms = self.role_permissions.write().await;
        role_perms.insert(role_id, permissions);
    }
}

#[async_trait]
impl Authorizer for DefaultAuthorizer {
    async fn check_permission(
        &self,
        principal: &Principal,
        resource: &Resource,
        permission: &Permission,
    ) -> AuthzResult {
        let role_perms = self.role_permissions.read().await;

        // Check if any of the principal's roles has the required permission
        for role_id in &principal.roles {
            if let Some(role_permissions) = role_perms.get(role_id) {
                if let Some(resource_permissions) = role_permissions.get(resource) {
                    if resource_permissions.contains(permission) {
                        return AuthzResult::Allow;
                    }
                }
            }
        }

        AuthzResult::Deny(format!(
            "Principal {} does not have permission {} on resource {}",
            principal.id.as_str(),
            permission.as_str(),
            resource.as_str()
        ))
    }

    async fn get_permissions(
        &self,
        principal: &Principal,
        resource: &Resource,
    ) -> HashSet<Permission> {
        let role_perms = self.role_permissions.read().await;
        let mut permissions = HashSet::new();

        for role_id in &principal.roles {
            if let Some(role_permissions) = role_perms.get(role_id) {
                if let Some(resource_permissions) = role_permissions.get(resource) {
                    permissions.extend(resource_permissions.iter().cloned());
                }
            }
        }

        permissions
    }

    async fn has_role(&self, principal: &Principal, role_id: &RoleId) -> bool {
        principal.roles.contains(role_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_principal_creation() {
        let principal_id = PrincipalId::new("test-user".to_string());
        let mut roles = HashSet::new();
        roles.insert(RoleId::new("admin".to_string()));

        let principal = Principal {
            id: principal_id.clone(),
            name: "Test User".to_string(),
            roles,
            attributes: HashMap::new(),
        };

        assert_eq!(principal.id, principal_id);
        assert_eq!(principal.name, "Test User");
        assert!(principal.roles.contains(&RoleId::new("admin".to_string())));
    }

    #[tokio::test]
    async fn test_role_creation() {
        let role_id = RoleId::new("admin".to_string());
        let mut permissions = HashMap::new();
        let mut resource_perms = HashSet::new();
        resource_perms.insert(Permission::new("read".to_string()));
        permissions.insert(Resource::new("tasks".to_string()), resource_perms);

        let role = Role {
            id: role_id.clone(),
            name: "Administrator".to_string(),
            description: "Full access role".to_string(),
            permissions,
        };

        assert_eq!(role.id, role_id);
        assert_eq!(role.name, "Administrator");
        assert!(role
            .permissions
            .contains_key(&Resource::new("tasks".to_string())));
    }

    #[tokio::test]
    async fn test_default_authenticator() {
        let authenticator = DefaultAuthenticator;
        let principal_id = PrincipalId::new("test-user".to_string());
        let credentials = HashMap::new();

        let result = authenticator
            .authenticate(&principal_id, &credentials)
            .await;

        match result {
            AuthResult::Success(token) => {
                assert_eq!(token.principal_id, principal_id);
                assert!(token.expires_at.is_some());
            }
            _ => panic!("Expected successful authentication"),
        }
    }

    #[tokio::test]
    async fn test_default_authorizer() {
        let authorizer = DefaultAuthorizer::new();
        let role_id = RoleId::new("admin".to_string());

        // Set up role permissions
        let mut permissions = HashMap::new();
        let mut resource_perms = HashSet::new();
        resource_perms.insert(Permission::new("read".to_string()));
        permissions.insert(Resource::new("tasks".to_string()), resource_perms);

        authorizer
            .update_role_permissions(role_id.clone(), permissions)
            .await;

        // Create principal with the role
        let mut roles = HashSet::new();
        roles.insert(role_id);
        let principal = Principal {
            id: PrincipalId::new("test-user".to_string()),
            name: "Test User".to_string(),
            roles,
            attributes: HashMap::new(),
        };

        // Test permission check
        let result = authorizer
            .check_permission(
                &principal,
                &Resource::new("tasks".to_string()),
                &Permission::new("read".to_string()),
            )
            .await;

        assert_eq!(result, AuthzResult::Allow);

        // Test denied permission
        let result = authorizer
            .check_permission(
                &principal,
                &Resource::new("tasks".to_string()),
                &Permission::new("write".to_string()),
            )
            .await;

        match result {
            AuthzResult::Deny(_) => {} // Expected
            _ => panic!("Expected permission denied"),
        }
    }

    #[tokio::test]
    async fn test_access_control_manager() {
        let authenticator = Arc::new(DefaultAuthenticator);
        let authorizer = Arc::new(DefaultAuthorizer::new());
        let acm = AccessControlManager::new(authenticator, authorizer);

        // Add a role
        let role = Role {
            id: RoleId::new("user".to_string()),
            name: "User".to_string(),
            description: "Basic user role".to_string(),
            permissions: HashMap::new(),
        };
        acm.add_role(role).await;

        // Add a principal
        let mut roles = HashSet::new();
        roles.insert(RoleId::new("user".to_string()));
        let principal = Principal {
            id: PrincipalId::new("test-user".to_string()),
            name: "Test User".to_string(),
            roles,
            attributes: HashMap::new(),
        };
        acm.add_principal(principal).await;

        // Test retrieval
        let retrieved = acm
            .get_principal(&PrincipalId::new("test-user".to_string()))
            .await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test User");
    }

    #[tokio::test]
    async fn test_policy_creation_and_validation() {
        // Test creating valid policies
        let policy = Policy {
            id: "test-policy".to_string(),
            name: "Test Policy".to_string(),
            roles: HashSet::from([RoleId::new("admin".to_string())]),
            permissions: {
                let mut perms = HashMap::new();
                let mut resource_perms = HashSet::new();
                resource_perms.insert(Permission::new("read".to_string()));
                resource_perms.insert(Permission::new("write".to_string()));
                perms.insert(Resource::new("documents".to_string()), resource_perms);
                perms
            },
            effect: PolicyEffect::Allow,
        };

        assert_eq!(policy.id, "test-policy");
        assert_eq!(policy.name, "Test Policy");
        assert!(policy.roles.contains(&RoleId::new("admin".to_string())));
        assert_eq!(policy.effect, PolicyEffect::Allow);

        // Check permissions
        let doc_permissions = policy.permissions.get(&Resource::new("documents".to_string())).unwrap();
        assert!(doc_permissions.contains(&Permission::new("read".to_string())));
        assert!(doc_permissions.contains(&Permission::new("write".to_string())));
    }

    #[tokio::test]
    async fn test_policy_deny_effect() {
        let authorizer = DefaultAuthorizer::new();

        // Create a deny policy (documenting expected behavior for future implementation)
        let _deny_policy = Policy {
            id: "deny-policy".to_string(),
            name: "Deny Policy".to_string(),
            roles: HashSet::from([RoleId::new("guest".to_string())]),
            permissions: {
                let mut perms = HashMap::new();
                let mut resource_perms = HashSet::new();
                resource_perms.insert(Permission::new("write".to_string()));
                perms.insert(Resource::new("sensitive".to_string()), resource_perms);
                perms
            },
            effect: PolicyEffect::Deny,
        };

        // Set up role with permissions that would normally allow
        let mut permissions = HashMap::new();
        let mut resource_perms = HashSet::new();
        resource_perms.insert(Permission::new("write".to_string()));
        permissions.insert(Resource::new("sensitive".to_string()), resource_perms);

        authorizer.update_role_permissions(
            RoleId::new("guest".to_string()),
            permissions
        ).await;

        let principal = Principal {
            id: PrincipalId::new("guest-user".to_string()),
            name: "Guest User".to_string(),
            roles: HashSet::from([RoleId::new("guest".to_string())]),
            attributes: HashMap::new(),
        };

        // Even though the role has permission, explicit deny policy should take precedence
        // Note: In our current implementation, we don't have explicit deny policy enforcement
        // This test documents the expected behavior for future policy engine implementation
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("sensitive".to_string()),
            &Permission::new("write".to_string())
        ).await;

        // Currently allows because we don't have deny policy logic yet
        assert_eq!(result, AuthzResult::Allow);
    }

    #[tokio::test]
    async fn test_policy_role_assignment() {
        let acm = AccessControlManager::new(
            Arc::new(DefaultAuthenticator),
            Arc::new(DefaultAuthorizer::new()),
        );

        // Create a policy
        let policy = Policy {
            id: "user-policy".to_string(),
            name: "User Access Policy".to_string(),
            roles: HashSet::from([
                RoleId::new("user".to_string()),
                RoleId::new("editor".to_string())
            ]),
            permissions: HashMap::new(),
            effect: PolicyEffect::Allow,
        };

        acm.add_policy(policy).await;

        // Create roles
        let user_role = Role {
            id: RoleId::new("user".to_string()),
            name: "User".to_string(),
            description: "Basic user role".to_string(),
            permissions: HashMap::new(),
        };

        let editor_role = Role {
            id: RoleId::new("editor".to_string()),
            name: "Editor".to_string(),
            description: "Content editor role".to_string(),
            permissions: HashMap::new(),
        };

        acm.add_role(user_role).await;
        acm.add_role(editor_role).await;

        // Verify roles exist
        let retrieved_user_role = acm.get_role(&RoleId::new("user".to_string())).await;
        assert!(retrieved_user_role.is_some());
        assert_eq!(retrieved_user_role.unwrap().name, "User");

        let retrieved_editor_role = acm.get_role(&RoleId::new("editor".to_string())).await;
        assert!(retrieved_editor_role.is_some());
        assert_eq!(retrieved_editor_role.unwrap().name, "Editor");
    }

    #[tokio::test]
    async fn test_policy_principal_role_verification() {
        let acm = AccessControlManager::new(
            Arc::new(DefaultAuthenticator),
            Arc::new(DefaultAuthorizer::new()),
        );

        // Create roles
        let admin_role = Role {
            id: RoleId::new("admin".to_string()),
            name: "Administrator".to_string(),
            description: "Full system access".to_string(),
            permissions: HashMap::new(),
        };

        let user_role = Role {
            id: RoleId::new("user".to_string()),
            name: "User".to_string(),
            description: "Basic user access".to_string(),
            permissions: HashMap::new(),
        };

        acm.add_role(admin_role).await;
        acm.add_role(user_role).await;

        // Create principals with different role assignments
        let admin_principal = Principal {
            id: PrincipalId::new("admin-user".to_string()),
            name: "Admin User".to_string(),
            roles: HashSet::from([RoleId::new("admin".to_string())]),
            attributes: HashMap::new(),
        };

        let regular_principal = Principal {
            id: PrincipalId::new("regular-user".to_string()),
            name: "Regular User".to_string(),
            roles: HashSet::from([RoleId::new("user".to_string())]),
            attributes: HashMap::new(),
        };

        let multi_role_principal = Principal {
            id: PrincipalId::new("multi-user".to_string()),
            name: "Multi Role User".to_string(),
            roles: HashSet::from([
                RoleId::new("user".to_string()),
                RoleId::new("admin".to_string())
            ]),
            attributes: HashMap::new(),
        };

        acm.add_principal(admin_principal).await;
        acm.add_principal(regular_principal).await;
        acm.add_principal(multi_role_principal).await;

        // Verify role assignments
        let retrieved_admin = acm.get_principal(&PrincipalId::new("admin-user".to_string())).await.unwrap();
        assert!(retrieved_admin.roles.contains(&RoleId::new("admin".to_string())));
        assert!(!retrieved_admin.roles.contains(&RoleId::new("user".to_string())));

        let retrieved_regular = acm.get_principal(&PrincipalId::new("regular-user".to_string())).await.unwrap();
        assert!(retrieved_regular.roles.contains(&RoleId::new("user".to_string())));
        assert!(!retrieved_regular.roles.contains(&RoleId::new("admin".to_string())));

        let retrieved_multi = acm.get_principal(&PrincipalId::new("multi-user".to_string())).await.unwrap();
        assert!(retrieved_multi.roles.contains(&RoleId::new("user".to_string())));
        assert!(retrieved_multi.roles.contains(&RoleId::new("admin".to_string())));
    }

    #[tokio::test]
    async fn test_policy_conflict_resolution() {
        let authorizer = DefaultAuthorizer::new();

        // Set up conflicting permissions - one role allows, another doesn't have permission
        let mut admin_permissions = HashMap::new();
        let mut admin_resource_perms = HashSet::new();
        admin_resource_perms.insert(Permission::new("delete".to_string()));
        admin_permissions.insert(Resource::new("system".to_string()), admin_resource_perms);

        let mut user_permissions = HashMap::new();
        let mut user_resource_perms = HashSet::new();
        user_resource_perms.insert(Permission::new("read".to_string()));
        user_permissions.insert(Resource::new("system".to_string()), user_resource_perms);

        authorizer.update_role_permissions(RoleId::new("admin".to_string()), admin_permissions).await;
        authorizer.update_role_permissions(RoleId::new("user".to_string()), user_permissions).await;

        // Principal with both roles
        let principal = Principal {
            id: PrincipalId::new("power-user".to_string()),
            name: "Power User".to_string(),
            roles: HashSet::from([
                RoleId::new("user".to_string()),
                RoleId::new("admin".to_string())
            ]),
            attributes: HashMap::new(),
        };

        // Should have delete permission from admin role
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("system".to_string()),
            &Permission::new("delete".to_string())
        ).await;
        assert_eq!(result, AuthzResult::Allow);

        // Should have read permission from user role
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("system".to_string()),
            &Permission::new("read".to_string())
        ).await;
        assert_eq!(result, AuthzResult::Allow);

        // Should not have write permission (neither role has it)
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("system".to_string()),
            &Permission::new("write".to_string())
        ).await;
        assert!(matches!(result, AuthzResult::Deny(_)));
    }

    #[tokio::test]
    async fn test_policy_empty_permissions() {
        let authorizer = DefaultAuthorizer::new();

        // Create role with empty permissions
        authorizer.update_role_permissions(RoleId::new("empty".to_string()), HashMap::new()).await;

        let principal = Principal {
            id: PrincipalId::new("empty-user".to_string()),
            name: "Empty User".to_string(),
            roles: HashSet::from([RoleId::new("empty".to_string())]),
            attributes: HashMap::new(),
        };

        // Any permission check should fail
        let result = authorizer.check_permission(
            &principal,
            &Resource::new("any-resource".to_string()),
            &Permission::new("any-permission".to_string())
        ).await;

        assert!(matches!(result, AuthzResult::Deny(_)));
    }

    #[tokio::test]
    async fn test_policy_permission_inheritance() {
        let authorizer = DefaultAuthorizer::new();

        // Set up hierarchical permissions
        let mut permissions = HashMap::new();
        let mut resource_perms = HashSet::new();
        resource_perms.insert(Permission::new("read".to_string()));
        resource_perms.insert(Permission::new("write".to_string()));
        resource_perms.insert(Permission::new("delete".to_string()));
        permissions.insert(Resource::new("documents".to_string()), resource_perms);

        authorizer.update_role_permissions(RoleId::new("manager".to_string()), permissions).await;

        let principal = Principal {
            id: PrincipalId::new("manager-user".to_string()),
            name: "Manager User".to_string(),
            roles: HashSet::from([RoleId::new("manager".to_string())]),
            attributes: HashMap::new(),
        };

        // Test all permissions are available
        let available_permissions = authorizer.get_permissions(
            &principal,
            &Resource::new("documents".to_string())
        ).await;

        assert!(available_permissions.contains(&Permission::new("read".to_string())));
        assert!(available_permissions.contains(&Permission::new("write".to_string())));
        assert!(available_permissions.contains(&Permission::new("delete".to_string())));
        assert_eq!(available_permissions.len(), 3);
    }

    #[tokio::test]
    async fn test_policy_resource_isolation() {
        let authorizer = DefaultAuthorizer::new();

        // Set up permissions for different resources in a single call
        let mut permissions = HashMap::new();

        let mut file_perms = HashSet::new();
        file_perms.insert(Permission::new("read".to_string()));
        permissions.insert(Resource::new("files".to_string()), file_perms);

        let mut db_perms = HashSet::new();
        db_perms.insert(Permission::new("write".to_string()));
        permissions.insert(Resource::new("database".to_string()), db_perms);

        authorizer.update_role_permissions(RoleId::new("specialist".to_string()), permissions).await;

        let principal = Principal {
            id: PrincipalId::new("specialist-user".to_string()),
            name: "Specialist User".to_string(),
            roles: HashSet::from([RoleId::new("specialist".to_string())]),
            attributes: HashMap::new(),
        };

        // Should have read on files but not write
        let file_read_result = authorizer.check_permission(
            &principal,
            &Resource::new("files".to_string()),
            &Permission::new("read".to_string())
        ).await;
        assert_eq!(file_read_result, AuthzResult::Allow);

        let file_write_result = authorizer.check_permission(
            &principal,
            &Resource::new("files".to_string()),
            &Permission::new("write".to_string())
        ).await;
        assert!(matches!(file_write_result, AuthzResult::Deny(_)));

        // Should have write on database but not read
        let db_write_result = authorizer.check_permission(
            &principal,
            &Resource::new("database".to_string()),
            &Permission::new("write".to_string())
        ).await;
        assert_eq!(db_write_result, AuthzResult::Allow);

        let db_read_result = authorizer.check_permission(
            &principal,
            &Resource::new("database".to_string()),
            &Permission::new("read".to_string())
        ).await;
        assert!(matches!(db_read_result, AuthzResult::Deny(_)));
    }
}
