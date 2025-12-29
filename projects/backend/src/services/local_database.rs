//! Local database manager for bundled device information.

use crate::models::{DeviceDeepInfo, DeviceIdentifier, DeviceType, LocalDatabase};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::RwLock;

/// Bundled device database JSON (embedded at compile time)
const BUNDLED_DATABASE: &str = include_str!("../../resources/device_database.json");

/// Manages the local device database with fuzzy matching.
pub struct LocalDatabaseManager {
    db_file: PathBuf,
    database: RwLock<LocalDatabase>,
}

impl LocalDatabaseManager {
    /// Create a new LocalDatabaseManager, loading or initializing the database.
    pub fn new() -> Result<Self> {
        let db_dir = Self::get_app_data_dir()?;
        std::fs::create_dir_all(&db_dir)
            .context("Failed to create database directory")?;

        let db_file = db_dir.join("device_database.json");

        // Initialize with bundled database if doesn't exist or is outdated
        let database = if db_file.exists() {
            Self::load_database_file(&db_file)?
        } else {
            let db = Self::parse_bundled_database()?;
            Self::save_database_file(&db_file, &db)?;
            db
        };

        Ok(Self {
            db_file,
            database: RwLock::new(database),
        })
    }

    /// Get the app data directory for Syslens.
    fn get_app_data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join("Syslens"))
            .context("Failed to get app data directory")
    }

    /// Parse the bundled database JSON.
    fn parse_bundled_database() -> Result<LocalDatabase> {
        serde_json::from_str(BUNDLED_DATABASE)
            .context("Failed to parse bundled device database")
    }

    /// Load database from disk.
    fn load_database_file(path: &PathBuf) -> Result<LocalDatabase> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read database file")?;
        serde_json::from_str(&content)
            .context("Failed to parse database file")
    }

    /// Save database to disk.
    fn save_database_file(path: &PathBuf, db: &LocalDatabase) -> Result<()> {
        let content = serde_json::to_string_pretty(db)
            .context("Failed to serialize database")?;
        std::fs::write(path, content)
            .context("Failed to write database file")
    }

    /// Save current database state to disk.
    fn save(&self) -> Result<()> {
        let db = self.database.read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire read lock"))?;
        Self::save_database_file(&self.db_file, &db)
    }

    /// Search for a device in the local database using fuzzy matching.
    pub fn find_device(
        &self,
        identifier: &DeviceIdentifier,
        device_type: &DeviceType,
    ) -> Option<DeviceDeepInfo> {
        let db = self.database.read().ok()?;
        let devices = db.devices.get_by_type(device_type);

        // Try exact match first
        if let Some(device) = devices.iter()
            .find(|d| Self::exact_match(&d.identifier, identifier))
        {
            return Some(device.clone());
        }

        // Try fuzzy match on model name
        devices.iter()
            .find(|d| Self::fuzzy_match(&d.identifier, identifier))
            .cloned()
    }

    /// Exact match on manufacturer and model (case-insensitive).
    fn exact_match(db_id: &DeviceIdentifier, search_id: &DeviceIdentifier) -> bool {
        db_id.manufacturer.eq_ignore_ascii_case(&search_id.manufacturer)
            && db_id.model.eq_ignore_ascii_case(&search_id.model)
    }

    /// Fuzzy match - checks if model names are similar.
    fn fuzzy_match(db_id: &DeviceIdentifier, search_id: &DeviceIdentifier) -> bool {
        // Must match manufacturer
        if !db_id.manufacturer.eq_ignore_ascii_case(&search_id.manufacturer) {
            return false;
        }

        // Normalize model names for comparison
        let db_model = Self::normalize_model(&db_id.model);
        let search_model = Self::normalize_model(&search_id.model);

        // Check if one contains the other
        db_model.contains(&search_model) || search_model.contains(&db_model)
    }

    /// Normalize model name for fuzzy matching.
    fn normalize_model(model: &str) -> String {
        model
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Add or update a device in the database.
    pub fn upsert_device(&self, device_info: DeviceDeepInfo) -> Result<()> {
        let mut db = self.database.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        let devices = db.devices.get_by_type_mut(&device_info.device_type);

        // Remove existing device with same ID
        devices.retain(|d| d.device_id != device_info.device_id);

        // Add new device
        devices.push(device_info);

        // Update timestamp
        db.last_updated = chrono::Utc::now();

        drop(db);
        self.save()
    }

    /// Get all devices of a specific type.
    pub fn get_all_by_type(&self, device_type: &DeviceType) -> Vec<DeviceDeepInfo> {
        let db = match self.database.read() {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        db.devices.get_by_type(device_type).clone()
    }

    /// Get database statistics.
    pub fn stats(&self) -> DatabaseStats {
        let db = match self.database.read() {
            Ok(d) => d,
            Err(_) => return DatabaseStats::default(),
        };

        DatabaseStats {
            version: db.version.clone(),
            last_updated: db.last_updated.to_rfc3339(),
            cpu_count: db.devices.cpu.len(),
            gpu_count: db.devices.gpu.len(),
            motherboard_count: db.devices.motherboard.len(),
            memory_count: db.devices.memory.len(),
            storage_count: db.devices.storage.len(),
        }
    }

    /// Reset database to bundled version.
    pub fn reset_to_bundled(&self) -> Result<()> {
        let bundled = Self::parse_bundled_database()?;

        let mut db = self.database.write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire write lock"))?;

        *db = bundled;

        drop(db);
        self.save()
    }
}

/// Database statistics
#[derive(Debug, Default)]
pub struct DatabaseStats {
    pub version: String,
    pub last_updated: String,
    pub cpu_count: usize,
    pub gpu_count: usize,
    pub motherboard_count: usize,
    pub memory_count: usize,
    pub storage_count: usize,
}
