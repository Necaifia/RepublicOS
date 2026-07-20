use pantheon_kernel::capability::{Action, CapabilityConstraints, CapabilityGrant, CapabilityId, ResourcePattern};
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::{Error, Result};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct CapabilityManager {
    grants: HashMap<CapabilityId, CapabilityGrant>,
    issued: HashMap<CapabilityId, CellId>,
    revocations: HashSet<CapabilityId>,
    start_time: std::time::Instant,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self {
            grants: HashMap::new(),
            issued: HashMap::new(),
            revocations: HashSet::new(),
            start_time: std::time::Instant::now(),
        }
    }

    pub fn issue(
        &mut self,
        issuer: CellId,
        subject: CellId,
        resource: impl Into<ResourcePattern>,
        actions: Vec<Action>,
        constraints: CapabilityConstraints,
    ) -> Result<CapabilityId> {
        let grant = CapabilityGrant::new(issuer, subject, resource, actions, constraints);
        let id = grant.capability_id.clone();
        self.issued.insert(id.clone(), issuer);
        self.grants.insert(id.clone(), grant);
        Ok(id)
    }

    pub fn delegate(
        &mut self,
        existing_id: &CapabilityId,
        new_subject: CellId,
        resource: Option<ResourcePattern>,
        actions: Option<Vec<Action>>,
        constraints: Option<CapabilityConstraints>,
    ) -> Result<CapabilityId> {
        let parent = self
            .grants
            .get(existing_id)
            .ok_or_else(|| Error::CapabilityDenied(format!("Capability not found: {}", existing_id)))?;

        let attenuated = parent.attenuate(new_subject, resource, actions, constraints)?;
        let id = attenuated.capability_id.clone();
        self.issued.insert(id.clone(), parent.issuer);
        self.grants.insert(id.clone(), attenuated);
        Ok(id)
    }

    pub fn verify(
        &self,
        capability_id: &CapabilityId,
        subject: &CellId,
        action: &Action,
        resource: &str,
    ) -> Result<()> {
        if self.revocations.contains(capability_id) {
            return Err(Error::CapabilityDenied("Capability has been revoked".into()));
        }

        let grant = self
            .grants
            .get(capability_id)
            .ok_or_else(|| Error::CapabilityDenied(format!("Capability not found: {}", capability_id)))?;

        if &grant.subject != subject {
            return Err(Error::CapabilityDenied(format!(
                "Capability belongs to {}, not {}",
                grant.subject, subject
            )));
        }

        if !grant.resource.matches(resource) {
            return Err(Error::CapabilityDenied(format!(
                "Resource '{}' does not match pattern '{}'",
                resource, grant.resource
            )));
        }

        if !grant.actions.contains(action) {
            return Err(Error::CapabilityDenied(format!(
                "Action '{}' not in allowed actions",
                action
            )));
        }

        let elapsed = self.start_time.elapsed().as_secs();
        if grant.constraints.is_expired(elapsed) {
            return Err(Error::CapabilityExpired(format!(
                "Capability {} expired (TTL: {}s, elapsed: {}s)",
                capability_id, grant.constraints.ttl_seconds, elapsed
            )));
        }

        if grant.constraints.uses_exhausted() {
            return Err(Error::CapabilityDenied("Max uses exhausted".into()));
        }

        if grant.constraints.cost_exhausted() {
            return Err(Error::CapabilityDenied("Cost budget exhausted".into()));
        }

        Ok(())
    }

    pub fn revoke(&mut self, capability_id: &CapabilityId) -> Result<()> {
        if !self.grants.contains_key(capability_id) {
            return Err(Error::CapabilityDenied(format!(
                "Capability not found: {}",
                capability_id
            )));
        }
        self.revocations.insert(capability_id.clone());
        Ok(())
    }

    pub fn check_chain(
        &self,
        capability_id: &CapabilityId,
        subject: &CellId,
        action: &Action,
        resource: &str,
    ) -> Result<()> {
        self.verify(capability_id, subject, action, resource)?;

        let mut current = capability_id.clone();
        loop {
            let grant = match self.grants.get(&current) {
                Some(g) => g,
                None => break,
            };

            match &grant.parent {
                Some(parent_id) => {
                    self.verify(parent_id, &grant.issuer, action, resource)?;
                    current = parent_id.clone();
                }
                None => break,
            }
        }

        Ok(())
    }

    pub fn list_for_subject(&self, subject: &CellId) -> Vec<&CapabilityGrant> {
        self.grants
            .values()
            .filter(|g| &g.subject == subject)
            .collect()
    }

    pub fn list_for_issuer(&self, issuer: &CellId) -> Vec<&CapabilityGrant> {
        self.grants
            .values()
            .filter(|g| &g.issuer == issuer)
            .collect()
    }

    pub fn get_grant(&self, id: &CapabilityId) -> Option<&CapabilityGrant> {
        self.grants.get(id)
    }

    pub fn use_capability(&mut self, capability_id: &CapabilityId) -> Result<()> {
        let grant = self
            .grants
            .get_mut(capability_id)
            .ok_or_else(|| Error::CapabilityDenied(format!("Capability not found: {}", capability_id)))?;

        if let Some(max) = grant.constraints.max_uses {
            if grant.constraints.use_count >= max {
                return Err(Error::CapabilityDenied("Max uses exhausted".into()));
            }
        }

        grant.constraints.use_count += 1;
        Ok(())
    }

    pub fn is_revoked(&self, capability_id: &CapabilityId) -> bool {
        self.revocations.contains(capability_id)
    }
}

