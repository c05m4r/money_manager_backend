// Copyright (C) 2026 Marcos Gabriel Miller
use casbin::{CoreApi, DefaultModel, Enforcer, MemoryAdapter, MgmtApi};
use tokio::sync::RwLock;
use uuid::Uuid;

use std::sync::Arc;

use crate::errors::AppError;

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_MANAGER: &str = "manager";
pub const ROLE_AUDITOR: &str = "auditor";
pub const ROLE_USER: &str = "user";

pub type AuthorizationEnforcer = Arc<RwLock<Enforcer>>;

pub async fn build_enforcer() -> Result<AuthorizationEnforcer, AppError> {
    let model = DefaultModel::from_str(
        r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && keyMatch2(r.obj, p.obj) && regexMatch(r.act, p.act)
"#,
    )
    .await
    .map_err(|error| AppError::InternalServerError(format!("Failed to load Casbin model: {error}")))?;

    let adapter = MemoryAdapter::default();
    let mut enforcer = Enforcer::new(model, adapter)
        .await
        .map_err(|error| AppError::InternalServerError(format!("Failed to initialize Casbin: {error}")))?;

    let policies = vec![
        vec![ROLE_ADMIN.to_string(), "users".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "currencies".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "accounts".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "categories".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "transaction_types".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "transactions".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_ADMIN.to_string(), "audit_logs".to_string(), "list|get".to_string()],
        vec![ROLE_MANAGER.to_string(), "users".to_string(), "create".to_string()],
        vec![ROLE_MANAGER.to_string(), "users:own".to_string(), "list|get|update|delete".to_string()],
        vec![ROLE_MANAGER.to_string(), "currencies".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_MANAGER.to_string(), "accounts".to_string(), "create".to_string()],
        vec![ROLE_MANAGER.to_string(), "accounts:own".to_string(), "list|get|update|delete".to_string()],
        vec![ROLE_MANAGER.to_string(), "categories".to_string(), "create".to_string()],
        vec![ROLE_MANAGER.to_string(), "categories:own".to_string(), "list|get|update|delete".to_string()],
        vec![ROLE_MANAGER.to_string(), "transaction_types".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_MANAGER.to_string(), "transactions".to_string(), "create".to_string()],
        vec![ROLE_MANAGER.to_string(), "transactions:own".to_string(), "list|get|update|delete".to_string()],
        vec![ROLE_AUDITOR.to_string(), "users".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "currencies".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "accounts".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "categories".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "transaction_types".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "transactions".to_string(), "list|get".to_string()],
        vec![ROLE_AUDITOR.to_string(), "audit_logs".to_string(), "list|get".to_string()],
        vec![ROLE_USER.to_string(), "users:own".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_USER.to_string(), "currencies".to_string(), "list|get".to_string()],
        vec![ROLE_USER.to_string(), "transaction_types".to_string(), "list|get".to_string()],
        vec![ROLE_USER.to_string(), "accounts:own".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_USER.to_string(), "categories:own".to_string(), "list|get|create|update|delete".to_string()],
        vec![ROLE_USER.to_string(), "transactions:own".to_string(), "list|get|create|update|delete".to_string()],
    ];

    for policy in policies {
        enforcer
            .add_policy(policy)
            .await
            .map_err(|error| AppError::InternalServerError(format!("Failed to set Casbin policy: {error}")))?;
    }

    Ok(Arc::new(RwLock::new(enforcer)))
}

pub fn normalize_role(role: &str) -> String {
    match role.trim().to_ascii_lowercase().as_str() {
        ROLE_ADMIN => ROLE_ADMIN.to_string(),
        ROLE_MANAGER => ROLE_MANAGER.to_string(),
        ROLE_AUDITOR => ROLE_AUDITOR.to_string(),
        _ => ROLE_USER.to_string(),
    }
}

pub fn is_own_scoped_role(role: &str) -> bool {
    role == ROLE_USER || role == ROLE_MANAGER
}

fn casbin_object(resource: &str, owner_uuid: Option<Uuid>, actor_uuid: Uuid, role: &str) -> String {
    if is_own_scoped_role(role) {
        if owner_uuid.is_some_and(|owner| owner == actor_uuid) {
            return format!("{resource}:own");
        }
        return resource.to_string();
    }
    resource.to_string()
}

pub async fn authorize(
    enforcer: &AuthorizationEnforcer,
    role: &str,
    actor_uuid: Uuid,
    resource: &str,
    action: &str,
    owner_uuid: Option<Uuid>,
) -> Result<(), AppError> {
    let object = casbin_object(resource, owner_uuid, actor_uuid, role);
    let guard = enforcer.read().await;
    let allowed = guard
        .enforce((role, object.as_str(), action))
        .map_err(|error| AppError::InternalServerError(format!("Casbin enforce failed: {error}")))?;

    if allowed {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}
