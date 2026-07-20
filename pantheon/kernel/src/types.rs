use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LamportTimestamp(pub u64);

impl LamportTimestamp {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn increment(&mut self) -> Self {
        self.0 += 1;
        *self
    }

    pub fn max(a: Self, b: Self) -> Self {
        Self(a.0.max(b.0))
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for LamportTimestamp {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for LamportTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SequenceNumber(pub u64);

impl SequenceNumber {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn increment(&mut self) -> Self {
        self.0 += 1;
        *self
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for SequenceNumber {
    fn default() -> Self {
        Self(0)
    }
}

impl std::fmt::Display for SequenceNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "seq{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub Vec<u8>);

impl ContentHash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        let hash = Sha256::digest(bytes);
        Self(hash.to_vec())
    }

    pub fn as_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hash:{}", self.as_hex().get(..16).unwrap_or(&self.as_hex()))
    }
}

impl Default for ContentHash {
    fn default() -> Self {
        Self(Vec::new())
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamport_timestamp_increment() {
        let mut ts = LamportTimestamp::new(0);
        assert_eq!(ts.increment(), LamportTimestamp::new(1));
        assert_eq!(ts, LamportTimestamp::new(1));
    }

    #[test]
    fn test_lamport_timestamp_max() {
        let a = LamportTimestamp::new(5);
        let b = LamportTimestamp::new(10);
        assert_eq!(LamportTimestamp::max(a, b), LamportTimestamp::new(10));
        assert_eq!(LamportTimestamp::max(b, a), LamportTimestamp::new(10));
    }

    #[test]
    fn test_sequence_number_increment() {
        let mut seq = SequenceNumber::new(0);
        assert_eq!(seq.increment(), SequenceNumber::new(1));
        assert_eq!(seq, SequenceNumber::new(1));
    }

    #[test]
    fn test_content_hash() {
        let hash = ContentHash::from_bytes(b"hello world");
        assert!(!hash.is_empty());
        assert_eq!(
            hash.as_hex(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_content_hash_consistency() {
        let a = ContentHash::from_bytes(b"test data");
        let b = ContentHash::from_bytes(b"test data");
        assert_eq!(a, b);
    }
}
