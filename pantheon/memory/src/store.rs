use std::collections::HashMap;
use std::path::{Path, PathBuf};
use pantheon_kernel::error::{Error, Result};

pub struct MemoryStore {
    base_path: Option<PathBuf>,
    cache: HashMap<String, Vec<u8>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            base_path: None,
            cache: HashMap::new(),
        }
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: Some(path.into()),
            cache: HashMap::new(),
        }
    }

    pub fn write(&mut self, key: &str, data: &[u8]) -> Result<()> {
        self.cache.insert(key.to_string(), data.to_vec());

        if let Some(ref base) = self.base_path {
            let file_path = self.key_to_path(base, key);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::Custom(format!("Failed to create store dir: {}", e)))?;
            }
            std::fs::write(&file_path, data)
                .map_err(|e| Error::Custom(format!("Failed to write store file: {}", e)))?;
        }
        Ok(())
    }

    pub fn read(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Check cache first
        if let Some(data) = self.cache.get(key) {
            return Ok(Some(data.clone()));
        }

        // Try disk if path is set
        if let Some(ref base) = self.base_path {
            let file_path = self.key_to_path(base, key);
            if file_path.exists() {
                let data = std::fs::read(&file_path)
                    .map_err(|e| Error::Custom(format!("Failed to read store file: {}", e)))?;
                return Ok(Some(data));
            }
        }
        Ok(None)
    }

    pub fn delete(&mut self, key: &str) -> Result<()> {
        self.cache.remove(key);
        if let Some(ref base) = self.base_path {
            let file_path = self.key_to_path(base, key);
            if file_path.exists() {
                std::fs::remove_file(&file_path)
                    .map_err(|e| Error::Custom(format!("Failed to delete store file: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn list_keys(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    fn key_to_path(&self, base: &Path, key: &str) -> PathBuf {
        let safe_name = key.replace('/', "_").replace('\\', "_").replace(':', "_");
        base.join(format!("{}.mem", safe_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read() {
        let mut store = MemoryStore::new();
        store.write("test-key", &[1, 2, 3, 4]).unwrap();
        let data = store.read("test-key").unwrap().unwrap();
        assert_eq!(data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_read_nonexistent() {
        let store = MemoryStore::new();
        assert!(store.read("nonexistent").unwrap().is_none());
    }

    #[test]
    fn test_delete() {
        let mut store = MemoryStore::new();
        store.write("del-key", &[1]).unwrap();
        assert!(store.read("del-key").unwrap().is_some());
        store.delete("del-key").unwrap();
        assert!(store.read("del-key").unwrap().is_none());
    }

    #[test]
    fn test_list_keys() {
        let mut store = MemoryStore::new();
        store.write("a", &[1]).unwrap();
        store.write("b", &[2]).unwrap();
        let keys = store.list_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn test_persistent_storage() {
        let dir = std::env::temp_dir().join("pantheon-test-store");
        let _ = std::fs::remove_dir_all(&dir);

        {
            let mut store = MemoryStore::with_path(&dir);
            store.write("persistent-key", &[42]).unwrap();
        }

        {
            let store = MemoryStore::with_path(&dir);
            let data = store.read("persistent-key").unwrap().unwrap();
            assert_eq!(data, vec![42]);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }
}
