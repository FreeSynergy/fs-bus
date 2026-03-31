// fs-bus/src/topic.rs — TopicHandler trait and glob pattern matching.

use async_trait::async_trait;

use crate::error::BusError;
use crate::event::Event;

// ── TopicHandler ──────────────────────────────────────────────────────────────

/// A subscriber that handles events matching a topic pattern.
///
/// Implement this trait on any type that wants to receive bus events.
/// The [`Router`](crate::router::Router) calls [`handle`](TopicHandler::handle)
/// for every event whose topic matches [`topic_pattern`](TopicHandler::topic_pattern).
///
/// # Pattern syntax
///
/// Topics use `::` as separator (`"registry::service::registered"`).
///
/// - `"registry::service::registered"` — exact match
/// - `"registry::*"` — namespace wildcard: matches all registry topics
/// - `"registry::service::*"` — trailing wildcard: matches `registry::service::registered` etc.
/// - `"#"` — greedy: matches any topic
///
/// # Example
///
/// ```rust,ignore
/// use fs_bus::{TopicHandler, Event, BusError};
/// use async_trait::async_trait;
///
/// struct DeployLogger;
///
/// #[async_trait]
/// impl TopicHandler for DeployLogger {
///     fn topic_pattern(&self) -> &str { "deploy::*" }
///
///     async fn handle(&self, event: &Event) -> Result<(), BusError> {
///         println!("deploy event: {}", event.topic());
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait TopicHandler: Send + Sync {
    /// Glob-style pattern this handler subscribes to.
    fn topic_pattern(&self) -> &str;

    /// Process an event that matched this handler's pattern.
    ///
    /// # Errors
    ///
    /// Returns [`BusError`] if the handler fails to process the event.
    async fn handle(&self, event: &Event) -> Result<(), BusError>;
}

// ── Pattern matching ──────────────────────────────────────────────────────────

/// Check whether `pattern` matches `topic`.
///
/// Topics and patterns use `::` as a segment separator, e.g.:
///   `"registry::service::registered"`, `"registry::service::*"`, `"registry::#"`.
///
/// - `#` as the last pattern segment matches any number of remaining `::‑separated` segments
///   (greedy namespace wildcard).
/// - `*` matches exactly **one** segment.
/// - Anything else must equal the corresponding segment literally.
///
/// Examples:
/// - `"registry::#"` matches all registry topics of any depth (greedy)
/// - `"registry::service::*"` matches `"registry::service::registered"` etc. (exact 3rd segment)
/// - `"registry::service::registered"` matches only itself (exact match)
/// - `"*"` matches any single-segment topic
#[must_use]
pub fn topic_matches(pattern: &str, topic: &str) -> bool {
    if pattern == "#" {
        return true;
    }
    let mut pat_parts = pattern.split("::");
    let mut top_parts = topic.split("::");

    loop {
        match (pat_parts.next(), top_parts.next()) {
            (Some("#"), _) | (None, None) => return true,
            (Some("*"), Some(_)) => {}
            (Some(p), Some(t)) if p == t => {}
            _ => return false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(topic_matches("deploy::started", "deploy::started"));
        assert!(!topic_matches("deploy::started", "deploy::failed"));
    }

    #[test]
    fn single_wildcard() {
        assert!(topic_matches("deploy::*", "deploy::started"));
        assert!(topic_matches("deploy::*", "deploy::failed"));
        assert!(!topic_matches("deploy::*", "deploy::started::now"));
    }

    #[test]
    fn greedy_wildcard() {
        assert!(topic_matches("#", "deploy::started"));
        assert!(topic_matches("#", "anything::at::all"));
        assert!(topic_matches("deploy::#", "deploy::started::now"));
    }

    #[test]
    fn no_match() {
        assert!(!topic_matches("health::*", "deploy::started"));
    }

    #[test]
    fn registry_namespace() {
        // # is the greedy namespace wildcard.
        assert!(topic_matches(
            "registry::#",
            "registry::service::registered"
        ));
        assert!(topic_matches("registry::#", "registry::service::stopped"));
        assert!(topic_matches("registry::#", "registry::capability::added"));
        assert!(!topic_matches("registry::#", "session::user::login"));
        // * matches exactly one segment.
        assert!(topic_matches(
            "registry::service::*",
            "registry::service::registered"
        ));
        assert!(!topic_matches(
            "registry::service::*",
            "registry::service::registered::extra"
        ));
    }
}
