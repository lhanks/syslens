//! Image cache service for storing and retrieving device images.
//!
//! Provides downloading, caching, and thumbnail generation for hardware device images.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Statistics for image cache operations.
#[derive(Debug, Default)]
pub struct ImageCacheStats {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub downloads: AtomicU64,
    pub download_failures: AtomicU64,
    pub total_bytes_cached: AtomicU64,
}

/// Image cache service for managing device images.
pub struct ImageCache {
    /// Cache directory path
    cache_dir: PathBuf,
    /// Maximum cache size in bytes (default 500MB)
    max_size_bytes: u64,
    /// HTTP client for downloading images
    client: reqwest::Client,
    /// In-memory tracking of cached files
    cached_files: RwLock<HashMap<String, CachedImageInfo>>,
    /// Cache statistics
    pub stats: ImageCacheStats,
}

/// Information about a cached image.
#[derive(Debug, Clone)]
pub struct CachedImageInfo {
    pub cache_key: String,
    pub file_path: PathBuf,
    pub original_url: String,
    pub file_size: u64,
    pub cached_at: SystemTime,
    pub thumbnail_path: Option<PathBuf>,
}

/// Result of a cache lookup or download operation.
#[derive(Debug, Clone)]
pub struct ImageCacheResult {
    pub cache_key: String,
    pub file_path: PathBuf,
    pub is_cached: bool,
    pub thumbnail_path: Option<PathBuf>,
}

impl ImageCache {
    /// Create a new ImageCache with default settings.
    pub fn new() -> Result<Self> {
        Self::with_max_size(500 * 1024 * 1024) // 500MB default
    }

    /// Create a new ImageCache with a custom max size.
    pub fn with_max_size(max_size_bytes: u64) -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        std::fs::create_dir_all(&cache_dir)
            .context("Failed to create image cache directory")?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Syslens/1.0 (Desktop System Monitor)")
            .build()
            .context("Failed to create HTTP client")?;

        let cache = Self {
            cache_dir,
            max_size_bytes,
            client,
            cached_files: RwLock::new(HashMap::new()),
            stats: ImageCacheStats::default(),
        };

