# Syslens Project Instructions

## Project Overview

Syslens is a desktop system information dashboard built with Tauri (Rust backend) and Angular (frontend).

## Directory Structure

- `specs/` - Project specifications (read-only reference)
- `projects/ui/` - Angular frontend application
- `projects/backend/` - Rust/Tauri backend

## Development Commands

### Frontend (Angular)
```bash
cd projects/ui
npm start          # Start dev server on port 4200
npm run build      # Production build
npm test           # Run tests
```

### Backend (Rust)
```bash
cd projects/backend
cargo tauri dev    # Start Tauri in dev mode
cargo tauri build  # Production build
cargo test         # Run tests
```

## Code Conventions

### Angular (Frontend)
- Use standalone components
- Feature-based organization in `src/app/features/`
- Shared components in `src/app/shared/`
- Services in `src/app/core/services/`
- Use Angular signals for state management
- Tailwind CSS for styling

### Rust (Backend)
- Commands in `src/commands/` - one file per feature area
- Data collectors in `src/collectors/`
- Models in `src/models/` - use serde for serialization
- Follow Rust naming conventions (snake_case)

## Tauri IPC Pattern

Frontend calls backend:
```typescript
import { invoke } from '@tauri-apps/api/core';

const data = await invoke('get_cpu_info');
```

Backend command:
```rust
#[tauri::command]
pub fn get_cpu_info() -> CpuInfo {
    // Implementation
}
```

## Key Dependencies

### Frontend
- Angular 21
- Tailwind CSS
- @tauri-apps/api

### Backend
- Tauri 2.0
- sysinfo (system information)
- serde (serialization)
- windows crate (Windows API)

## Testing

- Frontend: Jasmine/Karma for unit tests
- Backend: Rust's built-in test framework

## Build Outputs

- Frontend: `projects/ui/dist/`
- Backend: `projects/backend/target/`
- Final app: `projects/backend/target/release/`
