//! Process information collector

use sysinfo::{System, Process, ProcessStatus, Users};
use crate::models::process::{ProcessInfo, ProcessSummary};

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

        let running_count = processes.iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Run))
            .count();

        let sleeping_count = processes.iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Sleep))
            .count();

        // Normalize CPU usage by dividing by core count (sysinfo reports per-core %)
        let total_cpu_usage: f32 = processes.iter()
            .map(|p| p.cpu_usage())
            .sum::<f32>() / cpu_count.max(1.0);

        let total_memory_bytes: u64 = processes.iter()
            .map(|p| p.memory())
            .sum();

        ProcessSummary {
            total_count: processes.len(),
            running_count,
            sleeping_count,
            total_cpu_usage,
            total_memory_bytes,
        }
    }

    fn process_to_info(pid: sysinfo::Pid, process: &Process, users: &Users, cpu_count: f32) -> ProcessInfo {
        let user = process.user_id().and_then(|uid| {
            users.get_user_by_id(uid).map(|u| u.name().to_string())
        });

        let disk_usage = process.disk_usage();

        // Normalize CPU usage by dividing by core count (sysinfo reports per-core %)
        let normalized_cpu = process.cpu_usage() / cpu_count.max(1.0);

        ProcessInfo {
            pid: pid.as_u32(),
            parent_pid: process.parent().map(|p| p.as_u32()),
            name: process.name().to_string_lossy().to_string(),
            cpu_usage: normalized_cpu,
            memory_bytes: process.memory(),
            virtual_memory_bytes: process.virtual_memory(),
            status: format!("{:?}", process.status()),
            user,
            command: process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" "),
            start_time: process.start_time(),
            disk_read_bytes: disk_usage.read_bytes,
            disk_write_bytes: disk_usage.written_bytes,
        }
    }
}
