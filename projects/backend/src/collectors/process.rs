//! Process information collector

use crate::models::process::{ProcessInfo, ProcessSummary};
use crate::services::ICON_CACHE;
use sysinfo::{Process, ProcessStatus, System, Users};

/// Collector for process information
pub struct ProcessCollector;

impl ProcessCollector {
    /// Get list of all running processes
    pub fn get_processes() -> Vec<ProcessInfo> {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        // Need a second refresh for accurate CPU usage
        std::thread::sleep(std::time::Duration::from_millis(50));
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let users = Users::new_with_refreshed_list();
        let cpu_count = sys.cpus().len() as f32;

        sys.processes()
            .iter()
            .map(|(pid, process)| Self::process_to_info(*pid, process, &users, cpu_count))
            .collect()
    }

    /// Get process summary statistics
    pub fn get_process_summary() -> ProcessSummary {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

        let processes: Vec<&Process> = sys.processes().values().collect();
        let cpu_count = sys.cpus().len() as f32;

        let running_count = processes
            .iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Run))
            .count();

        let sleeping_count = processes
            .iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Sleep))
            .count();

        // Normalize CPU usage by dividing by core count (sysinfo reports per-core %)
        let total_cpu_usage: f32 =
            processes.iter().map(|p| p.cpu_usage()).sum::<f32>() / cpu_count.max(1.0);

        let total_memory_bytes: u64 = processes.iter().map(|p| p.memory()).sum();

        ProcessSummary {
            total_count: processes.len(),
            running_count,
            sleeping_count,
            total_cpu_usage,
            total_memory_bytes,
        }
    }

    fn process_to_info(
        pid: sysinfo::Pid,
        process: &Process,
        users: &Users,
        cpu_count: f32,
    ) -> ProcessInfo {
        let user = process
            .user_id()
            .and_then(|uid| users.get_user_by_id(uid).map(|u| u.name().to_string()));

        let disk_usage = process.disk_usage();

        // Normalize CPU usage by dividing by core count (sysinfo reports per-core %)
        let normalized_cpu = process.cpu_usage() / cpu_count.max(1.0);

        // Get executable path and icon
        let exe_path = process.exe().map(|p| p.to_string_lossy().to_string());
        let icon_base64 =
            ICON_CACHE.get_icon_for_process(&process.name().to_string_lossy(), exe_path.as_deref());

        ProcessInfo {
            pid: pid.as_u32(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            name: process.name().to_string_lossy().to_string(),
            cpu_usage: normalized_cpu,
            memory_bytes: process.memory(),
            virtual_memory_bytes: process.virtual_memory(),
            status: format!("{:?}", process.status()),
            user,
            command: process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" "),
            start_time: process.start_time(),
            disk_read_bytes: disk_usage.read_bytes,
            disk_write_bytes: disk_usage.written_bytes,
            exe_path,
            icon_base64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_processes() {
        let processes = ProcessCollector::get_processes();
        // System should have at least some processes
        assert!(!processes.is_empty(), "Should have at least one process");

        // All processes should have valid data
        for process in &processes {
            // PID 0 is valid on Windows (System Idle Process)
            assert!(!process.name.is_empty(), "Process should have a name");
            assert!(process.cpu_usage >= 0.0, "CPU usage should be non-negative");
        }
    }

    #[test]
    fn test_get_process_summary() {
        let summary = ProcessCollector::get_process_summary();

        // Should have at least one process
        assert!(summary.total_count > 0, "Should have at least one process");

        // Running + sleeping should not exceed total
        assert!(
            summary.running_count + summary.sleeping_count <= summary.total_count,
            "Running + sleeping should not exceed total"
        );

        // CPU usage should be within valid range (0-100%)
        assert!(
            summary.total_cpu_usage >= 0.0,
            "CPU usage should be non-negative"
        );
        assert!(
            summary.total_cpu_usage <= 100.0,
            "CPU usage should not exceed 100%"
        );

        // Memory should be positive
        assert!(
            summary.total_memory_bytes > 0,
            "Total memory should be positive"
        );
    }

    #[test]
    fn test_process_info_serialization() {
        let process_info = ProcessInfo {
            pid: 1234,
            parent_pid: Some(1000),
            name: "test_process".to_string(),
            cpu_usage: 5.5,
            memory_bytes: 1024 * 1024 * 100,         // 100 MB
            virtual_memory_bytes: 1024 * 1024 * 500, // 500 MB
            status: "Run".to_string(),
            user: Some("testuser".to_string()),
            command: "test_process --flag".to_string(),
            start_time: 1704067200, // 2024-01-01 00:00:00 UTC
            disk_read_bytes: 1024 * 1024,
            disk_write_bytes: 512 * 1024,
            exe_path: Some("C:\\test\\test_process.exe".to_string()),
            icon_base64: None,
        };

        let json = serde_json::to_string(&process_info).unwrap();
        assert!(json.contains("\"pid\":1234"));
        assert!(json.contains("\"parentPid\":1000"));
        assert!(json.contains("\"name\":\"test_process\""));
        assert!(json.contains("\"cpuUsage\":5.5"));
        assert!(json.contains("\"status\":\"Run\""));
        assert!(json.contains("\"exePath\":"));
    }

    #[test]
    fn test_process_summary_serialization() {
        let summary = ProcessSummary {
            total_count: 100,
            running_count: 10,
            sleeping_count: 85,
            total_cpu_usage: 25.5,
            total_memory_bytes: 1024 * 1024 * 1024 * 8, // 8 GB
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalCount\":100"));
        assert!(json.contains("\"runningCount\":10"));
        assert!(json.contains("\"sleepingCount\":85"));
        assert!(json.contains("\"totalCpuUsage\":25.5"));
    }
}
