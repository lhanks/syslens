# Changelog: Task 0 Implementation

**Date:** 2025-12-28
**Tasks:** 3.7, 3.8, 4.10, 4.11, 4.12

---

## Summary

This update implements 5 new enhancement tasks for improved device information, network data, numeric formatting, navigation, and progressive data loading.

---

## Backend Changes (Rust)

### Task 3.7: Enhanced Device Information Collection

**Files Modified:**
- `projects/backend/src/collectors/system.rs` (+500 lines)
- `projects/backend/Cargo.toml` (added `wmi` and `windows` crates)

**Implementation Details:**

Added Windows Management Instrumentation (WMI) support for comprehensive device information:

```rust
// WMI query structures added:
- Win32ComputerSystem (manufacturer, model, system_type, system_sku)
- Win32BIOS (manufacturer, serial_number, smbios_version, release_date)
- Win32BaseBoard (manufacturer, product, serial_number, version)
- Win32OperatingSystem (caption, version, build_number, install_date)
- Win32TPM (spec_version, is_activated, is_enabled)
```

**New Data Available:**
| Field | Source |
|-------|--------|
| Manufacturer | WMI Win32_ComputerSystem |
| Model | WMI Win32_ComputerSystem |
| Serial Number | WMI Win32_BIOS |
| System SKU | WMI Win32_ComputerSystem.SystemSKUNumber |
| Product ID | Windows Registry |
| BIOS Version | WMI Win32_BIOS.SMBIOSBIOSVersion |
| BIOS Release Date | WMI Win32_BIOS.ReleaseDate |
| TPM Info | WMI Win32_TPM |

**Fallback Strategy:**
- Primary: WMI queries
- Fallback: Windows Registry
- Last resort: sysinfo crate / hardcoded defaults

---

### Task 3.8: Complete IP Configuration Data (ipconfig /all)

**Files Modified:**
- `projects/backend/src/collectors/network.rs` (+600 lines)

**Implementation Details:**

Added comprehensive IP configuration data matching `ipconfig /all` output:

```rust
// New structures added:
- IpConfigurationFull
- DnsConfiguration
- DhcpConfiguration
- AdapterDetailedInfo
```

**New Data Available:**
| Field | Description |
|-------|-------------|
| Connection-specific DNS Suffix | Per-adapter DNS suffix |
| Description | Adapter description |
| Physical Address (MAC) | Hardware address |
| DHCP Enabled | Whether DHCP is active |
| Autoconfiguration Enabled | IPv4 autoconfig status |
| IPv4 Address | Primary IPv4 with subnet mask |
| Subnet Mask | Network mask |
| Default Gateway | Gateway address(es) |
| DHCP Server | DHCP server address |
| DNS Servers | All configured DNS servers |
| Lease Obtained/Expires | DHCP lease times |
| IPv6 Address | Link-local and global IPv6 |

---

## Frontend Changes (Angular)

### Task 4.10: Numeric Value Formatting (1 Decimal Place)

**Files Created:**
- `projects/ui/src/app/shared/pipes/decimal.pipe.ts`

**Files Modified:**
- `projects/ui/src/app/shared/pipes/bytes.pipe.ts`
- `projects/ui/src/app/shared/pipes/index.ts`
- `projects/ui/src/app/features/hardware/hardware.component.ts`
- `projects/ui/src/app/features/storage/storage.component.ts`
- `projects/ui/src/app/shared/components/progress-ring/progress-ring.component.ts`

**Implementation Details:**

```typescript
// New DecimalPipe
@Pipe({ name: 'decimal', standalone: true })
export class DecimalPipe implements PipeTransform {
  transform(value: number | null | undefined, precision = 1): string {
    if (value === null || value === undefined || isNaN(value)) {
      return '0';
    }
    const fixed = value.toFixed(precision);
    // Remove trailing zeros for cleaner display
    const trimmed = fixed.replace(/(\.\d*?)0+$/, '$1').replace(/\.$/, '');
    return trimmed || '0';
  }
}
```

**Usage:**
```html
{{ cpuUsage | decimal }}      <!-- "45.3" -->
{{ memoryPercent | decimal:2 }} <!-- "67.89" -->
```

**BytesPipe Enhancement:**
- Now uses consistent 1 decimal place formatting
- Cleaner output: "1.5 GB" instead of "1.536743164 GB"

---

### Task 4.11: Click-Through to Detailed Views

**Files Modified:**
- `projects/ui/src/app/features/dashboard/dashboard.component.ts`
- Component templates (dashboard, hardware, storage)

**Implementation Details:**

Added navigation from summary cards to detailed views:

```typescript
// Dashboard component
navigateToDetail(section: string): void {
  this.router.navigate(['/' + section]);
}
```

```html
<!-- Summary card with click handler -->
<div class="stat-card" (click)="navigateToDetail('hardware')">
  <h3>CPU Usage</h3>
  <span>{{ cpuPercent | decimal }}%</span>
</div>
```

---

### Task 4.12: Progressive Data Loading

**Files Created:**
- `projects/ui/src/app/core/services/preload.service.ts`

**Files Modified:**
- `projects/ui/src/app/app.component.ts`
- `projects/ui/src/app/core/services/index.ts`
- `projects/ui/src/app/shared/components/sidebar/sidebar.component.ts`

**Implementation Details:**

```typescript
@Injectable({ providedIn: 'root' })
export class PreloadService {
  // Tracks which views have been preloaded
  private preloadedViews = new Set<string>();

  // Cancellation subject for pending preloads
  private preloadCancel$ = new Subject<void>();
}
```

**Loading Strategy:**

1. **Current Tab First:** When user navigates, current view data loads immediately
2. **Background Preload:** After 500ms delay, other views start preloading
3. **Staggered Loading:** Each view preloads 200ms after the previous
4. **Priority System:** Related views preload first (e.g., from Dashboard → Hardware → Storage)
5. **Cancellation:** If user clicks another tab, pending preloads cancel immediately
6. **Hover Preload:** Hovering over nav items triggers priority preload

**View Priority Map:**
```typescript
const priority = {
  'dashboard': ['hardware', 'storage', 'network', 'system'],
  'hardware':  ['dashboard', 'system', 'storage', 'network'],
  'storage':   ['dashboard', 'hardware', 'system', 'network'],
  'network':   ['dashboard', 'system', 'hardware', 'storage'],
  'system':    ['dashboard', 'hardware', 'network', 'storage']
};
```

**Sidebar Hover Enhancement:**
```typescript
// sidebar.component.ts
onNavHover(route: string): void {
  this.preloadService.priorityPreload(route);
}
```

---

## Dependencies Added

### Backend (Cargo.toml)
```toml
wmi = "0.13"      # WMI queries for Windows system info
windows = { version = "0.52", features = ["Win32_System_Registry"] }
```

---

## Testing Notes

1. **Device Info:** Verify all fields populate on Windows (WMI-dependent)
2. **Network Info:** Compare output with `ipconfig /all`
3. **Decimal Formatting:** Check all numeric displays show 1 decimal max
4. **Click-Through:** Test navigation from dashboard cards
5. **Preloading:** Use DevTools Network tab to verify background requests

---

## Breaking Changes

None. All changes are additive and backward-compatible.
