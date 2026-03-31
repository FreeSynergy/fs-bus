// fs-bus/src/topics.rs — Standard topic constants for the FreeSynergy event bus.
//
// All well-known topics are defined here as `pub const` strings.
// Use these constants everywhere instead of raw string literals to prevent
// typos and to make refactoring easier.
//
// # Namespace convention
//
//   domain::entity::action
//
//   domain   — the service that owns this event (registry, session, …)
//   entity   — the thing that changed (service, user, package, …)
//   action   — what happened (registered, login, installed, …)
//
// # Wildcard usage
//
//   "registry::*"     — all registry events
//   "registry::service::*"  — all service events in the registry namespace

// ── registry:: ────────────────────────────────────────────────────────────────

/// A service has registered itself with `fs-registry`.
pub const REGISTRY_SERVICE_REGISTERED: &str = "registry::service::registered";

/// A service has stopped and deregistered from `fs-registry`.
pub const REGISTRY_SERVICE_STOPPED: &str = "registry::service::stopped";

/// A new capability has been advertised in `fs-registry`.
pub const REGISTRY_CAPABILITY_ADDED: &str = "registry::capability::added";

/// A capability has been removed from `fs-registry`.
pub const REGISTRY_CAPABILITY_REMOVED: &str = "registry::capability::removed";

// ── session:: ─────────────────────────────────────────────────────────────────

/// A user has logged in to a session.
pub const SESSION_USER_LOGIN: &str = "session::user::login";

/// A user has logged out of a session.
pub const SESSION_USER_LOGOUT: &str = "session::user::logout";

/// An application window was opened in a session.
pub const SESSION_APP_OPENED: &str = "session::app::opened";

/// An application window was closed in a session.
pub const SESSION_APP_CLOSED: &str = "session::app::closed";

// ── inventory:: ───────────────────────────────────────────────────────────────

/// A package was successfully installed.
pub const INVENTORY_PACKAGE_INSTALLED: &str = "inventory::package::installed";

/// A package was removed.
pub const INVENTORY_PACKAGE_REMOVED: &str = "inventory::package::removed";

/// A package was updated to a new version.
pub const INVENTORY_PACKAGE_UPDATED: &str = "inventory::package::updated";

// ── system:: ──────────────────────────────────────────────────────────────────

/// System health has degraded below a configured threshold.
pub const SYSTEM_HEALTH_DEGRADED: &str = "system::health::degraded";

/// System health has returned to normal after a degraded period.
pub const SYSTEM_HEALTH_RESTORED: &str = "system::health::restored";

/// Node boot completed successfully.
pub const SYSTEM_NODE_STARTED: &str = "system::node::started";

/// Node is shutting down.
pub const SYSTEM_NODE_STOPPING: &str = "system::node::stopping";

// ── auth:: ────────────────────────────────────────────────────────────────────

/// A new user account was created.
pub const AUTH_USER_CREATED: &str = "auth::user::created";

/// A user account was deleted.
pub const AUTH_USER_DELETED: &str = "auth::user::deleted";

/// User credentials were updated (password change, MFA, …).
pub const AUTH_USER_UPDATED: &str = "auth::user::updated";

/// An access token was issued.
pub const AUTH_TOKEN_ISSUED: &str = "auth::token::issued";

/// An access token was revoked.
pub const AUTH_TOKEN_REVOKED: &str = "auth::token::revoked";

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_topics_are_namespaced() {
        let topics = [
            REGISTRY_SERVICE_REGISTERED,
            REGISTRY_SERVICE_STOPPED,
            REGISTRY_CAPABILITY_ADDED,
            REGISTRY_CAPABILITY_REMOVED,
            SESSION_USER_LOGIN,
            SESSION_USER_LOGOUT,
            SESSION_APP_OPENED,
            SESSION_APP_CLOSED,
            INVENTORY_PACKAGE_INSTALLED,
            INVENTORY_PACKAGE_REMOVED,
            INVENTORY_PACKAGE_UPDATED,
            SYSTEM_HEALTH_DEGRADED,
            SYSTEM_HEALTH_RESTORED,
            SYSTEM_NODE_STARTED,
            SYSTEM_NODE_STOPPING,
            AUTH_USER_CREATED,
            AUTH_USER_DELETED,
            AUTH_USER_UPDATED,
            AUTH_TOKEN_ISSUED,
            AUTH_TOKEN_REVOKED,
        ];
        for t in topics {
            assert!(t.contains("::"), "topic must be namespaced: {t}");
            assert_eq!(t.split("::").count(), 3, "topic must have 3 segments: {t}");
        }
    }

    #[test]
    fn topics_are_snake_case_lowercase() {
        let topics = [
            REGISTRY_SERVICE_REGISTERED,
            SESSION_USER_LOGIN,
            INVENTORY_PACKAGE_INSTALLED,
            SYSTEM_HEALTH_DEGRADED,
            AUTH_TOKEN_ISSUED,
        ];
        for t in topics {
            assert_eq!(t, t.to_lowercase().as_str(), "topic must be lowercase: {t}");
        }
    }
}
