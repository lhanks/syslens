//! Shared system state management for efficient sysinfo operations
//!
//! This module provides a cached System instance that persists across Tauri IPC calls,
//! avoiding the expensive cost of creating new System objects for each request.

use sysinfo::{MemoryRefreshKind, ProcessRefreshKind, System, Users};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Cached system information state
pub struct SysInfoState {
    /// The shared System instance
    system: Mutex<System>,
    /// Cached users list
    users: Mutex<Users>,
    /// Last CPU refresh time (for accurate measurements, need interval between refreshes)
    last_cpu_refresh: Mutex<Option<Instant>>,
    /// Last process refresh time
    last_process_refresh: Mutex<Option<Instant>>,
}

impl SysInfoState {
    /// Create a new SysInfoState with an initialized System
    pub fn new() -> Self {
        // Create system with initial refresh
        let mut system = System::new();

        // Do initial refresh of all data we'll need
        system.refresh_cpu_all();
        system.refresh_memory_specifics(MemoryRefreshKind::everything());
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            ProcessRefreshKind::everything()
        );

        let users = Users::new_with_refreshed_list();

        Self {
            system: Mutex::new(system),
            users: Mutex::new(users),
            last_cpu_refresh: Mutex::new(Some(Instant::now())),
            last_process_refresh: Mutex::new(Some(Instant::now())),
        }
    }

    /// Get CPU metrics with proper refresh timing
    pub fn get_cpu_metrics(&self) -> (f32, Vec<f32>, u32) {
        let mut sys = self.system.lock().unwrap();
        let mut last_refresh = self.last_cpu_refresh.lock().unwrap();

        // Check if we need to wait for accurate measurement
        let needs_initial_wait = last_refresh.is_none()
            || last_refresh.unwrap().elapsed() > Duration::from_secs(5);

        if needs_initial_wait {
            // First refresh
            sys.refresh_cpu_all();
            // Need brief wait for accurate first measurement
            std::thread::sleep(Duration::from_millis(50));
        }

        // Refresh CPU data
        sys.refresh_cpu_all();
        *last_refresh = Some(Instant::now());

        let cpus = sys.cpus();
        let total_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;
        let per_core_usage: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
        let current_clock = cpus.first().map(|c| c.frequency() as u32).unwrap_or(0);

        (total_usage, per_core_usage, current_clock)
    }

    /// Get memory metrics
    pub fn get_memory_metrics(&self) -> (u64, u64, u64, u64) {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_memory_specifics(MemoryRefreshKind::everything());

        (
            sys.total_memory(),
            sys.used_memory(),
            sys.total_swap(),
            sys.used_swap(),
        )
    }

    /// Get process list with proper refresh timing
    pub fn with_processes<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&System, &Users, f32) -> R,
    {
        let mut sys = self.system.lock().unwrap();
        let users = self.users.lock().unwrap();
        let mut last_refresh = self.last_process_refresh.lock().unwrap();

        // Check if we need to wait for accurate CPU measurement
        let needs_initial_wait = last_refresh.is_none()
            || last_refresh.unwrap().elapsed() > Duration::from_secs(5);

        if needs_initial_wait {
            // First refresh for CPU baseline
            sys.refresh_cpu_all();
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::All,
                ProcessRefreshKind::everything()
            );
            // Brief wait for accurate CPU measurement
            std::thread::sleep(Duration::from_millis(50));
        }

        // Refresh processes
        sys.refresh_cpu_all();
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            ProcessRefreshKind::everything()
        );
        *last_refresh = Some(Instant::now());

        let cpu_count = sys.cpus().len() as f32;
        f(&sys, &users, cpu_count)
    }

    /// Get CPU count
    pub fn cpu_count(&self) -> usize {
        let sys = self.system.lock().unwrap();
        sys.cpus().len()
    }

    /// Get physical core count
    pub fn physical_core_count(&self) -> Option<usize> {
        let sys = self.system.lock().unwrap();
        sys.physical_core_count()
    }
}

impl Default for SysInfoState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sysinfo_state_creation() {
        let state = SysInfoState::new();
        assert!(state.cpu_count() > 0);
    }

    #[test]
    fn test_cpu_metrics() {
        let state = SysInfoState::new();
        let (total_usage, per_core_usage, _clock) = state.get_cpu_metrics();

        assert!(total_usage >= 0.0);
        assert!(!per_core_usage.is_empty());
        for usage in &per_core_usage {
            assert!(*usage >= 0.0);
        }
    }

    #[test]
    fn test_memory_metrics() {
        let state = SysInfoState::new();
        let (total, used, _total_swap, _used_swap) = state.get_memory_metrics();

        assert!(total > 0);
        assert!(used <= total);
    }

    #[test]
    fn test_process_listing() {
        let state = SysInfoState::new();

        let count = state.with_processes(|sys, _users, _cpu_count| {
            sys.processes().len()
        });

        assert!(count > 0, "Should have at least one process");
    }

    #[test]
    fn test_repeated_calls_efficiency() {
        let state = SysInfoState::new();

        // Multiple rapid calls should reuse the cached state
        for _ in 0..5 {
            let _ = state.get_cpu_metrics();
            let _ = state.get_memory_metrics();
        }

        // If we got here without hanging, the caching is working
    }
}
