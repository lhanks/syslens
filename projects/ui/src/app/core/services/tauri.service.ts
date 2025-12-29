import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { Observable, from, Subject, shareReplay, of, throwError, catchError } from 'rxjs';

/**
 * Base service for Tauri IPC communication.
 * Provides methods to invoke Tauri commands and listen to events.
 * Falls back to mock data when running in browser (not Tauri window).
 */
@Injectable({
  providedIn: 'root'
})
export class TauriService {
  private eventListeners = new Map<string, UnlistenFn>();

  /**
   * Check if running inside Tauri window.
   */
  get isTauri(): boolean {
    return !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  }

  /**
   * Invoke a Tauri command with optional arguments.
   * @param command - The command name to invoke
   * @param args - Optional arguments to pass to the command
   * @returns Observable of the command result
   */
  invoke<T>(command: string, args?: Record<string, unknown>): Observable<T> {
    if (!this.isTauri) {
      console.warn(`[TauriService] Not running in Tauri. Returning mock data for: ${command}`);
      return this.getMockData<T>(command);
    }
    return from(invoke<T>(command, args)).pipe(
      catchError(err => {
        console.error(`[TauriService] Command failed: ${command}`, err);
        return throwError(() => err);
      })
    );
  }

  /**
   * Listen to a Tauri event and return an Observable.
   * @param event - The event name to listen to
   * @returns Observable that emits event payloads
   */
  listen<T>(event: string): Observable<T> {
    const subject = new Subject<T>();

    listen<T>(event, (e) => {
      subject.next(e.payload);
    }).then((unlisten) => {
      // Store the unlisten function for cleanup
      const existing = this.eventListeners.get(event);
      if (existing) {
        existing();
      }
      this.eventListeners.set(event, unlisten);
    });

    return subject.asObservable().pipe(shareReplay(1));
  }

  /**
   * Stop listening to a specific event.
   * @param event - The event name to stop listening to
   */
  unlisten(event: string): void {
    const unlisten = this.eventListeners.get(event);
    if (unlisten) {
      unlisten();
      this.eventListeners.delete(event);
    }
  }

  /**
   * Clean up all event listeners.
   */
  cleanup(): void {
    this.eventListeners.forEach((unlisten) => unlisten());
    this.eventListeners.clear();
  }

