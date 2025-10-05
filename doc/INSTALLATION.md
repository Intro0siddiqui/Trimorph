# Trimorph Installation Guide

This document describes the various methods available to install Trimorph on different Linux distributions.

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Installation Methods](#installation-methods)
4. [Makefile Targets](#makefile-targets)
5. [Post-Installation Setup](#post-installation-setup)
6. [Verification](#verification)

## Overview

Trimorph provides multiple installation methods to work across different Linux distributions:
- A comprehensive installation script (`install.sh`)
- A standard Makefile with various targets
- Manual installation process

All methods result in the same final installation but provide flexibility based on your preferences and system setup.

## Prerequisites

Before installing, ensure your system has:
- Rust toolchain (rustc and cargo)
- A supported init system (systemd, OpenRC, or others)
- Package managers for target distributions you want to support
- Sudo privileges for installation

### Distribution-Specific Requirements

See [CROSS_DISTRO_SETUP.md](CROSS_DISTRO_SETUP.md) for specific requirements per distribution.

## Installation Methods

### Method 1: Using the Installation Script

The installation script provides an automated installation process:

```bash
# Make the script executable
chmod +x install.sh

# Run the installation script
./install.sh
```

The script will:
1. Check dependencies
2. Build all Rust components
3. Install binaries and configurations
4. Set up required directories
5. Configure systemd slice (if available)
6. Provide post-installation instructions

### Method 2: Using Make

The Makefile provides standardized build and installation targets:

```bash
# Build all components
make build

# Install to system
sudo make install

# Or install components separately
sudo make install-bin    # Install only binaries
sudo make install-config # Install only configurations
```

### Method 3: Manual Installation

For advanced users who prefer manual control:

```bash
# 1. Build Rust components
cd tui && cargo build --release && cd ..
cd jail-runner && cargo build --release && cd ..

# 2. Create required directories
sudo mkdir -p /etc/trimorph/jails.d
sudo mkdir -p /var/cache/trimorph/packages
sudo mkdir -p /var/log/trimorph
sudo mkdir -p /var/lib/trimorph/host-installs
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/sbin

# 3. Install binaries
sudo install -m 755 tui/target/release/trimorph-tui /usr/local/bin/
sudo install -m 755 jail-runner/target/release/jail-runner /usr/local/bin/
sudo install -m 755 trimorph /usr/local/bin/

# 4. Install scripts
for script in usr/local/sbin/*; do
    sudo install -m 755 "$script" /usr/local/sbin/
done
for script in usr/local/bin/*; do
    sudo install -m 755 "$script" /usr/local/bin/
done

# 5. Install configuration files
for conf in etc/trimorph/jails.d/*.conf; do
    sudo install -m 644 "$conf" /etc/trimorph/jails.d/
done

# 6. Setup systemd slice (if systemd is available)
if command -v systemctl >/dev/null 2>&1; then
    sudo tee /etc/systemd/system/trimorph.slice > /dev/null << EOF
[Unit]
Description=Trimorph Resource Limits
Before=slices.target

[Slice]
CPUQuota=50%
MemoryMax=120M
TasksMax=50

[Install]
WantedBy=slices.target
EOF
    sudo systemctl daemon-reload
fi
```

## Makefile Targets

The Makefile provides the following targets:

### `make build` or `make`
Builds all Rust components (TUI and jail-runner) in release mode.

### `make install`
Installs all Trimorph components to the system, including:
- All binaries and scripts
- Configuration files
- Required directories
- Systemd slice (if applicable)

### `make install-bin`
Installs only binaries and scripts, excluding configurations.

### `make install-config`
Installs only configuration files and directories.

### `make setup-systemd`
Sets up the systemd slice (if systemd is available). Called automatically by `make install`.

### `make uninstall`
Removes all Trimorph components from the system.

### `make test`
Runs the test suite (if available).

### `make clean`
Removes build artifacts from the Rust projects.

### `make distclean`
Cleans all build directories completely.

### `make check`
Verifies that required dependencies are available.

### `make help`
Displays this help information.

## Post-Installation Setup

After installation, you may want to:

### 1. Verify Installation
```bash
# Check if all binaries are accessible
which trimorph
which trimorph-tui
which jail-runner

# Check if wrapper commands are available
which apt pacman dnf apk zypper
```

### 2. Bootstrap Your First Jail
```bash
# Example: Bootstrap a Debian jail
sudo trimorph-bootstrap deb

# Check status
trimorph-status
```

### 3. Test Functionality
```bash
# Test wrapper command
apt --version  # Should run in Debian jail

# Launch the TUI
trimorph
```

## Verification

To verify that Trimorph is properly installed:

### 1. Check Installation
```bash
# Check for installed binaries
ls -la /usr/local/bin/trimorph*
ls -la /usr/local/sbin/trimorph*

# Check for configuration
ls -la /etc/trimorph/jails.d/
```

### 2. Test Basic Functionality
```bash
# Check status of jails
trimorph-status

# Run a simple check
trimorph-solo --check deb  # If deb jail is configured
```

### 3. Verify Cross-Distro Functionality
```bash
# Test that the jail runner works
jail-runner --help

# Test wrapper commands
apt --help    # Should call trimorph-solo internally
pacman --help # Should call trimorph-solo internally
```

## Troubleshooting Installation

### If the TUI doesn't start
```bash
# Ensure it's built
cd tui && cargo build --release && cd ..

# Check if the binary exists
ls -la tui/target/release/trimorph-tui
```

### If wrapper commands fail
```bash
# Verify they exist
ls -la /usr/local/bin/apt
ls -la /usr/local/bin/pacman
```

### If Rust components fail to build
```bash
# Check Rust installation
rustc --version
cargo --version

# Try building individually
cd tui && cargo build --release
cd ../jail-runner && cargo build --release
```

## Uninstalling

To completely remove Trimorph from your system:

```bash
# Using Make
sudo make uninstall

# Or using the installation script approach (not available, manual removal needed)
sudo rm -rf /usr/local/bin/trimorph*
sudo rm -rf /usr/local/bin/jail-runner
sudo rm -rf /usr/local/sbin/trimorph*
sudo rm -rf /etc/trimorph
sudo rm -rf /var/cache/trimorph
sudo rm -rf /var/log/trimorph
sudo rm -rf /var/lib/trimorph
sudo rm -f /etc/systemd/system/trimorph.slice
sudo systemctl daemon-reload  # If systemctl exists
```

## Next Steps

After successful installation:
1. Configure your desired jail environments in `/etc/trimorph/jails.d/`
2. Bootstrap the jails you plan to use
3. Test functionality with the wrapper commands
4. Explore the TUI for interactive management
5. Set up automatic updates if desired