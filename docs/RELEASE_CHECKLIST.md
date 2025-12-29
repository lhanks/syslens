# Syslens Release Checklist

Pre-release checklist for Syslens desktop application.

## Version Information

- **Current Version:** 1.0.0
- **Target Platform:** Windows 10/11 (x64)
- **Tech Stack:** Tauri 2.0 + Angular 21 + Rust

---

## Pre-Release Checks

### 1. Code Quality

- [ ] All ESLint rules pass (`npm run lint` in projects/ui)
- [ ] All Clippy warnings resolved (`cargo clippy` in projects/backend)
- [ ] TypeScript compilation succeeds with no errors
- [ ] Rust compilation succeeds with no warnings

```bash
# Run all quality checks
cd projects/ui && npm run lint
cd projects/backend && cargo clippy -- -D warnings
```

### 2. Testing

- [ ] All Rust tests pass (62+ tests)
- [ ] All Angular tests pass (47+ tests)
- [ ] Manual testing of all features completed

```bash
# Run all tests
cd projects/backend && cargo test
cd projects/ui && npm test
```

### 3. Feature Verification

#### Dashboard
- [ ] CPU usage displays correctly
- [ ] Memory usage displays correctly
- [ ] Disk usage displays correctly
- [ ] Network speeds update in real-time
- [ ] Network traffic graph renders

#### System Tab
- [ ] Device information displays
- [ ] BIOS/UEFI information displays
- [ ] OS information displays

#### Hardware Tab
- [ ] CPU details with cache sizes
- [ ] Memory modules with XMP speeds
- [ ] GPU information with driver links
- [ ] Motherboard details
- [ ] Monitor/display information
- [ ] Device detail modals open and display data

#### Network Tab
- [ ] All adapters listed
- [ ] IP configuration shows for each adapter
- [ ] Traffic graphs per adapter
- [ ] Connection statistics

#### Storage Tab
- [ ] All drives detected
- [ ] Partition information displays
- [ ] S.M.A.R.T. data (if available)
- [ ] Health status indicators

#### Processes Tab
- [ ] Process list loads
- [ ] Sorting works (CPU, Memory, Name)
- [ ] Search/filter works
- [ ] Pagination works
- [ ] Process detail modal opens

### 4. Performance

- [ ] App starts within 3 seconds
- [ ] Dashboard loads within 2 seconds
- [ ] Real-time updates at 1-second intervals
- [ ] No memory leaks during extended use
- [ ] Smooth graph animations

### 5. Build Verification

```bash
# Production build
./bin/build-ui
./bin/build-backend

# Or full build
cargo tauri build
```

- [ ] NSIS installer builds successfully
- [ ] MSI installer builds successfully
- [ ] Installer size is reasonable (<50MB)
- [ ] App icon displays correctly in installer

### 6. Installation Testing

- [ ] Fresh install on clean Windows system
- [ ] Start menu shortcut created
- [ ] Desktop shortcut created (if selected)
- [ ] App launches after installation
- [ ] Uninstall removes all files

### 7. Runtime Testing (Installed App)

- [ ] App starts without admin privileges
- [ ] All features work in installed version
- [ ] State persistence works (route restoration)
- [ ] No console errors in production

---

## Release Artifacts

### Required Files

| Artifact | Location | Description |
|----------|----------|-------------|
| NSIS Installer | `target/release/bundle/nsis/` | `.exe` installer |
| MSI Installer | `target/release/bundle/msi/` | Windows Installer package |
| Portable EXE | `target/release/` | Standalone executable |

### Documentation

- [ ] README.md is up to date
- [ ] CHANGELOG updated with release notes
- [ ] TAURI_IPC_COMMANDS.md current

---

## Release Steps

### 1. Final Preparation

```bash
# Ensure clean working directory
git status

# Update version in Cargo.toml if needed
# Update version in package.json if needed
# Update version in tauri.conf.json if needed
```

### 2. Create Release Build

```bash
cd projects/backend
cargo tauri build --release
```

### 3. Verify Installers

- [ ] Test NSIS installer on Windows
- [ ] Test MSI installer on Windows
- [ ] Verify installed app functionality

### 4. Create Git Tag

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 5. Create GitHub Release

- [ ] Create release from tag
- [ ] Upload NSIS installer
- [ ] Upload MSI installer
- [ ] Add release notes

---

## Post-Release

- [ ] Verify download links work
- [ ] Test downloaded installers
- [ ] Update project documentation
- [ ] Archive release artifacts

---

## Rollback Plan

If critical issues are found after release:

1. Remove release from GitHub
2. Delete git tag: `git tag -d v1.0.0 && git push origin :refs/tags/v1.0.0`
3. Fix issues
4. Re-run checklist
5. Create new release

---

## Version History

| Version | Date | Notes |
|---------|------|-------|
| 1.0.0 | TBD | Initial release |