impl Default for CapabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pantheon_kernel::capability::{Action, CapabilityConstraints};
    use pantheon_kernel::cell_id::CellId;

    #[test]
    fn test_issue_capability() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();

        let id = mgr
            .issue(
                issuer,
                subject,
                "memory://team-alpha/*",
                vec![Action::Read, Action::Write],
                CapabilityConstraints::unlimited(),
            )
            .unwrap();

        assert!(mgr.grants.contains_key(&id));
    }

    #[test]
    fn test_verify_capability() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();

        let id = mgr
            .issue(
                issuer,
                subject,
                "memory://team-alpha/config",
                vec![Action::Read],
                CapabilityConstraints::unlimited(),
            )
            .unwrap();

        assert!(mgr.verify(&id, &subject, &Action::Read, "memory://team-alpha/config").is_ok());
        assert!(mgr.verify(&id, &subject, &Action::Write, "memory://team-alpha/config").is_err());
        assert!(mgr.verify(&id, &subject, &Action::Read, "memory://other-team/config").is_err());
    }

    #[test]
    fn test_revoke_capability() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();

        let id = mgr
            .issue(
                issuer,
                subject,
                "memory://*",
                vec![Action::Read],
                CapabilityConstraints::unlimited(),
            )
            .unwrap();

        assert!(mgr.verify(&id, &subject, &Action::Read, "memory://anything").is_ok());

        mgr.revoke(&id).unwrap();
        assert!(mgr.is_revoked(&id));
        assert!(mgr.verify(&id, &subject, &Action::Read, "memory://anything").is_err());
    }

    #[test]
    fn test_delegate_and_chain_verify() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();
        let delegate = CellId::random();

        let parent_id = mgr
            .issue(
                issuer,
                subject,
                "memory://team-alpha/*",
                vec![Action::Read, Action::Write],
                CapabilityConstraints::unlimited(),
            )
            .unwrap();

        let child_id = mgr
            .delegate(
                &parent_id,
                delegate,
                Some(ResourcePattern::new("memory://team-alpha/config")),
                Some(vec![Action::Read]),
                None,
            )
            .unwrap();

        assert!(mgr
            .check_chain(&child_id, &delegate, &Action::Read, "memory://team-alpha/config")
            .is_ok());
    }

    #[test]
    fn test_list_for_subject() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();

        mgr.issue(issuer, subject, "mem://a/*", vec![Action::Read], CapabilityConstraints::unlimited())
            .unwrap();
        mgr.issue(issuer, subject, "mem://b/*", vec![Action::Write], CapabilityConstraints::unlimited())
            .unwrap();

        let list = mgr.list_for_subject(&subject);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_capability_use_count() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();
        let constraints = CapabilityConstraints {
            max_uses: Some(2),
            ..CapabilityConstraints::unlimited()
        };

        let id = mgr
            .issue(issuer, subject, "memory://*", vec![Action::Read], constraints)
            .unwrap();

        assert!(mgr.use_capability(&id).is_ok());
        assert!(mgr.use_capability(&id).is_ok());
        assert!(mgr.use_capability(&id).is_err());
    }

    #[test]
    fn test_capability_ttl_expiry() {
        let mut mgr = CapabilityManager::new();
        let issuer = CellId::random();
        let subject = CellId::random();
        let constraints = CapabilityConstraints {
            ttl_seconds: 0,
            ..CapabilityConstraints::unlimited()
        };

        let id = mgr
            .issue(issuer, subject, "memory://*", vec![Action::Read], constraints)
            .unwrap();

        // TTL 0 means it should be expired almost immediately
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(mgr.verify(&id, &subject, &Action::Read, "memory://test").is_err());
    }
}
