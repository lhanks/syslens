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
      console.warn(`[TauriService] Not running in Tauri. Returning mock data for: ${command}`, args);
      return this.getMockData<T>(command, args);
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
  private getMockData<T>(command: string, args?: Record<string, unknown>): Observable<T> {
    // Handle device deep info with different data per device type
    if (command === 'get_device_deep_info' && args) {
      return of(this.getMockDeviceDeepInfo(args['deviceId'] as string, args['deviceType'] as string) as T);
    }

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
        formFactor: 'Desktop',
        biosVendor: 'American Megatrends Inc.',
        biosVersion: '2401',
        biosReleaseDate: '2024-01-15',
        bootMode: 'UEFI',
        secureBoot: true,
        tpmVersion: 'TPM 2.0',
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
        {
          deviceId: 0,
          model: 'Samsung 980 Pro 1TB',
          manufacturer: 'Samsung',
          serialNumber: 'S123456789',
          mediaType: 'NVMe',
          interfaceType: 'NVMe',
          sizeBytes: 1000204886016,
          partitionStyle: 'GPT',
          status: 'Healthy',
          firmware: 'FW1.0'
        },
        {
          deviceId: 1,
          model: 'WD Blue 2TB',
          manufacturer: 'Western Digital',
          serialNumber: 'WD987654321',
          mediaType: 'HDD',
          interfaceType: 'SATA',
          sizeBytes: 2000398934016,
          partitionStyle: 'GPT',
          status: 'OK',
          firmware: 'FW2.5'
        }
      ],
      get_partitions: [],
      get_volumes: [
        {
          driveLetter: 'C',
          label: 'Windows',
          fileSystem: 'NTFS',
          totalBytes: 500107862016,
          freeBytes: 250053931008,
          usedBytes: 250053931008,
          percentUsed: 50.0,
          volumeSerial: 'ABC123',
          isCompressed: false,
          isEncrypted: false,
          isSystem: true,
          isBoot: true
        },
        {
          driveLetter: 'D',
          label: 'Data',
          fileSystem: 'NTFS',
          totalBytes: 2000398934016,
          freeBytes: 1500298450512,
          usedBytes: 500100483504,
          percentUsed: 25.0,
          volumeSerial: 'DEF456',
          isCompressed: false,
          isEncrypted: false,
          isSystem: false,
          isBoot: false
        }
      ],
      get_disk_health: {
        deviceId: 0,
        status: 'Good',
        temperatureCelsius: 42,
        powerOnHours: 1250,
        powerCycles: 150,
        wearLevelPercent: 2,
        smartAttributes: []
      },
      get_disk_performance: [
        {
          deviceId: 0,
          readBytesPerSec: 52428800,
          writeBytesPerSec: 26214400,
          readIops: 1000,
          writeIops: 500,
          activeTimePercent: 15.5
        }
      ],
      get_network_drives: [],

      // System
      get_restore_points: [
        {
          sequenceNumber: 1,
          description: 'Windows Update',
          restorePointType: 'WindowsUpdate',
          creationTime: '2025-12-28 10:30:00'
        },
        {
          sequenceNumber: 2,
          description: 'Installed NVIDIA Graphics Driver',
          restorePointType: 'DeviceDriverInstall',
          creationTime: '2025-12-25 14:15:00'
        },
        {
          sequenceNumber: 3,
          description: 'Installed Visual Studio 2022',
          restorePointType: 'ApplicationInstall',
          creationTime: '2025-12-20 09:45:00'
        },
        {
          sequenceNumber: 4,
          description: 'Manual Checkpoint',
          restorePointType: 'ManualCheckpoint',
          creationTime: '2025-12-15 16:00:00'
        }
      ],

      // Device Deep Info - handled dynamically in getMockDeviceDeepInfo()
    };

    const data = mockData[command];
    if (data !== undefined) {
      return of(data as T);
    }
    console.warn(`[TauriService] No mock data for command: ${command}`);
    return of(null as T);
  }

  /**
   * Get mock device deep info based on device type.
   */
  private getMockDeviceDeepInfo(deviceId: string, deviceType: string): unknown {
    const baseMetadata = {
      source: 'AiAgent',
      lastUpdated: '2024-12-28T00:00:00.000Z',
      expiresAt: '2025-01-04T00:00:00.000Z',
      aiConfidence: 0.85
    };

    switch (deviceType) {
      case 'Cpu':
        return {
          deviceId,
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
            productPage: 'https://ark.intel.com/content/www/us/en/ark/products/230496/intel-core-i913900k-processor.html',
            supportPage: 'https://www.intel.com/support',
            manuals: [],
            datasheets: [],
            firmwareUpdates: []
          },
          images: { primaryImage: null, gallery: [] },
          metadata: { ...baseMetadata, sourceUrl: 'https://www.intel.com' }
        };

      case 'Gpu':
        return {
          deviceId,
          deviceType: 'Gpu',
          identifier: {
            manufacturer: 'NVIDIA',
            model: deviceId || 'GeForce RTX 4080',
            partNumber: 'PG141-SKU330',
            serialNumber: null,
            hardwareIds: []
          },
          specifications: {
            specs: {
              'CUDA Cores': '9728',
              'Base Clock': '2205 MHz',
              'Boost Clock': '2505 MHz',
              'Memory': '16 GB GDDR6X',
              'Memory Bus': '256-bit',
              'TDP': '320W'
            },
            categories: [],
            description: 'NVIDIA GeForce RTX 4080 graphics card',
            releaseDate: '2022-11-16'
          },
          drivers: {
            installedVersion: '555.42',
            latestVersion: '560.70',
            downloadUrl: 'https://www.nvidia.com/Download/index.aspx',
            driverPageUrl: 'https://www.nvidia.com/drivers',
            releaseDate: '2024-12-01',
            updateAvailable: true
          },
          documentation: {
            productPage: 'https://www.nvidia.com/en-us/geforce/graphics-cards/40-series/rtx-4080/',
            supportPage: 'https://www.nvidia.com/support',
            manuals: [],
            datasheets: [],
            firmwareUpdates: []
          },
          images: { primaryImage: null, gallery: [] },
          metadata: { ...baseMetadata, sourceUrl: 'https://www.nvidia.com' }
        };

      case 'Storage':
        return {
          deviceId,
          deviceType: 'Storage',
          identifier: {
            manufacturer: 'Samsung',
            model: deviceId || 'Samsung 980 Pro',
            partNumber: 'MZ-V8P1T0B/AM',
            serialNumber: null,
            hardwareIds: []
          },
          specifications: {
            specs: {
              'Capacity': '1 TB',
              'Interface': 'PCIe 4.0 x4 NVMe',
              'Form Factor': 'M.2 2280',
              'Sequential Read': '7,000 MB/s',
              'Sequential Write': '5,000 MB/s',
              'NAND Type': 'V-NAND 3-bit MLC',
              'Controller': 'Samsung Elpis'
            },
            categories: [],
            description: 'Samsung 980 PRO NVMe SSD',
            releaseDate: '2020-09-22'
          },
          drivers: {
            installedVersion: '3.0',
            latestVersion: '4.0',
            downloadUrl: 'https://www.samsung.com/semiconductor/minisite/ssd/download/tools/',
            driverPageUrl: 'https://www.samsung.com/support',
            releaseDate: '2024-06-15',
            updateAvailable: true
          },
          documentation: {
            productPage: 'https://www.samsung.com/us/computing/memory-storage/solid-state-drives/980-pro-pcie-4-0-nvme-ssd-1tb-mz-v8p1t0b-am/',
            supportPage: 'https://www.samsung.com/support',
            manuals: [{ title: 'User Manual', url: 'https://www.samsung.com/manual', fileType: 'PDF' }],
            datasheets: [{ title: 'Datasheet', url: 'https://www.samsung.com/datasheet', fileType: 'PDF' }],
            firmwareUpdates: []
          },
          images: { primaryImage: null, gallery: [] },
          metadata: { ...baseMetadata, sourceUrl: 'https://www.samsung.com' }
        };

      case 'Memory':
        return {
          deviceId,
          deviceType: 'Memory',
          identifier: {
            manufacturer: 'G.Skill',
            model: deviceId || 'Trident Z5 RGB',
            partNumber: 'F5-5600U4040A16G',
            serialNumber: null,
            hardwareIds: []
          },
          specifications: {
            specs: {
              'Capacity': '16 GB per module',
              'Type': 'DDR5',
              'Speed': '5600 MHz',
              'CAS Latency': 'CL40',
              'Voltage': '1.1V',
              'Heat Spreader': 'Yes'
            },
            categories: [],
            description: 'G.Skill Trident Z5 RGB DDR5 Memory',
            releaseDate: '2022-03-01'
          },
          drivers: null,
          documentation: {
            productPage: 'https://www.gskill.com/product/165/326/1604306952/F5-5600U4040A16G',
            supportPage: 'https://www.gskill.com/support',
            manuals: [],
            datasheets: [],
            firmwareUpdates: []
          },
          images: { primaryImage: null, gallery: [] },
          metadata: { ...baseMetadata, sourceUrl: 'https://www.gskill.com' }
        };

      case 'Motherboard':
        return {
          deviceId,
          deviceType: 'Motherboard',
          identifier: {
            manufacturer: 'ASUS',
            model: deviceId || 'ROG STRIX Z790-E GAMING WIFI',
            partNumber: '90MB1CL0-M0EAY0',
            serialNumber: null,
            hardwareIds: []
          },
          specifications: {
            specs: {
              'Socket': 'LGA 1700',
              'Chipset': 'Intel Z790',
              'Form Factor': 'ATX',
              'Memory Slots': '4 x DDR5',
              'Max Memory': '128 GB',
              'PCIe Slots': '2 x PCIe 5.0 x16',
              'M.2 Slots': '5'
            },
            categories: [],
            description: 'ASUS ROG STRIX Z790-E GAMING WIFI Motherboard',
            releaseDate: '2022-10-20'
          },
          drivers: {
            installedVersion: 'BIOS 2401',
            latestVersion: 'BIOS 2601',
            downloadUrl: 'https://www.asus.com/support/Download-Center/',
            driverPageUrl: 'https://www.asus.com/support/',
            releaseDate: '2024-08-01',
            updateAvailable: true
          },
          documentation: {
            productPage: 'https://www.asus.com/motherboards-components/motherboards/rog-strix/rog-strix-z790-e-gaming-wifi/',
            supportPage: 'https://www.asus.com/support/',
            manuals: [{ title: 'User Manual', url: 'https://www.asus.com/manual', fileType: 'PDF' }],
            datasheets: [],
            firmwareUpdates: [{ title: 'BIOS Update 2601', version: '2601', url: 'https://www.asus.com/bios', releaseDate: '2024-08-01' }]
          },
          images: { primaryImage: null, gallery: [] },
          metadata: { ...baseMetadata, sourceUrl: 'https://www.asus.com' }
        };

      default:
        return {
          deviceId,
          deviceType,
          identifier: {
            manufacturer: 'Unknown',
            model: deviceId || 'Unknown Device',
            partNumber: null,
            serialNumber: null,
            hardwareIds: []
          },
          specifications: null,
          drivers: null,
          documentation: null,
          images: { primaryImage: null, gallery: [] },
          metadata: baseMetadata
        };
    }
  }
}
