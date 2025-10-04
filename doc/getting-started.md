# Trimorph Getting Started Guide

This guide provides a step-by-step introduction to setting up and using Trimorph.

## Prerequisites

Before installing Trimorph, ensure your system has:

- **Operating System**: Gentoo Linux (primary target)
- **Init System**: systemd (primary) or OpenRC (with rust jail-runner)
- **Required Tools**:
  - `systemd-nspawn` (for systemd systems) or `bubblewrap` (for OpenRC)
  - Rust toolchain (for TUI and jail-runner)
  - Package manager tools for target distros (apt, pacman, etc.)

## Installation

### 1. Install Dependencies
```bash
sudo emerge -av sys-apps/systemd  # Core dependency
# Install package managers for target distros:
sudo emerge -av dev-util/apt app-arch/pacman app-arch/dpkg app-arch/debootstrap
```

### 2. Set Up Directories
```bash
sudo mkdir -p /etc/trimorph/{jails.d,runtime} /var/log/trimorph
sudo mkdir -p /usr/local/{bin,sbin}
```

### 3. Install Systemd Slice Resource Limits
Create `/etc/systemd/system/trimorph.slice`:
```
[Slice]
CPUQuota=50%
MemoryMax=120M
TasksMax=50
```
```bash
sudo systemctl daemon-reload
```

### 4. Deploy Scripts
Copy all Trimorph scripts to the appropriate locations:
- Scripts in `usr/local/sbin/` should go to `/usr/local/sbin/`
- Scripts in `usr/local/bin/` should go to `/usr/local/bin/`

## Basic Configuration

### 1. Create Your First Jail Configuration
Example for Debian (`/etc/trimorph/jails.d/deb.conf`):
```
name=deb
root=/usr/local/trimorph/deb
bootstrap=debootstrap --variant=minbase bookworm {root} http://deb.debian.org/debian
pkgmgr=/usr/bin/apt
pkgmgr_args=-o Root={root} -o Dir::State::status={root}/var/lib/dpkg/status -o Dir::Cache::Archives={root}/var/cache/apt/archives
mounts=/run/user/$(id -u)/bus
env=DEBIAN_FRONTEND=noninteractive,HOME=/tmp
```

### 2. Bootstrap the Jail
```bash
sudo trimorph-bootstrap deb
```

## Basic Usage

### 1. Run Commands in the Jail
```bash
# Using wrappers (recommended)
apt update
apt install htop

# Or using trimorph-solo directly
trimorph-solo deb apt update
trimorph-solo deb apt install htop
```

### 2. Check Jail Status
```bash
trimorph-status
```

### 3. Validate Setup
```bash
trimorph-solo --check deb
```

### 4. Dry Run Simulation
```bash
trimorph-solo --dry-run deb apt install htop
```

## Cross-Distribution Package Management

### 1. Validate Packages
```bash
trimorph-validate deb htop
```

### 2. Preview Installation
```bash
trimorph-dry-run deb htop
```

### 3. Check Dependencies
```bash
trimorph-check-deps deb htop
```

### 4. Install to Host
```bash
trimorph-install-to-host deb htop
```

### 5. Uninstall from Host
```bash
# Find the log file
ls /var/lib/trimorph/host-installs/
# Uninstall using the log
trimorph-uninstall-from-host /var/lib/trimorph/host-installs/deb_timestamp.log
```

## Intermediate Features

### 1. Export Packages
```bash
trimorph-export deb htop vim
```

### 2. Database Management
```bash
# List installed packages
trimorph-db list-packages
```

### 3. Configuration Management
```bash
# Check current settings
trimorph-config list

# Enable auto-update
trimorph-config set auto_update true
trimorph-config set check_updates true
```

### 4. Update Management
```bash
# Check for updates
trimorph-update-check

# Manual update run
trimorph-auto-update
```

## Advanced Usage

### 1. Multiple Distros
Set up multiple jails with different configurations:
- `/etc/trimorph/jails.d/arch.conf` for Arch Linux
- `/etc/trimorph/jails.d/fedora.conf` for Fedora
- `/etc/trimorph/jails.d/alpine.conf` for Alpine

### 2. Parallel Execution (Expert)
```bash
trimorph-solo --parallel deb apt install package
```
⚠️ **Warning**: Use with caution as this can consume significant resources

### 3. Diagnostics
```bash
# Check if jail is ready
trimorph-solo --check deb

# Simulate commands
trimorph-solo --dry-run deb command
```

### 4. Logging
Enable debug logging:
```bash
export TRIMORPH_DEBUG=1
apt update  # Logs to /var/log/trimorph/deb.log
```

## Using the TUI

### 1. Build the TUI
```bash
cd tui
cargo build --release
```

### 2. Run the TUI
```bash
./target/release/trimorph-tui
# Or using the main trimorph entry point
./trimorph
```

### 3. TUI Controls
- `q` - Quit
- Up/Down arrows - Navigate jails
- `c` - Run check (--check) on selected jail
- `d` - Run dry-run (--dry-run) on selected jail

## Best Practices

1. **Always dry run first**: Use `trimorph-dry-run` before installing packages
2. **Check dependencies**: Use `trimorph-check-deps` for potential conflicts
3. **Verify packages**: Use `trimorph-validate` to ensure packages exist
4. **Monitor logs**: Enable `TRIMORPH_DEBUG` for troubleshooting
5. **Regular updates**: Set up cron for automatic update checking
6. **Backup configs**: Keep copies of your jail configurations

## Troubleshooting Start

If you encounter issues, check:
1. `trimorph-status` - Verify jail status
2. `trimorph-solo --check <jail>` - Validate configuration
3. `trimorph-dry-run <jail> <command>` - Simulate without running
4. Logs in `/var/log/trimorph/` when debugging is enabled

## Next Steps

After completing this guide:
- Explore the configuration examples in `/etc/trimorph/jails.d/`
- Try setting up jails for different distributions
- Experiment with cross-distribution package installation
- Set up automatic update checking
- Consider using the TUI for interactive management