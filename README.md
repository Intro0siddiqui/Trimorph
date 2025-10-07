# **Trimorph - Enhanced Package Management System**

## Overview

A lightweight, efficient package management system that provides direct access to system package managers without complex sandboxing. This enhanced implementation focuses on the core functionality of managing packages across different formats and systems with improved error handling, auto dependency updates, package manager conflict resolution, and comprehensive security protections.

## How It Works

The system consists of a single C-based binary that:
1. Identifies the appropriate package manager for the host system
2. Validates package format and available system tools
3. Executes package management commands directly with security validations
4. Handles error reporting and status management

### Core Components
- `execute_command()`: Safely executes commands using fork/exec approach
- `validate_file_path()`: Prevents path traversal attacks
- `validate_command_name()`: Prevents command injection
- Package format handlers for .deb, .pkg.tar.zst, .pkg.tar.xz, .rpm, .apk, .tbz

## Key Features

- **Direct System Integration**: Works directly with system package managers (apt, pacman, dnf, zypper, emerge, apk, etc.)
- **No Sandbox Overhead**: Bypasses complex containerization systems for improved performance
- **Multi-Format Support**: Handles various package formats including .deb, .pkg.tar.zst, .pkg.tar.xz, .rpm, .apk, .tbz
- **Lightweight**: Minimal resource usage compared to complex sandboxing systems
- **Cross-Platform**: Works across different Linux distributions
- **Enhanced Security**: Includes input validation, command injection protection, and path traversal prevention
- **Legacy System Cleaned**: Removed all legacy systemd-nspawn-based architecture for simplified operation

## Installation

To use the binary, compile it from source:
```bash
gcc -o trimorph final_pkgmgr.c
sudo cp trimorph /usr/local/bin/
```

## Usage

```bash
# Install a local package file
trimorph install <package-file>

# Execute package manager commands
trimorph run emerge --sync
trimorph run apt update
trimorph run pacman -Syu

# Check if a package manager exists on the system
trimorph check emerge

# List supported package formats
trimorph supported-formats

# Check system status and detect running package managers
trimorph status
```

## Troubleshooting

### Common Issues
1. **Permission Denied**: Ensure you have appropriate permissions for the target package manager
2. **Package Manager Not Found**: Verify the package manager is installed on your system
3. **Command Injection Blocked**: Invalid characters in command names are rejected for security
4. **Path Traversal Blocked**: Directory traversal attempts (../) are prevented

### Error Messages
- "Error: Invalid file path containing directory traversal" - Path contains forbidden characters
- "Error: Invalid package manager name" - Command name contains invalid characters
- "Warning: Failed to update dependencies" - Dependency update failed (non-fatal)
- "Error: Another package manager is currently running" - Conflict detection triggered

### Security Validation
- All file paths are validated before use
- Command names are sanitized to prevent injection
- Directory traversal attempts are blocked
- Buffer overflow protections are in place

## Ups (Advantages)

- **Cross-Platform Compatibility**: Supports all major Linux package managers (apt, pacman, dnf, zypper, emerge, apk)
- **Efficient Operation**: Direct system integration with minimal overhead
- **Security Enhanced**: Includes command injection and path traversal protections
- **User-Friendly**: Comprehensive error messages and troubleshooting suggestions
- **Conflict Prevention**: Monitors for running package managers to prevent system conflicts
- **Auto Dependency Updates**: Automatically updates system dependencies before installations
- **Lightweight**: Minimal memory footprint with pure C implementation

## Downs (Limitations)

- **System Dependencies**: Requires underlying package managers to be installed
- **Privilege Requirements**: Some operations need root privileges via underlying package managers
- **No Rollback**: No built-in rollback mechanism for failed installations
- **Limited to Supported Formats**: Only supports the specific package formats defined in the code

## Enhanced Features

- **Conflict Detection**: Automatically detects if another package manager is running to prevent system conflicts
- **Auto Dependency Updates**: Automatically updates system dependencies before installing packages
- **Improved Error Handling**: Provides more detailed error messages and troubleshooting suggestions
- **Status Monitoring**: Check system status with the new `status` command
- **Security Validations**: Input validation, command injection protection, path traversal prevention

## Supported Package Formats

- `.deb` - Debian packages (requires apt/dpkg)
- `.pkg.tar.zst`, `.pkg.tar.xz` - Arch Linux packages (requires pacman)
- `.rpm` - Red Hat packages (requires rpm/dnf/yum)
- `.apk` - Alpine packages (requires apk)
- `.tbz` - Gentoo packages (requires emerge)

## Implementation Details

Written in pure C for maximum portability and efficiency. The implementation:
- Directly executes system package managers with security validations
- Detects available tools on the host system
- Supports local package installation without complex sandboxing
- Provides consistent interface across different distributions
- Includes comprehensive input validation and security checks

## License

This project is licensed under the terms specified in the LICENSE file.

## Architecture

The system consists of a single C-based binary that:
1. Identifies the appropriate package manager for the host system
2. Validates package format and available system tools
3. Executes package management commands directly
4. Handles error reporting and status management

## Installation

To use the binary, you can copy it to a system directory:
```bash
sudo cp final-pkgmgr /usr/local/bin/trimorph
```

## Usage

```bash
# Install a local package file
trimorph install <package-file>

# Execute package manager commands
trimorph run emerge --sync
trimorph run apt update
trimorph run pacman -Syu

# Check if a package manager exists on the system
trimorph check emerge

# List supported package formats
trimorph supported-formats

# Check system status and detect running package managers
trimorph status
```

## Enhanced Features

- **Conflict Detection**: Automatically detects if another package manager is running to prevent system conflicts
- **Auto Dependency Updates**: Automatically updates system dependencies before installing packages
- **Improved Error Handling**: Provides more detailed error messages and troubleshooting suggestions
- **Status Monitoring**: Check system status with the new `status` command

## Supported Package Formats

- `.deb` - Debian packages (requires apt/dpkg)
- `.pkg.tar.zst`, `.pkg.tar.xz` - Arch Linux packages (requires pacman)
- `.rpm` - Red Hat packages (requires rpm/dnf/yum)
- `.apk` - Alpine packages (requires apk)
- `.tbz` - Gentoo packages (requires emerge)

## Implementation Details

Written in pure C for maximum portability and efficiency. The implementation:
- Directly executes system package managers
- Detects available tools on the host system
- Supports local package installation without complex sandboxing
- Provides consistent interface across different distributions

## License

This project is licensed under the terms specified in the LICENSE file.