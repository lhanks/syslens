//! Services for Syslens
//!
//! Business logic and data management services.

pub mod ai_agent;
pub mod cache_manager;
pub mod device_sources;
pub mod internet_fetcher;
pub mod knowledge_store;
pub mod local_database;

pub use ai_agent::AiAgent;
pub use cache_manager::CacheManager;
pub use device_sources::{DeviceSource, SourceResult};
pub use internet_fetcher::InternetFetcher;
pub use knowledge_store::{KnowledgeStore, PartialDeviceInfo};
pub use local_database::LocalDatabaseManager;
