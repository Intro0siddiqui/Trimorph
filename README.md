# **Simplified Package Management Core (pkgmgr-core)**

## Overview

A lightweight, efficient package management system that provides direct access to system package managers without complex sandboxing. This implementation focuses on the core functionality of managing packages across different formats and systems.

## Key Features

- **Direct System Integration**: Works directly with system package managers (apt, pacman, dnf, zypper, emerge, apk, etc.)
- **No Sandbox Overhead**: Bypasses complex containerization systems for improved performance
- **Multi-Format Support**: Handles various package formats including .deb, .pkg.tar.zst, .pkg.tar.xz, .rpm, .apk, .tbz
- **Lightweight**: Minimal resource usage compared to complex sandboxing systems
- **Cross-Platform**: Works across different Linux distributions

## Architecture

The system consists of a single C-based binary that:
1. Identifies the appropriate package manager for the host system
2. Validates package format and available system tools
3. Executes package management commands directly
4. Handles error reporting and status management

## Installation

The core binary is already installed as `/usr/local/bin/pkgmgr-core` and ready to use.

## Usage

```bash
# Install a local package file
pkgmgr-core install <package-file>

# Execute package manager commands
pkgmgr-core run emerge --sync
pkgmgr-core run apt update
pkgmgr-core run pacman -Syu

# Check if a package manager exists on the system
pkgmgr-core check emerge

# List supported package formats
pkgmgr-core supported-formats
```

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