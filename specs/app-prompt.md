
# Project Information

- Name: Syslens
- Purpose: A desktop app that provides a modern dashboard of all the machine config information, including networking, system, and other hardware information.
- Desktop app
- Real time
- Modern dashboard
- Personal use (at first)
- Think of it as ipconfig on steroids. We want all the same services as ipconfig (for networking info) as well as a platform to build debugging/visualizing tools on as well

## Directory Structure

All code lives in the `projects/` folder:
- projects/ui - Angular frontend
- projects/backend - Rust/Tauri backend

## Spec Details

** IMPORTANT ** Organize all specs into the appropriate files and directories.

- Spec Files:
    - specs/application.spec.md
    - specs/directory-structure.spec.md
    - specs/networking-config.spec.md
    - specs/system-config.spec.md
        - Include detailed device information, including CPU, memory, storage, and other hardware information.
            - Name
            - Manufacturer
            - Model
            - Serial Number
            - Firmware Version
            - BIOS Version
            - UEFI Version
            - Boot Mode
            - Boot Order
            - Boot Priority
    - specs/hardware-config.spec.md
    - specs/storage-config.spec.md

## Advanced Features (future)

- Allow the user to specify how the machine is used.
- Analysis Mode: This will take the system information combined with the user's specified usage to provide a detailed analysis of the machine's performance and efficiency.
    - Provides upgrade recommendations based on the user's usage and the system's configuration. Price: low, medium, high.
    - Should use AI to provide the analysis.
    - Should research the internet for the best prices on the components
- Provide an analysis of the machines software, including:
    - Software installed
    - Software updates
    - Software vulnerabilities
    - Software performance
    - Software recommendations
    - Software upgrades
    - Software removals
    - Software replacements
    - Software optimizations
