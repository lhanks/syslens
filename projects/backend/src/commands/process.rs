//! Process-related Tauri commands

use crate::models::{ProcessInfo, ProcessSummary, SelfMetrics};
use crate::services::ICON_CACHE;
use crate::state::SysInfoState;
use sysinfo::{Pid, Process, ProcessStatus, System, Users};
use tauri::State;

/// Get list of all running processes using shared state for efficiency
#[tauri::command]
pub fn get_processes(state: State<SysInfoState>) -> Vec<ProcessInfo> {
    log::debug!("Command: get_processes (optimized)");

    let processes: Vec<ProcessInfo> = state.with_processes(|sys, users, cpu_count| {
        sys.processes()
            .iter()
            .map(|(pid, process)| process_to_info(*pid, process, users, cpu_count))
            .collect()
    });

    // Log icon extraction stats (info level so visible in release builds)
    let with_icons = processes.iter().filter(|p| p.icon_base64.is_some()).count();
    let with_exe_path = processes.iter().filter(|p| p.exe_path.is_some()).count();
    log::info!(
        "Process list: {} total, {} with exe_path, {} with icons",
        processes.len(),
        with_exe_path,
        with_icons
    );

    // Log icon cache stats
    let (cache_total, cache_hits) = ICON_CACHE.stats();
    log::info!(
        "Icon cache: {} entries, {} with icons",
        cache_total,
        cache_hits
    );

    processes
}

/// Get process summary statistics using shared state for efficiency
#[tauri::command]
pub fn get_process_summary(state: State<SysInfoState>) -> ProcessSummary {
    log::debug!("Command: get_process_summary (optimized)");

    state.with_processes(|sys, _users, cpu_count| {
        let processes: Vec<&Process> = sys.processes().values().collect();

        let running_count = processes
            .iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Run))
            .count();

        let sleeping_count = processes
            .iter()
            .filter(|p| matches!(p.status(), ProcessStatus::Sleep))
            .count();

        // Normalize CPU usage by dividing by core count
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
    })
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

    // Normalize CPU usage by dividing by core count
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

/// Get metrics for the current Syslens process itself
#[tauri::command]
pub fn get_self_metrics(state: State<SysInfoState>) -> SelfMetrics {
    let current_pid = std::process::id();
    log::debug!("Command: get_self_metrics (pid={})", current_pid);

    state.with_processes(|sys, _users, cpu_count| {
        if let Some(process) = sys.process(Pid::from_u32(current_pid)) {
            // Normalize CPU usage by dividing by core count
            let normalized_cpu = process.cpu_usage() / cpu_count.max(1.0);

            SelfMetrics {
                pid: current_pid,
                cpu_usage: normalized_cpu,
                memory_bytes: process.memory(),
                virtual_memory_bytes: process.virtual_memory(),
            }
        } else {
            // Fallback if process not found (shouldn't happen)
            SelfMetrics {
                pid: current_pid,
                cpu_usage: 0.0,
                memory_bytes: 0,
                virtual_memory_bytes: 0,
            }
        }
    })
}

/// Kill a process by PID
/// Returns true if the process was successfully killed, false otherwise
#[tauri::command]
pub fn kill_process(pid: u32) -> Result<bool, String> {
    log::info!("Command: kill_process(pid={})", pid);

    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid)]));

    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        let name = process.name().to_string_lossy().to_string();
        let killed = process.kill();

        if killed {
            log::info!("Successfully killed process: {} (PID: {})", name, pid);
            Ok(true)
        } else {
            log::warn!("Failed to kill process: {} (PID: {})", name, pid);
            Err(format!(
                "Failed to kill process {} (PID: {}). Access may be denied.",
                name, pid
            ))
        }
    } else {
        log::warn!("Process not found: PID {}", pid);
        Err(format!("Process with PID {} not found", pid))
    }
}

#[cfg(test)]
mod tests {
    // Note: get_processes and get_process_summary tests moved to state module
    // since they now require Tauri State which cannot be easily instantiated in unit tests
    // The state module has comprehensive tests for process functionality
}
