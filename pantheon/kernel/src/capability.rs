use serde::{Deserialize, Serialize};
use std::fmt;

use crate::cell_id::CellId;
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cap:{}", self.0)
    }
}

impl From<String> for CapabilityId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CapabilityId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Read,
    Write,
    Create,
    Delete,
    Execute,
    List,
    Delegate,
    Revoke,
    Custom(String),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Read => write!(f, "read"),
            Action::Write => write!(f, "write"),
            Action::Create => write!(f, "create"),
            Action::Delete => write!(f, "delete"),
            Action::Execute => write!(f, "execute"),
            Action::List => write!(f, "list"),
            Action::Delegate => write!(f, "delegate"),
            Action::Revoke => write!(f, "revoke"),
            Action::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcePattern(pub String);

impl ResourcePattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }

    pub fn matches(&self, resource: &str) -> bool {
        let pattern = self.0.trim_end_matches('*');
        if self.0.ends_with('*') {
            resource.starts_with(pattern)
        } else {
            resource == self.0
        }
    }

    pub fn is_subset_of(&self, other: &ResourcePattern) -> bool {
        if other.0.ends_with('*') {
            let prefix = other.0.trim_end_matches('*');
            self.0.starts_with(prefix)
        } else {
            self.0 == other.0
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ResourcePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ResourcePattern {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ResourcePattern {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub id: CapabilityId,
    pub issuer: CellId,
    pub subject: CellId,
    pub parent: Option<CapabilityId>,
    pub resource: ResourcePattern,
    pub actions: Vec<Action>,
    pub constraints: CapabilityConstraints,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConstraints {
    pub ttl_seconds: u64,
    pub max_uses: Option<u64>,
    pub use_count: u64,
    pub max_cost_tokens: Option<u64>,
    pub cost_used_tokens: u64,
    pub delegation_depth: u32,
    pub current_depth: u32,
}

impl CapabilityConstraints {
    pub fn unlimited() -> Self {
        Self {
            ttl_seconds: 86400,
            max_uses: None,
            use_count: 0,
            max_cost_tokens: None,
            cost_used_tokens: 0,
            delegation_depth: 5,
            current_depth: 0,
        }
    }

    pub fn is_expired(&self, elapsed_seconds: u64) -> bool {
        elapsed_seconds >= self.ttl_seconds
    }

    pub fn uses_exhausted(&self) -> bool {
        self.max_uses
            .map(|max| self.use_count >= max)
            .unwrap_or(false)
    }

    pub fn cost_exhausted(&self) -> bool {
        self.max_cost_tokens
            .map(|max| self.cost_used_tokens >= max)
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub capability_id: CapabilityId,
    pub issuer: CellId,
    pub subject: CellId,
    pub parent: Option<CapabilityId>,
    pub resource: ResourcePattern,
    pub actions: Vec<Action>,
    pub constraints: CapabilityConstraints,
}

impl CapabilityGrant {
    pub fn new(
        issuer: CellId,
        subject: CellId,
        resource: impl Into<ResourcePattern>,
        actions: Vec<Action>,
        constraints: CapabilityConstraints,
    ) -> Self {
        let resource_pattern: ResourcePattern = resource.into();
        let id = CapabilityId::new(format!(
            "cap://{}/{}",
            issuer,
            resource_pattern.as_str().replace('/', "_")
        ));
        Self {
            capability_id: id,
            issuer,
            subject,
            parent: None,
            resource: resource_pattern,
            actions,
            constraints,
        }
    }

    pub fn attenuate(
        &self,
        new_subject: CellId,
        resource: Option<ResourcePattern>,
        actions: Option<Vec<Action>>,
        new_constraints: Option<CapabilityConstraints>,
    ) -> Result<Self> {
        let new_resource = resource.unwrap_or_else(|| self.resource.clone());

        if !new_resource.is_subset_of(&self.resource) {
            return Err(Error::Custom(
                "Attenuated capability must have subset of resource".into(),
            ));
        }

        let new_actions = actions.unwrap_or_else(|| self.actions.clone());
        if !actions_are_subset(&new_actions, &self.actions) {
            return Err(Error::Custom(
                "Attenuated capability must have subset of actions".into(),
            ));
        }

        let new_constraints = new_constraints.unwrap_or_else(|| CapabilityConstraints {
            ttl_seconds: self.constraints.ttl_seconds,
            max_uses: self.constraints.max_uses,
            use_count: 0,
            max_cost_tokens: self.constraints.max_cost_tokens,
            cost_used_tokens: 0,
            delegation_depth: self.constraints.delegation_depth.saturating_sub(1),
            current_depth: self.constraints.current_depth + 1,
        });

        if new_constraints.ttl_seconds > self.constraints.ttl_seconds {
            return Err(Error::Custom(
                "Attenuated capability must have shorter or equal TTL".into(),
            ));
        }

        if new_constraints.delegation_depth >= self.constraints.delegation_depth {
            return Err(Error::Custom(
                "Attenuated capability must have shorter delegation depth".into(),
            ));
        }

        Ok(Self {
            capability_id: CapabilityId::new(format!("{}-{}", self.capability_id, new_subject)),
            issuer: self.subject,
            subject: new_subject,
            parent: Some(self.capability_id.clone()),
            resource: new_resource,
            actions: new_actions,
            constraints: new_constraints,
        })
    }
}

fn actions_are_subset(a: &[Action], b: &[Action]) -> bool {
    a.iter().all(|action| b.contains(action))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell_id::CellId;

    #[test]
    fn test_resource_pattern_match_exact() {
        let pattern = ResourcePattern::new("memory://team-alpha/config");
        assert!(pattern.matches("memory://team-alpha/config"));
        assert!(!pattern.matches("memory://team-alpha/other"));
    }

    #[test]
    fn test_resource_pattern_match_wildcard() {
        let pattern = ResourcePattern::new("memory://team-alpha/*");
        assert!(pattern.matches("memory://team-alpha/config"));
        assert!(pattern.matches("memory://team-alpha/data"));
        assert!(!pattern.matches("memory://other-team/config"));
    }

    #[test]
    fn test_resource_pattern_is_subset() {
        let broad = ResourcePattern::new("memory://*");
        let narrow = ResourcePattern::new("memory://team-alpha/*");
        assert!(narrow.is_subset_of(&broad));
        assert!(!broad.is_subset_of(&narrow));
    }

    #[test]
    fn test_capability_grant_attenuate_resource() {
        let issuer = CellId::random();
        let subject = CellId::random();
        let new_subject = CellId::random();

        let grant = CapabilityGrant::new(
            issuer,
            subject,
            "memory://team-alpha/*",
            vec![Action::Read, Action::Write],
            CapabilityConstraints::unlimited(),
        );

        let attenuated = grant
            .attenuate(
                new_subject,
                Some(ResourcePattern::new("memory://team-alpha/config")),
                None,
                None,
            )
            .unwrap();

        assert_eq!(attenuated.subject, new_subject);
        assert_eq!(attenuated.resource, ResourcePattern::new("memory://team-alpha/config"));
    }

    #[test]
    fn test_capability_grant_attenuate_rejects_broader() {
        let issuer = CellId::random();
        let subject = CellId::random();
        let new_subject = CellId::random();

        let grant = CapabilityGrant::new(
            issuer,
            subject,
            "memory://team-alpha/config",
            vec![Action::Read],
            CapabilityConstraints::unlimited(),
        );

        let result = grant.attenuate(
            new_subject,
            Some(ResourcePattern::new("memory://*")),
            None,
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_capability_grant_attenuate_actions() {
        let issuer = CellId::random();
        let subject = CellId::random();
        let new_subject = CellId::random();

        let grant = CapabilityGrant::new(
            issuer,
            subject,
            "memory://*",
            vec![Action::Read, Action::Write, Action::Delete],
            CapabilityConstraints::unlimited(),
        );

        let attenuated = grant
            .attenuate(
                new_subject,
                None,
                Some(vec![Action::Read]),
                None,
            )
            .unwrap();

        assert_eq!(attenuated.actions, vec![Action::Read]);
    }

    #[test]
    fn test_capability_grant_attenuate_rejects_longer_ttl() {
        let issuer = CellId::random();
        let subject = CellId::random();
        let new_subject = CellId::random();

        let grant = CapabilityGrant::new(
            issuer,
            subject,
            "memory://*",
            vec![Action::Read],
            CapabilityConstraints {
                ttl_seconds: 3600,
                max_uses: None,
                use_count: 0,
                max_cost_tokens: None,
                cost_used_tokens: 0,
                delegation_depth: 5,
                current_depth: 0,
            },
        );

        let result = grant.attenuate(
            new_subject,
            None,
            None,
            Some(CapabilityConstraints {
                ttl_seconds: 7200,
                ..CapabilityConstraints::unlimited()
            }),
        );

        assert!(result.is_err());
    }
}
