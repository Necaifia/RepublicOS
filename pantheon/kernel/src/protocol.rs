use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolRef {
    pub name: String,
    pub version: ProtocolVersion,
}

impl ProtocolRef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: ProtocolVersion::new(1, 0, 0),
        }
    }

    pub fn with_version(name: impl Into<String>, major: u32, minor: u32, patch: u32) -> Self {
        Self {
            name: name.into(),
            version: ProtocolVersion::new(major, minor, patch),
        }
    }

    pub fn matches(&self, other: &ProtocolRef) -> bool {
        self.name == other.name && self.version.major == other.version.major
    }

    pub fn is_compatible(&self, other: &ProtocolRef) -> bool {
        self.name == other.name
            && self.version.major == other.version.major
            && self.version.minor >= other.version.minor
    }
}

impl fmt::Display for ProtocolRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.name, self.version)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ProtocolVersion {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone)]
pub struct ProtocolState {
    pub current: String,
    pub valid_transitions: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct MessageDef {
    pub name: String,
    pub direction: MessageDirection,
    pub description: &'static str,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageDirection {
    Request,
    Response,
    Both,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: &'static str,
    pub field_type: &'static str,
    pub description: &'static str,
    pub optional: bool,
}

pub trait Protocol: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> ProtocolVersion;
    fn messages(&self) -> Vec<MessageDef>;
    fn state_machine(&self) -> Vec<ProtocolState>;
    fn validate_message(&self, state: &str, message: &str) -> bool;
}

#[derive(Debug, Clone)]
pub struct ProtocolNegotiator {
    supported: HashMap<String, Vec<ProtocolVersion>>,
}

impl ProtocolNegotiator {
    pub fn new() -> Self {
        Self {
            supported: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, version: ProtocolVersion) {
        let entry = self.supported.entry(name.into()).or_default();
        if !entry.contains(&version) {
            entry.push(version);
            entry.sort_by(|a, b| b.major.cmp(&a.major).then(b.minor.cmp(&a.minor)));
        }
    }

    pub fn register_ref(&mut self, protocol_ref: &ProtocolRef) {
        self.register(&protocol_ref.name, protocol_ref.version);
    }

    pub fn negotiate(&self, requested: &ProtocolRef) -> Option<ProtocolRef> {
        let versions = self.supported.get(&requested.name)?;
        versions
            .iter()
            .find(|v| v.major == requested.version.major && v.minor >= requested.version.minor)
            .map(|v| ProtocolRef {
                name: requested.name.clone(),
                version: *v,
            })
    }

    pub fn supports(&self, protocol: &ProtocolRef) -> bool {
        self.supported
            .get(&protocol.name)
            .map(|versions| {
                versions
                    .iter()
                    .any(|v| v.major == protocol.version.major && v.minor >= protocol.version.minor)
            })
            .unwrap_or(false)
    }

    pub fn supported_protocols(&self) -> &HashMap<String, Vec<ProtocolVersion>> {
        &self.supported
    }
}

impl Default for ProtocolNegotiator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_ref_display() {
        let pr = ProtocolRef::with_version("pantheon.test", 1, 2, 3);
        assert_eq!(pr.to_string(), "pantheon.test/v1.2.3");
    }

    #[test]
    fn test_protocol_version_comparison() {
        let v1 = ProtocolVersion::new(1, 0, 0);
        let v2 = ProtocolVersion::new(2, 0, 0);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_protocol_match() {
        let a = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        let b = ProtocolRef::with_version("pantheon.test", 1, 2, 0);
        assert!(a.matches(&b));
        assert!(b.matches(&a));
    }

    #[test]
    fn test_protocol_no_match_different_major() {
        let a = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        let b = ProtocolRef::with_version("pantheon.test", 2, 0, 0);
        assert!(!a.matches(&b));
    }

    #[test]
    fn test_protocol_no_match_different_name() {
        let a = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        let b = ProtocolRef::with_version("pantheon.other", 1, 0, 0);
        assert!(!a.matches(&b));
    }

    #[test]
    fn test_protocol_compatibility() {
        let a = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        let b = ProtocolRef::with_version("pantheon.test", 1, 2, 0);
        assert!(b.is_compatible(&a));
        assert!(!a.is_compatible(&b)); // a.minor (0) < b.minor (2)
    }

    #[test]
    fn test_negotiator_register_and_negotiate() {
        let mut negotiator = ProtocolNegotiator::new();
        negotiator.register("pantheon.test", ProtocolVersion::new(1, 0, 0));
        negotiator.register("pantheon.test", ProtocolVersion::new(1, 2, 0));
        negotiator.register("pantheon.test", ProtocolVersion::new(2, 0, 0));

        let request = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        let result = negotiator.negotiate(&request);
        assert!(result.is_some());
        assert_eq!(result.unwrap().version, ProtocolVersion::new(1, 2, 0));
    }

    #[test]
    fn test_negotiator_unsupported_protocol() {
        let negotiator = ProtocolNegotiator::new();
        let request = ProtocolRef::with_version("pantheon.unknown", 1, 0, 0);
        assert!(negotiator.negotiate(&request).is_none());
    }

    #[test]
    fn test_negotiator_no_compatible_version() {
        let mut negotiator = ProtocolNegotiator::new();
        negotiator.register("pantheon.test", ProtocolVersion::new(2, 0, 0));

        let request = ProtocolRef::with_version("pantheon.test", 1, 0, 0);
        assert!(negotiator.negotiate(&request).is_none());
    }

    #[test]
    fn test_negotiator_supports() {
        let mut negotiator = ProtocolNegotiator::new();
        negotiator.register("pantheon.test", ProtocolVersion::new(1, 5, 0));

        assert!(negotiator.supports(&ProtocolRef::with_version("pantheon.test", 1, 0, 0)));
        assert!(!negotiator.supports(&ProtocolRef::with_version("pantheon.test", 2, 0, 0)));
    }
}
