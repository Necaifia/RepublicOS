pub mod store;

use std::collections::HashMap;
use pantheon_kernel::cell_id::CellId;
use pantheon_kernel::error::Result;
use serde::{Deserialize, Serialize};

pub use store::MemoryStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub cell_id: CellId,
    pub timestamp_ms: u64,
    pub ttl_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub entries: usize,
    pub size_bytes: usize,
    pub cells_using: usize,
}

pub struct MemoryManager {
    working: HashMap<String, MemoryEntry>,
    pub store: MemoryStore,
    ephemeral_keys: Vec<String>,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            working: HashMap::new(),
            store: MemoryStore::new(),
            ephemeral_keys: Vec::new(),
        }
    }

    pub fn put(&mut self, key: &str, value: Vec<u8>, cell: CellId) -> Result<()> {
        let entry = MemoryEntry {
            key: key.to_string(),
            value,
            cell_id: cell,
            timestamp_ms: now_ms(),
            ttl_ms: None,
        };
        self.working.insert(key.to_string(), entry);
        Ok(())
    }

    pub fn put_ephemeral(&mut self, key: &str, value: Vec<u8>, cell: CellId, ttl_ms: u64) -> Result<()> {
        let entry = MemoryEntry {
            key: key.to_string(),
            value,
            cell_id: cell,
            timestamp_ms: now_ms(),
            ttl_ms: Some(ttl_ms),
        };
        self.ephemeral_keys.push(key.to_string());
        self.working.insert(key.to_string(), entry);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&MemoryEntry> {
        let entry = self.working.get(key)?;
        // Check TTL
        if let Some(ttl) = entry.ttl_ms {
            if now_ms().saturating_sub(entry.timestamp_ms) > ttl {
                return None;
            }
        }
        Some(entry)
    }

    pub fn remove(&mut self, key: &str) -> Option<MemoryEntry> {
        self.working.remove(key)
    }

    pub fn persist(&mut self, key: &str) -> Result<()> {
        if let Some(entry) = self.working.get(key) {
            self.store.write(key, &bincode::serialize(entry).map_err(|e| {
                pantheon_kernel::error::Error::Custom(e.to_string())
            })?)?;
        }
        Ok(())
    }

    pub fn load(&mut self, key: &str) -> Result<Option<MemoryEntry>> {
        if let Some(data) = self.store.read(key)? {
            let entry: MemoryEntry = bincode::deserialize(&data).map_err(|e| {
                pantheon_kernel::error::Error::Custom(e.to_string())
            })?;
            self.working.insert(key.to_string(), entry.clone());
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub fn clear_expired(&mut self) {
        let now = now_ms();
        self.ephemeral_keys.retain(|k| {
            if let Some(entry) = self.working.get(k) {
                if let Some(ttl) = entry.ttl_ms {
                    if now.saturating_sub(entry.timestamp_ms) > ttl {
                        self.working.remove(k);
                        return false;
                    }
                }
                true
            } else {
                false
            }
        });
    }

    pub fn entries_for_cell(&self, cell: &CellId) -> Vec<&MemoryEntry> {
        self.working.values().filter(|e| &e.cell_id == cell).collect()
    }

    pub fn stats(&self) -> MemoryStats {
        let size = self.working.values().fold(0, |acc, e| acc + e.value.len());
        let mut cells = std::collections::HashSet::new();
        for entry in self.working.values() {
            cells.insert(entry.cell_id);
        }
        MemoryStats {
            entries: self.working.len(),
            size_bytes: size,
            cells_using: cells.len(),
        }
    }

    pub fn snapshot(&self) -> HashMap<String, MemoryEntry> {
        self.working.clone()
    }

    pub fn restore(&mut self, snap: HashMap<String, MemoryEntry>) {
        self.working = snap;
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use pantheon_kernel::cell_id::CellId;

    #[test]
    fn test_put_and_get() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("test-key", vec![1, 2, 3], cell).unwrap();
        let entry = mem.get("test-key").unwrap();
        assert_eq!(entry.value, vec![1, 2, 3]);
        assert_eq!(entry.cell_id, cell);
    }

    #[test]
    fn test_get_nonexistent() {
        let mem = MemoryManager::new();
        assert!(mem.get("nonexistent").is_none());
    }

    #[test]
    fn test_remove() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("key", vec![1], cell).unwrap();
        assert!(mem.remove("key").is_some());
        assert!(mem.get("key").is_none());
    }

    #[test]
    fn test_ephemeral_expiry() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        // TTL of 1ms — should expire almost immediately
        mem.put_ephemeral("temp", vec![42], cell, 1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        mem.clear_expired();
        assert!(mem.get("temp").is_none());
    }

    #[test]
    fn test_entries_for_cell() {
        let cell_a = CellId::random();
        let cell_b = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("a1", vec![1], cell_a).unwrap();
        mem.put("a2", vec![2], cell_a).unwrap();
        mem.put("b1", vec![3], cell_b).unwrap();

        assert_eq!(mem.entries_for_cell(&cell_a).len(), 2);
        assert_eq!(mem.entries_for_cell(&cell_b).len(), 1);
    }

    #[test]
    fn test_stats() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("k1", vec![1, 2, 3], cell).unwrap();
        mem.put("k2", vec![4, 5], cell).unwrap();

        let stats = mem.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.size_bytes, 5);
        assert_eq!(stats.cells_using, 1);
    }

    #[test]
    fn test_snapshot_restore() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("k1", vec![1, 2], cell).unwrap();
        mem.put("k2", vec![3, 4], cell).unwrap();

        let snap = mem.snapshot();
        assert_eq!(snap.len(), 2);

        let mut mem2 = MemoryManager::new();
        mem2.restore(snap);
        assert_eq!(mem2.stats().entries, 2);
    }

    #[test]
    fn test_persist_and_load() {
        let dir = std::env::temp_dir().join("pantheon-test-mem-persist");
        let _ = std::fs::remove_dir_all(&dir);

        let cell = CellId::random();
        {
            let mut mem = MemoryManager::new();
            mem.store = MemoryStore::with_path(&dir);
            mem.put("persist-key", vec![99, 100], cell).unwrap();
            mem.persist("persist-key").unwrap();
        }

        {
            let mut mem2 = MemoryManager::new();
            mem2.store = MemoryStore::with_path(&dir);
            let loaded = mem2.load("persist-key").unwrap();
            assert!(loaded.is_some());
            assert_eq!(loaded.unwrap().value, vec![99, 100]);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clear_expired_does_not_remove_permanent() {
        let cell = CellId::random();
        let mut mem = MemoryManager::new();
        mem.put("perm", vec![1], cell).unwrap();
        mem.clear_expired();
        assert!(mem.get("perm").is_some());
    }
}
