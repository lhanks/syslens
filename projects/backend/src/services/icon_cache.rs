//! Icon caching and extraction service for process icons.

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use log::{debug, trace};

#[cfg(windows)]
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
#[cfg(windows)]
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
};
#[cfg(windows)]
use windows::core::PCWSTR;

/// Cache for extracted process icons.
/// Maps executable path -> base64 PNG data.
pub struct IconCache {
    cache: RwLock<HashMap<String, Option<String>>>,
    /// Counter for successful extractions (for debugging)
    success_count: AtomicUsize,
    /// Counter for failed extractions (for debugging)
    fail_count: AtomicUsize,
}

impl IconCache {
    /// Create a new icon cache.
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            success_count: AtomicUsize::new(0),
            fail_count: AtomicUsize::new(0),
        }
    }

    /// Get icon for an executable path.
    /// Returns cached icon if available, otherwise extracts and caches it.
    pub fn get_icon(&self, exe_path: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.cache.read().ok()?.get(exe_path) {
            trace!("Icon cache hit for: {}", exe_path);
            return cached.clone();
        }

        // Extract icon and cache it
        let icon = self.extract_icon(exe_path);

        if icon.is_some() {
            self.success_count.fetch_add(1, Ordering::Relaxed);
            trace!("Icon extracted successfully for: {}", exe_path);
        } else {
            self.fail_count.fetch_add(1, Ordering::Relaxed);
            trace!("Icon extraction failed for: {}", exe_path);
        }

        if let Ok(mut cache) = self.cache.write() {
            cache.insert(exe_path.to_string(), icon.clone());
        }

        // Log stats periodically
        let total = self.success_count.load(Ordering::Relaxed) + self.fail_count.load(Ordering::Relaxed);
        if total > 0 && total.is_multiple_of(50) {
            debug!("Icon cache stats: {} successes, {} failures",
                   self.success_count.load(Ordering::Relaxed),
                   self.fail_count.load(Ordering::Relaxed));
        }

        icon
    }

    /// Extract icon from an executable file.
    #[cfg(windows)]
    fn extract_icon(&self, exe_path: &str) -> Option<String> {
        use std::mem::MaybeUninit;

        let path = Path::new(exe_path);
        if !path.exists() {
            trace!("Icon extract: file does not exist: {}", exe_path);
            return None;
        }

        // Convert path to wide string
        let wide_path: Vec<u16> = exe_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mut file_info: MaybeUninit<SHFILEINFOW> = MaybeUninit::uninit();

            let result = SHGetFileInfoW(
                PCWSTR::from_raw(wide_path.as_ptr()),
                windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
                Some(file_info.as_mut_ptr()),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_SMALLICON,
            );

            if result == 0 {
                trace!("Icon extract: SHGetFileInfoW returned 0 for: {}", exe_path);
                return None;
            }

            let file_info = file_info.assume_init();
            let hicon = file_info.hIcon;

            if hicon.is_invalid() {
                trace!("Icon extract: Invalid hIcon for: {}", exe_path);
                return None;
            }

            // Get icon bitmap info
            let mut icon_info: MaybeUninit<ICONINFO> = MaybeUninit::uninit();
            if GetIconInfo(hicon, icon_info.as_mut_ptr()).is_err() {
                trace!("Icon extract: GetIconInfo failed for: {}", exe_path);
                let _ = DestroyIcon(hicon);
                return None;
            }

            let icon_info = icon_info.assume_init();

            // Extract pixel data from the icon
            let png_data = self.icon_to_png(icon_info);

            // Clean up
            if !icon_info.hbmColor.is_invalid() {
                let _ = DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_invalid() {
                let _ = DeleteObject(icon_info.hbmMask);
            }
            let _ = DestroyIcon(hicon);

            if png_data.is_none() {
                trace!("Icon extract: PNG conversion failed for: {}", exe_path);
            }

            png_data.map(|data| STANDARD.encode(&data))
        }
    }

    #[cfg(windows)]
    fn icon_to_png(&self, icon_info: ICONINFO) -> Option<Vec<u8>> {
        use windows::Win32::Graphics::Gdi::{GetObjectW, BITMAP};

        unsafe {
            // Check if color bitmap is valid
            if icon_info.hbmColor.is_invalid() {
                trace!("icon_to_png: hbmColor is invalid (monochrome icon?)");
                return None;
            }

            // Get actual bitmap dimensions
            let mut bm: BITMAP = std::mem::zeroed();
            let bm_size = GetObjectW(
                icon_info.hbmColor,
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bm as *mut _ as *mut _),
            );

            if bm_size == 0 {
                trace!("icon_to_png: GetObjectW failed to get bitmap info");
                return None;
            }

            let width = bm.bmWidth;
            let height = bm.bmHeight.abs(); // Height can be negative
            trace!("icon_to_png: Bitmap dimensions: {}x{}", width, height);

            // Use 16x16 as target size for consistency
            let target_size = 16i32;

            let hdc = CreateCompatibleDC(None);
            if hdc.is_invalid() {
                trace!("icon_to_png: CreateCompatibleDC failed");
                return None;
            }

            // Get bitmap info - use actual dimensions
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // Negative for top-down
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [Default::default(); 1],
            };

            // Select the color bitmap
            let old_bitmap = SelectObject(hdc, icon_info.hbmColor);
            if old_bitmap.is_invalid() {
                trace!("icon_to_png: SelectObject failed");
                let _ = DeleteDC(hdc);
                return None;
            }

            // Allocate buffer for pixel data
            let pixel_count = (width * height) as usize;
            let mut pixels: Vec<u8> = vec![0u8; pixel_count * 4];

            let result = GetDIBits(
                hdc,
                icon_info.hbmColor,
                0,
                height as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Restore old bitmap
            let _ = SelectObject(hdc, old_bitmap);
            let _ = DeleteDC(hdc);

            if result == 0 {
                trace!("icon_to_png: GetDIBits returned 0");
                return None;
            }

            trace!("icon_to_png: GetDIBits returned {} scan lines", result);

            // Convert BGRA to RGBA
            for chunk in pixels.chunks_exact_mut(4) {
                chunk.swap(0, 2); // Swap B and R
            }

            // Create image with actual dimensions
            let img = match image::RgbaImage::from_raw(width as u32, height as u32, pixels) {
                Some(img) => img,
                None => {
                    trace!("icon_to_png: Failed to create RgbaImage from raw data");
                    return None;
                }
            };

            // Resize to 16x16 if needed for consistency
            let img = if width != target_size || height != target_size {
                trace!("icon_to_png: Resizing from {}x{} to {}x{}", width, height, target_size, target_size);
                image::imageops::resize(&img, target_size as u32, target_size as u32, image::imageops::FilterType::Lanczos3)
            } else {
                img
            };

            // Encode as PNG
            let mut png_data = Vec::new();
            let mut cursor = std::io::Cursor::new(&mut png_data);
            if let Err(e) = img.write_to(&mut cursor, image::ImageFormat::Png) {
                trace!("icon_to_png: PNG encoding failed: {}", e);
                return None;
            }

            trace!("icon_to_png: Successfully created PNG ({} bytes)", png_data.len());
            Some(png_data)
        }
    }

    #[cfg(not(windows))]
    fn extract_icon(&self, _exe_path: &str) -> Option<String> {
        None
    }

    /// Get icon for a process by name.
    /// This is a simpler method that just returns a default icon
    /// when no executable path is available.
    pub fn get_icon_for_process(&self, name: &str, exe_path: Option<&str>) -> Option<String> {
        if let Some(path) = exe_path {
            if !path.is_empty() {
                return self.get_icon(path);
            }
        }

        // Return cached icon by process name if we've seen it before
        if let Some(cached) = self.cache.read().ok()?.get(name) {
            return cached.clone();
        }

        None
    }

    /// Clear the cache.
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache statistics.
    pub fn stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.read() {
            let total = cache.len();
            let with_icons = cache.values().filter(|v| v.is_some()).count();
            (total, with_icons)
        } else {
            (0, 0)
        }
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    /// Global icon cache instance.
    pub static ref ICON_CACHE: IconCache = IconCache::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_cache_creation() {
        let cache = IconCache::new();
        assert_eq!(cache.stats(), (0, 0));
    }

    #[test]
    fn test_icon_cache_caching() {
        let cache = IconCache::new();

        // First call should attempt extraction and cache result
        let _ = cache.get_icon("nonexistent.exe");

        // Second call should use cache
        let _ = cache.get_icon("nonexistent.exe");

        let (total, _) = cache.stats();
        assert_eq!(total, 1);
    }

    #[test]
    fn test_icon_cache_clear() {
        let cache = IconCache::new();
        let _ = cache.get_icon("test.exe");
        cache.clear();
        assert_eq!(cache.stats(), (0, 0));
    }
}
