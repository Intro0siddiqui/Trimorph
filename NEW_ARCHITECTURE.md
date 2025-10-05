# New Trimorph C-Based Architecture

## Overview
This document describes the new C-based architecture for Trimorph that enables true parallel execution of multiple package managers with enhanced isolation and dependency auto-updating.

## Key Features Implemented

### 1. C-Based Core System
- **trimorph-core**: Main C executable for managing jails and package managers
- **Modular Design**: Separate components for configuration, jail management, daemon functionality, and installation
- **Enhanced Performance**: Written in C for better performance and resource usage

### 2. Parallel Execution Support
- **Simultaneous Jail Operations**: Multiple jails can now run simultaneously without conflicts
- **Independent Overlay Filesystems**: Each jail maintains its own overlay filesystem
- **Resource Management**: Proper allocation of system resources to prevent conflicts

### 3. Enhanced Isolation
- **Complete Package Manager Separation**: Each package manager runs in its own isolated environment
- **No Conflict Between Managers**: No conflicts when running pacman and apt simultaneously
- **Secure Execution**: Improved security model with proper isolation

### 4. Dependency Auto-Updating
- **Auto Update Functionality**: `trimorph-core update` command for automatic updates
- **Cross-Distro Updates**: Can update packages across different distribution jails
- **Scheduled Updates**: Support for cron-based automatic updates

### 5. Cross-Distro Installation
- **Local Package Installation**: Support for installing local package files (.deb, .pkg.tar.zst, .rpm, etc.)
- **Warp Terminal Installation**: Example implementation for the mentioned package file
- **Dependency Resolution**: Automatic dependency installation from appropriate jails

## Architecture Components

### Core Modules:
1. **Configuration Parser** (`src/config/config_parser.c`): Safely parses INI configuration files
2. **Jail Manager** (`src/core/jail_manager.c`): Manages jail lifecycle and execution
3. **Daemon** (`src/daemon/trimorphd.c`): Background service for managing multiple operations
4. **Installer** (`src/installer/installer.c`): Handles package installation and updates

### Command Line Interface:
- `trimorph-core list`: Show available jails
- `trimorph-core exec <jail> <command> [args]`: Execute command in jail
- `trimorph-core update`: Check for updates
- `trimorph-core auto-update`: Perform auto-updates
- `trimorph-core install-local <pkg>`: Install local package file
- `trimorph-core daemon`: Run as background service

## Implementation Details

### Parallel Execution
The new architecture enables parallel execution by:
- Using separate overlay mount points for each jail
- Managing separate process IDs for each jail instance
- Implementing proper resource isolation between concurrent operations

### Security Model
- Safe configuration parsing without `eval`
- Proper sandboxing for package manager execution
- Isolated filesystem operations per jail

### Package Handling
- Support for .deb, .pkg.tar.zst, .rpm, and other package formats
- Automatic package type detection
- Dependency extraction and management

## Usage Examples

### Running Pacman and Apt Commands
```bash
# Run pacman commands in Arch jail
sudo trimorph-core exec arch -Syu

# Run apt commands in Debian jail
sudo trimorph-core exec deb update && sudo trimorph-core exec deb upgrade

# These can now run simultaneously without conflicts
```

### Installing Local Packages (e.g., Warp Terminal)
```bash
sudo trimorph-core install-local /path/to/warp-terminal-v0.2025.10.01.08.12.stable_02-1-x86_64.pkg.tar.zst
```

### Auto-Update Functionality
```bash
# Check for updates
sudo trimorph-core update

# Set up automatic updates
sudo trimorph-core auto-update
```

## Benefits Over Previous System

1. **True Parallelism**: Multiple package managers can run simultaneously
2. **Enhanced Isolation**: Improved security and no conflicts between package managers
3. **Auto Dependency Updates**: Built-in update checking and dependency resolution
4. **Cross-Distro Compatibility**: Better support for installing packages across distributions
5. **C-Based Performance**: Better resource usage and performance
6. **Modern Architecture**: Clean, maintainable codebase with clear separation of concerns

## Files Created

- `src/trimorph.h`: Main header file with definitions and function declarations
- `src/core/main.c`: Main entry point for the application
- `src/config/config_parser.c`: Safe configuration file parsing
- `src/core/jail_manager.c`: Jail lifecycle and execution management
- `src/daemon/trimorphd.c`: Background daemon functionality
- `src/installer/installer.c`: Package installation and update functionality
- `Makefile_c`: Build system for C-based components
- `test_parallel.sh`: Demo script showing new capabilities

The new C-based architecture successfully implements all requirements: parallel package manager execution, enhanced isolation, dependency auto-updating, and cross-distro package installation capabilities.