        // Load existing cache index
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(cache.load_cache_index())
        })?;

        Ok(cache)
    }

    /// Get the cache directory path.
    fn get_cache_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .map(|p| p.join("Syslens").join("cache").join("images"))
            .context("Failed to get app data directory")
    }

    /// Generate a cache key from URL or device identifier.
    pub fn generate_cache_key(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)[..16].to_string()
    }

    /// Generate a cache key from device type and identifier.
    pub fn generate_device_cache_key(
        device_type: &str,
        manufacturer: &str,
        model: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(device_type.to_lowercase().as_bytes());
        hasher.update(b"|");
        hasher.update(manufacturer.to_lowercase().as_bytes());
        hasher.update(b"|");
        hasher.update(model.to_lowercase().as_bytes());
        let hash = hasher.finalize();
        format!("{:x}", hash)[..16].to_string()
    }

    /// Check if an image is already cached.
    pub async fn is_cached(&self, cache_key: &str) -> bool {
        let files = self.cached_files.read().await;
        if let Some(info) = files.get(cache_key) {
            // Also verify file still exists
            info.file_path.exists()
        } else {
            false
        }
    }

    /// Get the cached path for a cache key, if it exists.
    pub async fn get_cached_path(&self, cache_key: &str) -> Option<PathBuf> {
        let files = self.cached_files.read().await;
        files.get(cache_key).and_then(|info| {
            if info.file_path.exists() {
                self.stats.hits.fetch_add(1, Ordering::Relaxed);
                Some(info.file_path.clone())
            } else {
                self.stats.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        })
    }

    /// Fetch and cache an image from a URL.
    ///
    /// Returns the local file path to the cached image.
    pub async fn fetch_and_cache(&self, url: &str) -> Result<ImageCacheResult> {
        let cache_key = Self::generate_cache_key(url);

        // Check if already cached
        if let Some(path) = self.get_cached_path(&cache_key).await {
            let files = self.cached_files.read().await;
            let thumbnail_path = files.get(&cache_key).and_then(|i| i.thumbnail_path.clone());
            return Ok(ImageCacheResult {
                cache_key,
                file_path: path,
                is_cached: true,
                thumbnail_path,
            });
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);

        // Download the image
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch image")?;

        if !response.status().is_success() {
            self.stats.download_failures.fetch_add(1, Ordering::Relaxed);
            anyhow::bail!("HTTP error: {}", response.status());
        }

        // Get content type to determine extension
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg");

        let extension = Self::mime_to_extension(content_type);
        let file_path = self.cache_dir.join(format!("{}.{}", cache_key, extension));

        // Download bytes
        let bytes = response.bytes().await.context("Failed to read image bytes")?;
        let file_size = bytes.len() as u64;

        // Validate it's actually an image
        Self::validate_image(&bytes)?;

        // Check cache size and cleanup if needed
        self.ensure_cache_space(file_size).await?;

        // Save to disk
        tokio::fs::write(&file_path, &bytes)
            .await
            .context("Failed to write image to cache")?;

        self.stats.downloads.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes_cached.fetch_add(file_size, Ordering::Relaxed);

        // Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&file_path, 128).await.ok();

        // Update cache index
        let info = CachedImageInfo {
            cache_key: cache_key.clone(),
            file_path: file_path.clone(),
            original_url: url.to_string(),
            file_size,
            cached_at: SystemTime::now(),
            thumbnail_path: thumbnail_path.clone(),
        };

        {
            let mut files = self.cached_files.write().await;
            files.insert(cache_key.clone(), info);
        }

        // Save cache index
        self.save_cache_index().await?;

        log::info!("Cached image: {} -> {:?}", url, file_path);

        Ok(ImageCacheResult {
            cache_key,
            file_path,
            is_cached: false,
            thumbnail_path,
        })
    }

    /// Fetch and cache an image with a custom cache key.
    pub async fn fetch_and_cache_with_key(
        &self,
        url: &str,
        cache_key: &str,
    ) -> Result<ImageCacheResult> {
        // Check if already cached
        if let Some(path) = self.get_cached_path(cache_key).await {
            let files = self.cached_files.read().await;
            let thumbnail_path = files.get(cache_key).and_then(|i| i.thumbnail_path.clone());
            return Ok(ImageCacheResult {
                cache_key: cache_key.to_string(),
                file_path: path,
                is_cached: true,
                thumbnail_path,
            });
        }

        self.stats.misses.fetch_add(1, Ordering::Relaxed);

        // Download the image
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch image")?;

        if !response.status().is_success() {
            self.stats.download_failures.fetch_add(1, Ordering::Relaxed);
            anyhow::bail!("HTTP error: {}", response.status());
        }

        // Get content type to determine extension
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg");

        let extension = Self::mime_to_extension(content_type);
        let file_path = self.cache_dir.join(format!("{}.{}", cache_key, extension));

        // Download bytes
        let bytes = response.bytes().await.context("Failed to read image bytes")?;
        let file_size = bytes.len() as u64;

        // Validate it's actually an image
        Self::validate_image(&bytes)?;

        // Check cache size and cleanup if needed
        self.ensure_cache_space(file_size).await?;

        // Save to disk
        tokio::fs::write(&file_path, &bytes)
            .await
            .context("Failed to write image to cache")?;

        self.stats.downloads.fetch_add(1, Ordering::Relaxed);
        self.stats.total_bytes_cached.fetch_add(file_size, Ordering::Relaxed);

        // Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&file_path, 128).await.ok();

        // Update cache index
        let info = CachedImageInfo {
            cache_key: cache_key.to_string(),
            file_path: file_path.clone(),
            original_url: url.to_string(),
            file_size,
            cached_at: SystemTime::now(),
            thumbnail_path: thumbnail_path.clone(),
        };

        {
            let mut files = self.cached_files.write().await;
            files.insert(cache_key.to_string(), info);
        }

        // Save cache index
        self.save_cache_index().await?;

        log::info!("Cached image with key {}: {} -> {:?}", cache_key, url, file_path);

        Ok(ImageCacheResult {
            cache_key: cache_key.to_string(),
            file_path,
            is_cached: false,
            thumbnail_path,
        })
    }

    /// Generate a thumbnail from a cached image.
    pub async fn generate_thumbnail(&self, image_path: &Path, size: u32) -> Result<PathBuf> {
        let image_path = image_path.to_path_buf();
        let cache_dir = self.cache_dir.clone();

        // Run image processing in blocking task
        tokio::task::spawn_blocking(move || {
            let img = image::open(&image_path)
                .context("Failed to open image for thumbnail")?;

            let thumb = img.thumbnail(size, size);

            let file_stem = image_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let thumb_path = cache_dir.join(format!("{}_thumb.png", file_stem));
            thumb.save(&thumb_path)
                .context("Failed to save thumbnail")?;

            Ok(thumb_path)
        })
        .await
        .context("Thumbnail task panicked")?
    }

    /// Validate that bytes represent a valid image.
    fn validate_image(bytes: &[u8]) -> Result<()> {
        // Check for common image magic bytes
        if bytes.len() < 8 {
            anyhow::bail!("Image too small to be valid");
        }

        // PNG: 89 50 4E 47 0D 0A 1A 0A
        let is_png = bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

        // JPEG: FF D8 FF
        let is_jpeg = bytes.starts_with(&[0xFF, 0xD8, 0xFF]);

        // GIF: GIF87a or GIF89a
        let is_gif = bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a");

        // WebP: RIFF....WEBP
        let is_webp = bytes.len() >= 12
            && bytes.starts_with(b"RIFF")
            && &bytes[8..12] == b"WEBP";

        if !is_png && !is_jpeg && !is_gif && !is_webp {
            anyhow::bail!("Invalid or unsupported image format");
        }

        // Size limit: 10MB
        if bytes.len() > 10 * 1024 * 1024 {
            anyhow::bail!("Image exceeds maximum size of 10MB");
        }

        Ok(())
    }

    /// Convert MIME type to file extension.
    fn mime_to_extension(mime: &str) -> &'static str {
        match mime {
            "image/png" => "png",
            "image/jpeg" | "image/jpg" => "jpg",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "jpg", // Default to jpg
        }
    }

    /// Ensure there's enough space in the cache.
    async fn ensure_cache_space(&self, needed_bytes: u64) -> Result<()> {
        let current_size: u64 = {
            let files = self.cached_files.read().await;
            files.values().map(|f| f.file_size).sum()
        };

        if current_size + needed_bytes <= self.max_size_bytes {
            return Ok(());
        }

        // Need to clean up - remove oldest files first
        let to_free = (current_size + needed_bytes).saturating_sub(self.max_size_bytes);
        self.cleanup_oldest(to_free).await
    }

    /// Remove oldest cached files to free up space.
    async fn cleanup_oldest(&self, bytes_to_free: u64) -> Result<()> {
        let mut files = self.cached_files.write().await;

        // Sort by cached_at (oldest first)
        let mut entries: Vec<_> = files.values().cloned().collect();
        entries.sort_by_key(|e| e.cached_at);

        let mut freed = 0u64;
        let mut to_remove = Vec::new();

        for entry in entries {
            if freed >= bytes_to_free {
                break;
            }

            to_remove.push(entry.cache_key.clone());
            freed += entry.file_size;

            // Delete file
            if let Err(e) = std::fs::remove_file(&entry.file_path) {
                log::warn!("Failed to remove cached image: {}", e);
            }

            // Delete thumbnail if exists
            if let Some(thumb) = &entry.thumbnail_path {
                let _ = std::fs::remove_file(thumb);
            }
        }

        for key in to_remove {
            files.remove(&key);
        }

        log::info!("Cache cleanup: freed {} bytes", freed);
        Ok(())
    }

    /// Clean up images older than the specified duration.
    pub async fn cleanup_older_than(&self, max_age: Duration) -> Result<usize> {
        let cutoff = SystemTime::now() - max_age;
        let mut removed = 0;

        let mut files = self.cached_files.write().await;
        let keys_to_remove: Vec<_> = files
            .iter()
            .filter(|(_, info)| info.cached_at < cutoff)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            if let Some(info) = files.remove(&key) {
                if let Err(e) = std::fs::remove_file(&info.file_path) {
                    log::warn!("Failed to remove old cached image: {}", e);
                } else {
                    removed += 1;
                }

                if let Some(thumb) = &info.thumbnail_path {
                    let _ = std::fs::remove_file(thumb);
                }
            }
        }

        drop(files);
        self.save_cache_index().await?;

        log::info!("Cleaned up {} old cached images", removed);
        Ok(removed)
    }

    /// Load cache index from disk.
    async fn load_cache_index(&self) -> Result<()> {
        let index_path = self.cache_dir.join("cache_index.json");

        if !index_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&index_path)
            .await
            .context("Failed to read cache index")?;

        let entries: Vec<CachedImageInfoSerializable> =
            serde_json::from_str(&content).context("Failed to parse cache index")?;

        let mut files = self.cached_files.write().await;
        for entry in entries {
            // Only add if file still exists
            if entry.file_path.exists() {
                files.insert(entry.cache_key.clone(), entry.into());
            }
        }

        Ok(())
    }

    /// Save cache index to disk.
    async fn save_cache_index(&self) -> Result<()> {
        let files = self.cached_files.read().await;
        let entries: Vec<CachedImageInfoSerializable> =
            files.values().cloned().map(|i| i.into()).collect();

        let content = serde_json::to_string_pretty(&entries)
            .context("Failed to serialize cache index")?;

        let index_path = self.cache_dir.join("cache_index.json");
        tokio::fs::write(&index_path, content)
            .await
            .context("Failed to write cache index")?;

        Ok(())
    }

    /// Get cache statistics.
    pub fn get_stats(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.stats.hits.load(Ordering::Relaxed),
            self.stats.misses.load(Ordering::Relaxed),
            self.stats.downloads.load(Ordering::Relaxed),
            self.stats.download_failures.load(Ordering::Relaxed),
            self.stats.total_bytes_cached.load(Ordering::Relaxed),
        )
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get total cached file count.
    pub async fn cached_count(&self) -> usize {
        self.cached_files.read().await.len()
    }

    /// Get total cache size in bytes.
    pub async fn total_size(&self) -> u64 {
        self.cached_files
            .read()
            .await
            .values()
            .map(|f| f.file_size)
            .sum()
    }
}

