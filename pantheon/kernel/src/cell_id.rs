use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellId(pub [u8; 32]);

impl CellId {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, crate::error::Error> {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| crate::error::Error::InvalidArgument("CellId must be 32 bytes".into()))?;
        Ok(Self(arr))
    }

    pub fn random() -> Self {
        use rand::Rng;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill(&mut bytes);
        Self(bytes)
    }

    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn from_hex(hex: &str) -> Result<Self, crate::error::Error> {
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::error::Error::InvalidArgument(format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cell:{}", &self.to_hex()[..16])
    }
}

impl fmt::Debug for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CellId({})", self)
    }
}

impl FromStr for CellId {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix("cell:").unwrap_or(s);
        Self::from_hex(hex)
    }
}

impl Default for CellId {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_cell_id() {
        let id1 = CellId::random();
        let id2 = CellId::random();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_zero_cell_id() {
        let id = CellId::zero();
        assert_eq!(id.0, [0u8; 32]);
    }

    #[test]
    fn test_to_from_hex() {
        let id = CellId::random();
        let hex = id.to_hex();
        let id2 = CellId::from_hex(&hex).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_from_str() {
        let id = CellId::random();
        let s = format!("cell:{}", id.to_hex());
        let id2: CellId = s.parse().unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_display_truncated() {
        let id = CellId::random();
        let s = id.to_string();
        assert!(s.starts_with("cell:"));
        assert_eq!(s.len(), 21); // "cell:" + 16 hex chars
    }

    #[test]
    fn test_from_bytes_wrong_length() {
        let result = CellId::from_bytes(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cell_id_ordering() {
        let a = CellId([1u8; 32]);
        let b = CellId([2u8; 32]);
        assert!(a < b);
    }
}