  /**
   * Get mock data for browser development.
   */
  private getMockData<T>(command: string): Observable<T> {
    const mockData: Record<string, unknown> = {
      // Hardware
      get_cpu_info: {
        name: 'Mock CPU (Browser Mode)',
        manufacturer: 'Intel',
        architecture: 'x64',
        physicalCores: 8,
        logicalProcessors: 16,
        baseClockMhz: 3600,
        maxClockMhz: 5000,
        socket: 'LGA1700',
        cache: { l1DataKb: 48, l1InstructionKb: 32, l2Kb: 512, l3Kb: 16384 }
      },
      get_cpu_metrics: {
        totalUsage: 25.5,
        perCoreUsage: [20, 30, 15, 40, 25, 35, 10, 45, 22, 33, 18, 28, 24, 36, 12, 42],
        currentClockMhz: 4200,
        temperature: 55
      },
      get_memory_info: {
        totalBytes: 34359738368,
        usableBytes: 33489170432,
        memoryType: 'DDR5',
        speedMhz: 5600,
        slotsUsed: 2,
        slotsTotal: 4,
        modules: [
          { slot: 'DIMM 1', capacityBytes: 17179869184, speedMhz: 5600, manufacturer: 'G.Skill', partNumber: 'F5-5600U4040A16G' },
          { slot: 'DIMM 2', capacityBytes: 17179869184, speedMhz: 5600, manufacturer: 'G.Skill', partNumber: 'F5-5600U4040A16G' }
        ]
      },
      get_memory_metrics: {
        inUseBytes: 18253611008,
        availableBytes: 16106127360,
        committedBytes: 22548578304,
        cachedBytes: 8589934592,
        pagedPoolBytes: 536870912,
        nonPagedPoolBytes: 268435456
      },
      get_gpu_info: [
        {
          id: 'gpu-0',
          name: 'Mock GPU (Browser Mode)',
          manufacturer: 'NVIDIA',
          vramBytes: 12884901888,
          driverVersion: '555.42',
          driverDate: '2024-12-01',
          adapterType: 'Discrete',
          currentResolution: '2560x1440',
          refreshRateHz: 165,
          driverLink: 'https://www.nvidia.com/Download/index.aspx'
        }
      ],
      get_gpu_metrics: [
        { gpuId: 'gpu-0', usagePercent: 15.0, temperature: 42, vramUsedBytes: 2147483648 }
      ],
      get_motherboard_info: {
        manufacturer: 'ASUS',
        product: 'ROG STRIX Z790-E GAMING WIFI',
        version: 'Rev 1.xx',
        serialNumber: 'MB123456789',
        chipset: 'Intel Z790',
        biosVendor: 'American Megatrends Inc.',
        biosVersion: '2401',
        biosReleaseDate: '2024-01-15',
        supportUrl: 'https://www.asus.com/support/'
      },
      get_monitors: [
        { id: 'monitor-0', name: 'Mock Monitor', manufacturer: 'Dell', resolution: '2560x1440', refreshRateHz: 165, connection: 'DisplayPort', sizeInches: 27, hdrSupport: true }
      ],
      get_usb_devices: [],
      get_audio_devices: [],

      // System
      get_device_info: {
        deviceName: 'MOCK-PC',
        manufacturer: 'Mock Manufacturer',
        model: 'Mock Model',
        systemSku: 'SKU123',
        serialNumber: 'SN123456',
        productId: 'XXXXX-XXXXX-XXXXX-XXXXX'
      },
      get_os_info: {
        name: 'Windows 11 Pro',
        version: '23H2',
        buildNumber: '22631',
        architecture: 'x64',
        installDate: '2024-01-01T00:00:00Z'
      },
      get_bios_info: {
        vendor: 'American Megatrends Inc.',
        version: '2401',
        releaseDate: '2024-01-15'
      },
      get_uptime: { seconds: 86400 },
      get_boot_config: { uefiEnabled: true, secureBootEnabled: true },
      get_domain_info: { domain: 'WORKGROUP', role: 'Workstation' },
      get_user_info: { username: 'MockUser', domain: 'MOCK-PC' },

      // Network
      get_network_adapters: [
        { id: 'eth0', name: 'Ethernet', description: 'Mock Ethernet Adapter', adapterType: 'Ethernet', macAddress: '00:11:22:33:44:55', status: 'Up', speedMbps: 1000, mtu: 1500 }
      ],
      get_adapter_stats: [],
      get_active_connections: [],
      get_routing_table: [],

      // Storage
      get_physical_disks: [
        { id: 'disk0', name: 'Mock SSD', type: 'SSD', sizeBytes: 1000204886016, model: 'Samsung 980 Pro', serialNumber: 'S123456', interfaceType: 'NVMe' }
      ],
      get_partitions: [],
      get_volumes: [
        { letter: 'C', label: 'Windows', fileSystem: 'NTFS', sizeBytes: 500107862016, freeBytes: 250053931008 }
      ],
      get_disk_health: [],
      get_disk_performance: [],
      get_network_drives: [],

      // Device Deep Info
      get_device_deep_info: {
        deviceId: 'mock-device-001',
        deviceType: 'Cpu',
        identifier: {
          manufacturer: 'Intel',
          model: 'Core i9-13900K',
          partNumber: 'BX8071513900K',
          serialNumber: null,
          hardwareIds: ['BFEBFBFF000B0671']
        },
        specifications: {
          specs: {
            'Cores': '24 (8P + 16E)',
            'Threads': '32',
            'Base Clock': '3.0 GHz',
            'Max Turbo': '5.8 GHz',
            'TDP': '125W'
          },
          categories: [],
          description: 'Intel Core i9-13900K processor',
          releaseDate: '2022-10-20'
        },
        drivers: {
          installedVersion: '10.0.22621.0',
          latestVersion: '10.0.22621.0',
          downloadUrl: 'https://www.intel.com/download',
          releaseDate: '2024-03-15',
          updateAvailable: false
        },
        documentation: {
          productPage: 'https://www.intel.com/products',
          supportPage: 'https://www.intel.com/support',
          manuals: [],
          datasheets: [],
          firmwareUpdates: []
        },
        images: {
          primaryImage: 'https://www.intel.com/images/cpu.png',
          gallery: []
        },
        metadata: {
          source: 'AiAgent',
          lastUpdated: '2024-12-28T00:00:00.000Z',
          expiresAt: '2025-01-04T00:00:00.000Z',
          sourceUrl: 'https://www.intel.com',
          aiConfidence: 0.85
        }
      }
    };

    const data = mockData[command];
    if (data !== undefined) {
      return of(data as T);
    }
    console.warn(`[TauriService] No mock data for command: ${command}`);
    return of(null as T);
  }
}
