/** Information about a running process */
export interface ProcessInfo {
  pid: number;
  parentPid: number | null;
  name: string;
  cpuUsage: number;
  memoryBytes: number;
  virtualMemoryBytes: number;
  status: string;
  user: string | null;
  command: string;
  startTime: number;
  diskReadBytes: number;
  diskWriteBytes: number;
}

/** Summary of system processes */
export interface ProcessSummary {
  totalCount: number;
  runningCount: number;
  sleepingCount: number;
  totalCpuUsage: number;
  totalMemoryBytes: number;
}