/// Serializable version of CachedImageInfo for JSON persistence.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedImageInfoSerializable {
    cache_key: String,
    file_path: PathBuf,
    original_url: String,
    file_size: u64,
    cached_at_secs: u64,
    thumbnail_path: Option<PathBuf>,
}

impl From<CachedImageInfo> for CachedImageInfoSerializable {
    fn from(info: CachedImageInfo) -> Self {
        Self {
            cache_key: info.cache_key,
            file_path: info.file_path,
            original_url: info.original_url,
            file_size: info.file_size,
            cached_at_secs: info
                .cached_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            thumbnail_path: info.thumbnail_path,
        }
    }
}

impl From<CachedImageInfoSerializable> for CachedImageInfo {
    fn from(s: CachedImageInfoSerializable) -> Self {
        Self {
            cache_key: s.cache_key,
            file_path: s.file_path,
            original_url: s.original_url,
            file_size: s.file_size,
            cached_at: SystemTime::UNIX_EPOCH + Duration::from_secs(s.cached_at_secs),
            thumbnail_path: s.thumbnail_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_cache_key() {
        let key1 = ImageCache::generate_cache_key("https://example.com/image.jpg");
        let key2 = ImageCache::generate_cache_key("https://example.com/image.jpg");
        let key3 = ImageCache::generate_cache_key("https://example.com/other.jpg");

        assert_eq!(key1.len(), 16);
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_generate_device_cache_key() {
        let key1 = ImageCache::generate_device_cache_key("GPU", "NVIDIA", "RTX 4090");
        let key2 = ImageCache::generate_device_cache_key("gpu", "nvidia", "rtx 4090");
        let key3 = ImageCache::generate_device_cache_key("GPU", "AMD", "RX 7900 XTX");

        // Case insensitive
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_mime_to_extension() {
        assert_eq!(ImageCache::mime_to_extension("image/png"), "png");
        assert_eq!(ImageCache::mime_to_extension("image/jpeg"), "jpg");
        assert_eq!(ImageCache::mime_to_extension("image/gif"), "gif");
        assert_eq!(ImageCache::mime_to_extension("image/webp"), "webp");
        assert_eq!(ImageCache::mime_to_extension("application/octet-stream"), "jpg");
    }

    #[test]
    fn test_validate_image_png() {
        let png_magic = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
        assert!(ImageCache::validate_image(&png_magic).is_ok());
    }

    #[test]
    fn test_validate_image_jpeg() {
        let jpeg_magic = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0];
        assert!(ImageCache::validate_image(&jpeg_magic).is_ok());
    }

    #[test]
    fn test_validate_image_invalid() {
        let invalid = vec![0x00, 0x00, 0x00, 0x00, 0, 0, 0, 0];
        assert!(ImageCache::validate_image(&invalid).is_err());
    }

    #[test]
    fn test_validate_image_too_small() {
        let small = vec![0x89, 0x50];
        assert!(ImageCache::validate_image(&small).is_err());
    }
}
