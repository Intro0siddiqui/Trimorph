# Cross-Distribution Setup Guide for Trimorph

This guide explains how to set up Trimorph to work properly across different Linux distributions.

## Table of Contents
1. [Overview](#overview)
2. [Supported Distributions](#supported-distributions)
3. [Installation on Different Distributions](#installation-on-different-distributions)
4. [Configuration](#configuration)
5. [Troubleshooting](#troubleshooting)

## Overview

Trimorph is designed to work across different Linux distributions by using containerization with either `systemd-nspawn` (on systemd systems) or `bubblewrap` (on OpenRC systems). This guide will help you set it up correctly on your distribution.

## Supported Distributions

Trimorph supports the following distributions out of the box:
- Gentoo (primary development platform)
- Debian/Ubuntu
- Fedora/RHEL/CentOS/AlmaLinux/Rocky
- Arch Linux/Manjaro
- Alpine Linux

## Installation on Different Distributions

### Gentoo

```bash
# Install dependencies
sudo emerge -av app-arch/dpkg dev-util/apt app-arch/pacman app-arch/debootstrap sys-apps/systemd dev-lang/rust

# Build and install
make build
sudo make install
```

### Debian/Ubuntu

```bash
# Install dependencies
sudo apt update
sudo apt install -y debootstrap systemd-container rustc cargo bubblewrap

# Build and install
make build
sudo make install
```

### Fedora/RHEL-based

```bash
# Install dependencies
sudo dnf install -y debootstrap systemd-container rust-toolset bubblewrap

# Or for RHEL/CentOS:
sudo yum install -y debootstrap systemd-container rust bubblewrap

# Build and install
make build
sudo make install
```

### Arch Linux

```bash
# Install dependencies
sudo pacman -S --noconfirm debootstrap arch-install-scripts systemd rust bubblewrap

# Build and install
make build
sudo make install
```

### Alpine Linux

```bash
# Install dependencies
sudo apk add debootstrap systemd rust bubblewrap

# Build and install
make build
sudo make install
```

## Configuration

### Systemd Slice Setup (For systemd systems)

On systemd-based distributions, Trimorph uses a slice to limit resource usage. This is automatically set up during installation, but you can verify it:

```bash
# Check if the slice exists
ls /etc/systemd/system/trimorph.slice

# Reload systemd if needed
sudo systemctl daemon-reload
```

### Package Manager Dependencies

Each target distribution you want to use requires its respective bootstrap tool:

- **Debian/Ubuntu**: `debootstrap`
- **Arch Linux**: `arch-install-scripts` (provides `pacstrap`)
- **Alpine**: `apk` (already part of Alpine)
- **Fedora**: `dnf` or `yum`

Install the bootstrap tools for distributions you plan to use:

```bash
# For Debian support
sudo apt install debootstrap  # Debian/Ubuntu
sudo emerge debootstrap      # Gentoo
sudo pacman -S debootstrap   # Arch

# For Arch support
sudo apt install arch-install-scripts  # Debian/Ubuntu
sudo emerge arch-install-scripts       # Gentoo
sudo pacman -S arch-install-scripts    # Arch
```

## Verifying Cross-Distro Functionality

After installation, verify that Trimorph works properly:

```bash
# Check if all required binaries are available
trimorph-status

# Bootstrap a jail (example with Debian)
sudo trimorph-bootstrap deb

# Test the wrapper command
apt --version  # This should run in the Debian jail

# Launch the TUI
trimorph
```

## Common Issues and Solutions

### Issue: "systemd-nspawn not found"

**Solution:** Install systemd container tools:
- Debian/Ubuntu: `sudo apt install systemd-container`
- Fedora: `sudo dnf install systemd-container`
- Arch: `sudo pacman -S systemd`

### Issue: "bubblewrap not found" (on OpenRC systems)

**Solution:** Install bubblewrap:
- Most distributions: `sudo apt install bubblewrap` or equivalent

### Issue: Permission denied errors

**Solution:** Ensure you can run sudo without password prompts for Trimorph commands, or run them with sudo as needed.

### Issue: Package manager wrappers not working

**Solution:** Verify that the wrapper scripts are installed and executable:
```bash
ls -la /usr/local/bin/apt
ls -la /usr/local/bin/pacman
# etc.
```

## Custom Jail Configuration for Other Distributions

You can add support for other distributions by creating configuration files in `/etc/trimorph/jails.d/`. For example, to add support for openSUSE:

Create `/etc/trimorph/jails.d/opensuse.conf`:
```
name=opensuse
root=/usr/local/trimorph/opensuse
bootstrap=zypper --root {root} --non-interactive ar https://download.opensuse.org/tumbleweed/repo/oss/ && zypper --root {root} --non-interactive install --no-recommends filesystem
pkgmgr=/usr/bin/zypper
pkgmgr_args=--root {root}
env=HOME=/tmp
mounts=
```

## Advanced Configuration

### Environment Variables

Trimorph respects the following environment variables for testing and development:

- `TRIMORPH_ETC_DIR` - Configuration directory (default: /etc/trimorph)
- `TRIMORPH_BASE_DIR` - Base jail images directory (default: /usr/local/trimorph/base)
- `TRIMORPH_CACHE_DIR` - Package cache directory (default: /var/cache/trimorph/packages)
- `TRIMORPH_LOG_DIR` - Log directory (default: /var/log/trimorph)
- `TRIMORPH_TEST_MODE` - Enable test mode (default: 0)

### Resource Limits

The Trimorph slice limits resources as follows:
- CPU Quota: 50%
- Memory Max: 120MB
- Tasks Max: 50

You can modify these limits by editing `/etc/systemd/system/trimorph.slice`.

## Testing Cross-Distro Functionality

To verify that Trimorph works properly across distributions, run:

```bash
# Test that the main entry point works
trimorph --help

# Test that a jail can be bootstrapped
sudo trimorph-bootstrap deb

# Test that wrapper commands work
apt update

# Run the test suite if available
./test/bats-core/bin/bats test/core.bats
```

## Troubleshooting Commands

Use these commands to debug issues:

```bash
# Check active jails
trimorph-status

# Check for Trimorph processes
ps aux | grep trimorph

# Check systemd status (if using systemd)
systemctl list-units --type=scope | grep trimorph

# Check logs
tail -f /var/log/trimorph/*.log

# Run with debug output
export TRIMORPH_DEBUG=1
apt update
```

## Uninstalling

To remove Trimorph from your system:

```bash
sudo make uninstall
sudo rm -rf /usr/local/trimorph  # If any files remain
sudo rm -rf /etc/trimorph
sudo rm -rf /var/cache/trimorph
sudo rm -rf /var/log/trimorph
sudo rm -rf /var/lib/trimorph

# Remove systemd slice if it exists
sudo rm -f /etc/systemd/system/trimorph.slice
sudo systemctl daemon-reload
```

## Contributing

If you encounter issues specific to a distribution, please report them in the issue tracker. Include:
- Distribution name and version
- Kernel version (`uname -a`)
- Systemd/OpenRC version
- Error messages
- Steps to reproduce

Happy cross-distro packaging with Trimorph!