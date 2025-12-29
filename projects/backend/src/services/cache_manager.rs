//! Cache manager for device information with AppData storage.

use crate::models::{CacheEntry, DeviceCache, DeviceDeepInfo, DeviceType};
use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use std::path::PathBuf;
use std::sync::RwLock;

/// Manages device information cache in AppData directory.
pub struct CacheManager {
    cache_file: PathBuf,
    cache: RwLock<DeviceCache>,
}

impl CacheManager {
    /// Create a new CacheManager, loading existing cache from disk.
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_app_data_dir()?;
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create cache directory")?;

        let cache_file = cache_dir.join("device_cache.json");
        let cache = Self::load_cache_file(&cache_file).unwrap_or_default();

        Ok(Self {
            cache_file,
            cache: RwLock::new(cache),
        })
    }

    /// Get the app data directory for Syslens.
    fn get_app_data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join("Syslens"))
            .context("Failed to get app data directory")
    }

    /// Load cache from disk.
    fn load_cache_file(path: &PathBuf) -> Result<DeviceCache> {
        if !path.exists() {
            return Ok(DeviceCache::default());
        }

        let content = std::fs::read_to_string(path)
            .context("Failed to read cache file")?;
        let cache = serde_json::from_str(&content)
            .context("Failed to parse cache file")?;
        Ok(cache)
    }

    /// Save cache to disk.
    fn save_cache(&self) -> Result<()> {
        let cache = self.cache.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        let content = serde_json::to_string_pretty(&*cache)
            .context("Failed to serialize cache")?;
        std::fs::write(&self.cache_file, content)
            .context("Failed to write cache file")?;
        Ok(())
    }

    /// Get cached device info if not expired.
    pub fn get(&self, device_id: &str, device_type: &DeviceType) -> Option<DeviceDeepInfo> {
        let cache = self.cache.read().ok()?;
        let now = Utc::now();

        cache.entries.iter()
            .find(|e| e.device_id == device_id && &e.device_type == device_type)
            .filter(|e| e.expires_at > now)
            .map(|e| e.data.clone())
    }

    /// Store device info in cache with TTL.
    pub fn set(
        &self,
        device_id: String,
        device_type: DeviceType,
        mut data: DeviceDeepInfo,
        ttl_days: i64,
    ) -> Result<()> {
        let now = Utc::now();
        let expires_at = now + Duration::days(ttl_days);

        // Update metadata to reflect caching
        data.metadata.last_updated = now;
        data.metadata.expires_at = expires_at;

        let mut cache = self.cache.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        // Remove existing entry for this device
        cache.entries.retain(|e|
            !(e.device_id == device_id && e.device_type == device_type)
        );

        // Add new entry
        cache.entries.push(CacheEntry {
            device_id,
            device_type,
            data,
            cached_at: now,
            expires_at,
        });

        drop(cache);
        self.save_cache()
    }

    /// Remove a specific device from cache.
    pub fn remove(&self, device_id: &str, device_type: &DeviceType) -> Result<()> {
        let mut cache = self.cache.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        cache.entries.retain(|e|
            !(e.device_id == device_id && &e.device_type == device_type)
        );

        drop(cache);
        self.save_cache()
    }

    /// Clear all cache entries.
    pub fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        cache.entries.clear();

        drop(cache);
        self.save_cache()
    }

    /// Remove all expired entries.
    pub fn cleanup_expired(&self) -> Result<usize> {
        let now = Utc::now();
        let mut cache = self.cache.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        let before_count = cache.entries.len();
        cache.entries.retain(|e| e.expires_at > now);
        let removed = before_count - cache.entries.len();

        drop(cache);
        if removed > 0 {
            self.save_cache()?;
        }
        Ok(removed)
    }

    /// Get all cached devices (for offline viewing).
    pub fn get_all(&self) -> Vec<DeviceDeepInfo> {
        let cache = match self.cache.read() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        cache.entries.iter()
            .map(|e| e.data.clone())
            .collect()
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let cache = match self.cache.read() {
            Ok(c) => c,
            Err(_) => return CacheStats::default(),
        };

        let now = Utc::now();
        let total = cache.entries.len();
        let expired = cache.entries.iter()
            .filter(|e| e.expires_at <= now)
            .count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            valid_entries: total - expired,
        }
    }
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DataMetadata, DataSource, DeviceIdentifier};

    fn create_test_device_info(device_id: &str) -> DeviceDeepInfo {
        DeviceDeepInfo {
            device_id: device_id.to_string(),
            device_type: DeviceType::Cpu,
            identifier: DeviceIdentifier {
                manufacturer: "Intel".to_string(),
                model: "Core i7-12700K".to_string(),
                part_number: Some("BX8071512700K".to_string()),
                serial_number: None,
                hardware_ids: vec![],
            },
            specifications: None,
            drivers: None,
            documentation: None,
            images: None,
            metadata: DataMetadata {
                source: DataSource::LocalDatabase,
                last_updated: Utc::now(),
                expires_at: Utc::now() + Duration::days(7),
                source_url: None,
                ai_confidence: None,
            },
        }
    }

    #[test]
    fn test_cache_set_and_get() {
        let cache = CacheManager::new().unwrap();
        let device_info = create_test_device_info("test-cpu-1");

        cache.set(
            "test-cpu-1".to_string(),
            DeviceType::Cpu,
            device_info.clone(),
            7,
        ).unwrap();

        let retrieved = cache.get("test-cpu-1", &DeviceType::Cpu);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().device_id, "test-cpu-1");

        // Cleanup
        cache.remove("test-cpu-1", &DeviceType::Cpu).unwrap();
    }

    #[test]
    fn test_cache_miss() {
        let cache = CacheManager::new().unwrap();
        let result = cache.get("nonexistent", &DeviceType::Cpu);
        assert!(result.is_none());
    }
}
