# **Trimorph: Modular Multi-Distro Package Management on Gentoo**

---

## ✅ Key Features
- **No PID files** → uses `systemctl` unit state as source of truth.
- **No `eval`** → safe config parsing via associative arrays.
- **No nested scopes** → `systemd-nspawn` runs **directly under `systemd-run`**.
- **No host `/usr` bind-mount** → only GPU/X11/IPC paths are shared.
- **User-writable jails** → bootstrap ensures correct ownership.
- **Initialization guard** → prevents use of unbootstrapped jails.
- **Parallel Package Managers** (NEW) → Run pacman, apt, and other managers simultaneously without conflicts.
- **C-Based Performance** (NEW) → Enhanced performance through C-based core components.
- **Auto Dependency Updating** (NEW) → Built-in automatic dependency checking and updating.
- **True Isolation** (NEW) → Enhanced separation between different package managers.
- **Enhanced Cross-Distro Installation** (NEW) → Improved local package installation including .pkg.tar.zst format.

## Executive Summary

Trimorph enables on-demand use of foreign package managers (e.g., `apt`, `pacman`) on any Linux distribution. The original system works with **only one distro jail active at a time**, but the **new C-based architecture supports true parallel execution** of multiple package managers simultaneously. It works on Gentoo, Debian, Ubuntu, Fedora, Arch, Alpine, and other Linux distributions. It uses **transient systemd scopes** (on systemd systems) or **bubblewrap containers** (on OpenRC systems) for zero idle overhead (< 30 MB RAM, 0% CPU when idle) and is ideal for netbooks, SBCs, and embedded systems.

New distros are added via **drop-in `.conf` files**—no code changes required.

The new C-based architecture provides:
- **Parallel Package Management**: Run multiple package managers simultaneously (pacman, apt, etc.)
- **Enhanced Auto-Updates**: Automatic dependency checking and updating across all package managers
- **Improved Performance**: C-based core for faster execution
- **Better Cross-Distro Support**: Enhanced local package installation including .pkg.tar.zst format
- **True Isolation**: Enhanced separation preventing conflicts between package managers

## Quick Start

### Installation
```bash
# Clone the repository
git clone https://github.com/your/repo.git
cd Trimorph

# Method 1: Using the installation script (recommended)
chmod +x install.sh
./install.sh

# Method 2: Using Make
make build
sudo make install
```

### Basic Usage
```bash
# Bootstrap a Debian jail
sudo trimorph-bootstrap deb

# Use package managers through wrappers
apt update
apt install htop

# Check jail status
trimorph-status

# Launch the TUI
trimorph
```

For detailed setup on your specific distribution, see [CROSS_DISTRO_SETUP.md](CROSS_DISTRO_SETUP.md).

## Architecture Overview

Trimorph now uses a layered, caching architecture to provide fast, ephemeral, and isolated environments across all Linux distributions, enhanced with a new C-based core for parallel execution.

```
/usr/local/trimorph/
├── base/
│   ├── deb/          → Read-only base image for Debian
│   └── arch/         → Read-only base image for Arch
└── deb/              → Ephemeral OverlayFS mount point for live jail
/var/cache/trimorph/
└── packages/         → Shared package cache for all jails
/usr/local/sbin/
└── trimorph-core     → New C-based enhanced functionality
/src/                 → C source code for the new architecture
```

The architecture supports:
- **Cross-Distribution Compatibility**: Works on systemd and OpenRC-based systems
- **Automatic Init Detection**: Uses appropriate sandboxing backend based on host system
- **Consistent Interface**: Same commands work across all distributions
- **Resource Efficiency**: Ephemeral containers with minimal overhead
- **Parallel Execution** (NEW): Multiple package managers can run simultaneously
- **Enhanced Performance** (NEW): C-based core for improved efficiency
- **True Isolation** (NEW): Enhanced separation between package managers
- **Auto Dependency Updating** (NEW): Built-in update checking and dependency management

### 2.1 Smart Caching Workflow
1.  **Bootstrap:** `trimorph-bootstrap` creates a read-only base image in `/usr/local/trimorph/base/`.
2.  **Execution:** `trimorph-solo` uses `systemd-nspawn`'s native `--overlay` support to create a temporary, writable filesystem layer on top of the read-only base.
3.  **Launch:** The command runs inside this ephemeral overlay.
4.  **Cleanup:** On exit, the overlay's writable layer is automatically discarded, leaving the base image untouched.

The new C-based architecture adds:
5.  **Parallel Execution:** `trimorph-core` can manage multiple overlays simultaneously for different package managers
6.  **Enhanced Management:** Better resource allocation and cleanup for concurrent operations

This architecture provides:
-   **Speed:** Jails start almost instantly, as there's no need to copy a root filesystem.
-   **Efficiency:** A shared package cache (`/var/cache/trimorph/packages`) minimizes redundant downloads.
-   **Purity:** The base images remain clean, and all changes are temporary.
-   **Parallelism:** Multiple package managers can operate simultaneously (NEW)
-   **Enhanced Performance:** C-based core for faster operations (NEW)

## Installation

### 4.1 Host Dependencies

Trimorph requires different dependencies depending on your host distribution. Install the relevant packages:

**Gentoo:**
```bash
sudo emerge -av app-arch/dpkg dev-util/apt app-arch/pacman app-arch/debootstrap sys-apps/systemd rust cargo
```

**Debian/Ubuntu:**
```bash
sudo apt install debootstrap systemd-container rustc cargo bubblewrap
```

**Fedora/RHEL:**
```bash
sudo dnf install debootstrap systemd-container rust-toolset bubblewrap
# or for RHEL/CentOS:
sudo yum install debootstrap systemd-container rust cargo bubblewrap
```

**Arch Linux:**
```bash
sudo pacman -S --noconfirm debootstrap arch-install-scripts systemd rust bubblewrap
```

**Alpine:**
```bash
sudo apk add debootstrap systemd rust bubblewrap
```

### 4.2 Quick Installation

Trimorph provides multiple ways to install:

**Method 1: Using the installation script**
```bash
chmod +x install.sh
./install.sh
```

**Method 2: Using Make**
```bash
make build
sudo make install
```

The installation now includes both the original bash/Rust system and the new enhanced C-based architecture. The C-based `trimorph-core` binary will be installed to `/usr/local/sbin/` and can be used alongside existing functionality.

### 4.3 Manual Installation

If you prefer manual installation:

```bash
# Build the Rust components
cd tui && cargo build --release
cd ../jail-runner && cargo build --release
cd ..

# Install all components
sudo make install
```

### 4.4 Directory Setup (if installing manually)

```bash
sudo mkdir -p /etc/trimorph/{jails.d,runtime} /var/log/trimorph /var/cache/trimorph/packages /var/lib/trimorph/host-installs
sudo mkdir -p /usr/local/{bin,sbin}
```

## Core Utilities

This section provides an overview of the scripts that power Trimorph.

### 5.1 Safe Config Parser (`/usr/local/sbin/trimorph-parse-conf`)
A safe INI parser that reads `.conf` files without using `eval`, preventing code injection.

### 5.2 Solo Launcher (`/usr/local/sbin/trimorph-solo`)
The core script that manages jails. It ensures only one jail is active at a time, loads the correct configuration, and launches the package manager inside a transient `systemd-nspawn` container.

### 5.3 Bootstrap Tool (`/usr/local/sbin/trimorph-bootstrap`)
A one-time setup script that prepares a jail's root directory and runs the bootstrap command defined in its `.conf` file.

### 5.4 Enhanced Core CLI Tools (NEW - C-Based Architecture)
Trimorph now includes a new C-based architecture for enhanced functionality:

#### New C-Based Components
- **`/usr/local/sbin/trimorph-core`**: Main C executable providing enhanced functionality with parallel execution, auto-updating, and improved performance
- **`/src/core/main.c`**: Main entry point for the C-based system
- **`/src/config/config_parser.c`**: C-based safe configuration file parser with enhanced performance
- **`/src/core/jail_manager.c`**: Advanced jail management with true parallel execution support
- **`/src/daemon/trimorphd.c`**: Background daemon for managing concurrent operations
- **`/src/installer/installer.c`**: Enhanced installer with cross-distro package management

### 5.5 Component Overview
This section details each file's specific function in the Trimorph system:

#### TUI Components
- **`/tui/src/main.rs`**: Main entry point for the Terminal UI, handles UI rendering, user input, and integration with CLI tools
- **`/tui/src/components/log_view.rs`**: Renders live jail logs in the UI
- **`/tui/src/components/resource_monitor.rs`**: Displays CPU and Memory usage gauges
- **`/tui/src/components/status_bar.rs`**: Shows status information and keyboard shortcuts
- **`/tui/src/components/title_bar.rs`**: Displays the Trimorph logo and title
- **`/tui/src/config.rs`**: Handles TUI configuration and theming

#### Core CLI Tools (Original)
- **`/usr/local/sbin/trimorph-solo`**: Core execution engine for running commands in isolated jails
- **`/usr/local/sbin/trimorph-parse-conf`**: Safe configuration file parser (no eval usage)
- **`/usr/local/sbin/trimorph-bootstrap`**: Jail initialization and setup tool
- **`/usr/local/sbin/trimorph-install`**: Core installation engine for packages

#### Enhanced C-Based Tools (NEW)
- **`/usr/local/sbin/trimorph-core`**: Enhanced C-based core with parallel package manager support
- **`/usr/local/bin/trimorph`**: Main entry point that can use either traditional or enhanced system

#### Cross-Distribution Installation Tools
- **`/usr/local/bin/trimorph-install-to-host`**: Install packages from jails to the host system
- **`/usr/local/bin/trimorph-uninstall-from-host`**: Remove previously installed foreign packages
- **`/usr/local/bin/trimorph-validate`**: Verify packages exist in a jail's repository
- **`/usr/local/bin/trimorph-dry-run`**: Preview installations without executing them
- **`/usr/local/bin/trimorph-check-deps`**: Analyze dependency conflicts before installation

#### Update Management Tools
- **`/usr/local/bin/trimorph-update-check`**: Check for updates to installed foreign packages
- **`/usr/local/bin/trimorph-auto-update`**: Automatic update processing
- **`/usr/local/bin/trimorph-config`**: Configuration management for update settings
- **`/usr/local/bin/trimorph-cron-check`**: Cron-based update checking

#### Utility Tools
- **`/usr/local/bin/trimorph-status`**: Check status of all configured jails
- **`/usr/local/bin/trimorph-cleanup`**: Stop all running Trimorph scopes
- **`/usr/local/bin/trimorph-export`**: Export packages from jails for offline use
- **`/usr/local/bin/trimorph-db`**: Package database management

#### Enhanced C-Based Utility Tools (NEW)
- **`/usr/local/sbin/trimorph-core`**: Main C-based enhanced functionality
  - `trimorph-core list`: List all jails with status information
  - `trimorph-core exec <jail> <command> [args...]`: Execute commands in jails with parallel support
  - `trimorph-core update`: Check for updates across all package managers
  - `trimorph-core auto-update`: Perform automatic updates
  - `trimorph-core install-local <package>`: Enhanced local package installation
  - `trimorph-core start/stop <jail>`: Manage jail states

#### Cross-Platform Support
- **`/jail-runner/src/main.rs`**: Rust-based jail runner that automatically detects init system (systemd/OpenRC), uses systemd-nspawn for systemd systems and bubblewrap for OpenRC systems
- **`/jail-runner/Cargo.toml`**: Build configuration for the cross-platform jail runner
- **`/jail-runner/target/release/jail-runner`**: Built binary that provides consistent jail execution interface across different init systems

#### Installation and Build Tools
- **`/install.sh`**: Cross-distribution installation script
- **`/setup-distro.sh`**: Distribution detection and setup helper
- **`/Makefile`**: Standard build and installation automation
- **`/tui/Cargo.toml`**: Build configuration for the Terminal UI

#### Main Entry Points
- **`/trimorph`**: Project root entry point that launches the TUI when executed

#### Documentation
- **`/doc/`**: Complete documentation directory with guides for installation, configuration, and troubleshooting across all distributions

#### Wrapper Scripts
- **`/usr/local/bin/{apt,pacman,dnf,apk,zypper,emerge}`**: Package manager wrappers that redirect to trimorph-solo

## Default Jail Configurations

### 6.1 Debian (`/etc/trimorph/jails.d/deb.conf`)
```ini
name=deb
root=/usr/local/trimorph/deb
bootstrap=debootstrap --variant=minbase bookworm {root} http://deb.debian.org/debian
pkgmgr=/usr/bin/apt
pkgmgr_args=-o Root={root} -o Dir::State::status={root}/var/lib/dpkg/status -o Dir::Cache::Archives={root}/var/cache/apt/archives
mounts=/run/user/$(id -u)/bus
env=DEBIAN_FRONTEND=noninteractive,HOME=/tmp
```

### 6.2 Arch (`/etc/trimorph/jails.d/arch.conf`)
```ini
name=arch
root=/usr/local/trimorph/arch
bootstrap=curl -L https://geo.mirror.pkgbuild.com/iso/latest/archlinux-bootstrap-x86_64.tar.gz | tar -xz -C {root} --strip=1 && systemd-nspawn -q -D {root} sh -c "pacman-key --init && pacman-key --populate archlinux"
pkgmgr=/usr/bin/pacman
pkgmgr_args=--root {root} --dbpath {root}/var/lib/pacman --cachedir {root}/var/cache/pacman/pkg
env=LANG=C.UTF-8,HOME=/tmp
```

### 6.3 Expanded Support
Trimorph also includes configurations for other popular package managers:
-   **Fedora:** `etc/trimorph/jails.d/fedora.conf`
-   **Alpine:** `etc/trimorph/jails.d/alpine.conf`
-   **openSUSE:** `etc/trimorph/jails.d/opensuse.conf`

To use them, simply bootstrap the desired jail (e.g., `sudo trimorph-bootstrap fedora`) and then use the corresponding wrapper (`dnf`, `apk`, `zypper`).

## Wrappers & Integration

### 7.1 Package Manager Wrappers
Simple wrappers in `/usr/local/bin` redirect calls for `apt` and `pacman` to the `trimorph-solo` launcher.

### 7.2 Portage Safety Hook (`/etc/portage/bashrc`)
An optional hook that automatically stops any active Trimorph jail before a Portage `emerge` process begins, preventing potential conflicts.

### 7.3 Cross-Distribution Package Installation
Trimorph now supports installing packages from foreign package managers directly to the host system using the new installation tools:

- `trimorph-install-to-host <jail> <package> [package...]`: Install packages from a jail to the host system
- `trimorph-uninstall-from-host <log_file>`: Remove previously installed packages
- `trimorph-validate <jail> <package>`: Check if a package exists in a jail's repository
- `trimorph-dry-run <jail> <package> [package...]`: Preview what would be installed (including dependencies)
- `trimorph-check-deps <jail> <package>`: Analyze potential dependency conflicts

### 7.4 Automatic Dependency Update Management
Trimorph now includes automatic dependency checking and updating functionality:

- `trimorph-update-check`: Check for available updates to installed foreign packages
- `trimorph-auto-update`: Manually run the auto-update process
- `trimorph-config {get|set} <setting> [value]`: Configure update settings
- `trimorph-cron-check`: Run from cron for periodic update checking

Configuration options:
- `auto_update`: Enable/disable automatic updates (default: false)
- `check_updates`: Enable/disable update checking (default: true)
- `update_security_only`: Only update security-related packages (default: false)

### 7.5 Enhanced Update Management (NEW - C-Based)
The new C-based architecture provides enhanced update functionality:

- `trimorph-core update`: Enhanced update checking with parallel package manager support
- `trimorph-core auto-update`: Advanced auto-update with improved dependency tracking
- Supports simultaneous updates across multiple package managers (apt, pacman, etc.)
- More efficient update processing with C-based performance

Example usage:
```bash
# Install htop from the Debian jail to the host
trimorph-install-to-host deb htop

# Check for available updates
trimorph-update-check

# Configure auto-update
trimorph-config set auto_update true
trimorph-config set check_updates true

# Preview installation with dependency information
trimorph-dry-run deb htop

# Check for potential dependency conflicts
trimorph-check-deps deb htop

# List installed packages
ls /var/lib/trimorph/host-installs/

# Set up daily update checking via cron (as root)
# Add this line to root's crontab: crontab -e
0 2 * * * /usr/local/bin/trimorph-cron-check
```

**Important Notes:**
- Installing packages from foreign package managers to the host system bypasses normal package management
- Dependencies are automatically included and may cause conflicts with your host system
- Use `trimorph-dry-run` and `trimorph-check-deps` to preview installations and check for conflicts
- The system automatically checks for updates after installation
- Configure auto-update with caution as it may affect system stability
- This may cause dependency or library conflicts with your host system
- Use with caution and always verify compatibility before installation
- Keep a record of installed files to facilitate cleanup
- Use `trimorph-uninstall-from-host` to remove packages when no longer needed

### 7.5 Local Package Installation
Trimorph now supports installing packages directly from local package files:

- `trimorph-install-local <package_file> [package_file...]`: Install local packages to the host system (supports .deb, .rpm, .pkg.tar.zst, .apk, .tbz, etc.)
- `trimorph-uninstall-local <install_log_file>`: Remove previously installed local packages

Example usage:
```bash
# Install a local .deb package to the host system
trimorph-install-local /path/to/package.deb

# Install an Arch package (like Warp Terminal)
trimorph-install-local /home/gentoo/Downloads/warp-terminal-v0.2025.10.01.08.12.stable_02-1-x86_64.pkg.tar.zst

# Install multiple packages at once
trimorph-install-local /path/to/package1.deb /path/to/package2.rpm

# Uninstall a locally installed package using its log file
trimorph-uninstall-local /var/lib/trimorph/host-installs/local_package_name_timestamp.log

# Check for available updates (includes locally installed packages)
trimorph-update-check
```

### 7.6 Enhanced Local Package Installation (NEW - C-Based)
The new C-based architecture provides enhanced local package installation:

- `trimorph-core install-local <package_file>`: Advanced local package installation with improved format detection
- Enhanced support for complex package formats like `.pkg.tar.zst`
- Better dependency analysis and handling
- Improved error reporting and package content inspection
- Specific optimizations for packages like Warp Terminal

**Features:**
- Automatic package format detection (supports .deb, .rpm, .pkg.tar.zst, .apk, .tbz, .pkg.tar.xz, .pkg.tar.gz)
- Dependency extraction and installation using appropriate Trimorph jails
- File conflict detection and handling
- Automatic desktop database and MIME type updates for applications with URL scheme handlers
- SHA256 checksum verification and basic package integrity checks
- Enhanced dependency resolution with detailed status reporting
- Proper package name/version extraction from all supported formats (fixing previous empty name issues)
- Integration with Trimorph's database and logging system

**Important Notes:**
- For URL scheme handlers (like `warp://`) to work properly, the system automatically registers them after installation, but you may need to restart your desktop environment for changes to take effect
- Package dependencies are automatically identified and installed from appropriate jails when available (with graceful handling when unavailable)
- The system maintains logs of all installed files for proper uninstallation
- Package signature verification is attempted when possible (depending on package format and available tools)

## Utilities

### 8.1 Status & Cleanup
-   `trimorph-status`: Checks and displays the status of all configured jails (active or inactive).
-   `trimorph-cleanup`: Immediately stops all running Trimorph scopes.
-   `trimorph-export <jail> <package> [package...]`: Export packages from a jail for offline use

### 8.2 Enhanced Status & Management (NEW - C-Based)
-   `trimorph-core list`: List all available jails with detailed status information
-   `trimorph-core exec <jail> <command> [args...]`: Execute commands in jails with true parallel support
-   `trimorph-core start <jail>`: Start a jail (parallel execution ready)
-   `trimorph-core stop <jail>`: Stop a jail
-   `trimorph-core update`: Check for updates across all package managers simultaneously
-   `trimorph-core install-local <package_file>`: Enhanced local package installation

### 8.3 Diagnostics & Dry-Run
Trimorph includes flags for validating your setup and simulating commands without making any changes.

-   **`--check`**: Performs a series of environment checks to validate the jail's configuration, ensuring it's bootstrapped and that all required binaries are present.
    ```bash
    trimorph-solo --check <jail>
    ```

-   **`--dry-run`**: Simulates a command execution. It runs all the environment checks and then prints the exact `systemd-run` command that would be executed, but does not run it.
    ```bash
    trimorph-solo --dry-run <jail> <command>
    ```

### 8.3 Terminal UI (TUI)

Trimorph includes a terminal-based user interface for a real-time view of your jails.

      ▲
     ▲ ▲   Trimorph

**To build and run:**
```bash
# Navigate to the TUI directory
cd tui

# Build the TUI (requires Rust)
cargo build --release

# Run the TUI
./target/release/trimorph-tui
```

**Features:**
-   **Live Jail Status**: See which jails are active.
-   **Log Viewer**: Tail logs for any jail in real-time.
-   **Resource Monitoring**: View CPU and Memory usage for active jails.
-   **Diagnostics**: Run `--check` and `--dry-run` on jails directly from the TUI using the 'c' and 'd' keys.
-   **Customizable**: Create a `tui.toml` file in `~/.config/trimorph/` to set custom color themes.

**Global Installation:**
To make the `trimorph-tui` and other `trimorph` commands accessible from any directory, you can create symbolic links to `/usr/local/bin`:
```bash
sudo ln -s /path/to/your/project/usr/local/sbin/trimorph-bootstrap /usr/local/bin/trimorph-bootstrap
sudo ln -s /path/to/your/project/usr/local/sbin/trimorph-parse-conf /usr/local/bin/trimorph-parse-conf
sudo ln -s /path/to/your/project/usr/local/sbin/trimorph-solo /usr/local/bin/trimorph-solo
sudo ln -s /path/to/your/project/tui/target/release/trimorph-tui /usr/local/bin/trimorph-tui
```
Remember to replace `/path/to/your/project` with the actual path to the Trimorph project directory.

### 8.4 Debugging & Logging
Trimorph includes an optional logging feature to help with troubleshooting. When enabled, all output from commands executed within a jail is saved to a log file.

**To enable logging:**
Set the `TRIMORPH_DEBUG` environment variable to any non-empty value.
```bash
export TRIMORPH_DEBUG=1
apt update
```

**Log Location:**
Logs are stored in `/var/log/trimorph/`, with a separate file for each jail (e.g., `/var/log/trimorph/deb.log`). The `/var/log/trimorph` directory is owned by `root`, but the `trimorph-solo` script will create the individual log file and set its ownership to the current user, allowing for secure, user-specific logging.

### 8.5 Cross-Distribution Setup & Installation
Trimorph provides tools to help with installation across different Linux distributions:

- **`install.sh`**: Comprehensive installation script that works across distributions
- **`setup-distro.sh`**: Distribution detection and setup helper
- **`Makefile`**: Standard build and installation system with multiple targets
- **`doc/`**: Complete documentation for cross-distribution usage

## Bootstrap & Usage

```bash
# One-time setup for each jail
sudo trimorph-bootstrap deb
sudo trimorph-bootstrap arch

# Daily use with original system
apt update
pacman -Sy firefox
emerge -avuDN @world # auto-kills foreign scopes
trimorph-status

# Enhanced use with new C-based system
sudo trimorph-core list            # List all jails
sudo trimorph-core exec deb update # Run commands in deb jail
sudo trimorph-core exec arch -Syu  # Run commands in arch jail (can run simultaneously!)
sudo trimorph-core update          # Check updates across all package managers
sudo trimorph-core install-local /path/to/package.pkg.tar.zst  # Enhanced local install

# Cross-distribution package installation
trimorph-validate deb htop          # Verify package exists
trimorph-dry-run deb htop          # Preview installation
trimorph-check-deps deb htop       # Check for dependency conflicts
trimorph-install-to-host deb htop  # Install to host system
trimorph-uninstall-from-host /var/lib/trimorph/host-installs/deb_*.log  # Remove package

# Package export and management
trimorph-export deb htop           # Export packages for offline use
trimorph-update-check             # Check for updates to installed foreign packages
trimorph-config list              # List configuration settings

# New C-based enhanced features
trimorph-core update              # Enhanced update checking with parallel support
trimorph-core install-local <pkg> # Advanced local package handling (e.g., Warp Terminal)
```

---

## Security & Isolation

-   ✅ **No host `/usr` mount** → no ABI conflicts.
-   ✅ **Jails owned by user** → no permission errors.
-   ✅ **No PID files** → systemd is the source of truth.
-   ✅ **No `eval`** → config parsing is injection-safe.
-   ✅ **GPU/X11 works** via selective bind mounts (`/dev/dri`, `/tmp/.X11-unix`).
-   ✅ **Zero idle footprint** — scopes vanish on exit.

## Cross-Distribution Support

**Trimorph** provides robust support across different Linux distributions:

- **Multi-Distro Compatibility**: Works on Gentoo, Debian, Ubuntu, Fedora, Arch, Alpine, and other Linux distributions
- **Init System Detection**: Automatically detects init system (systemd/OpenRC) and uses appropriate sandboxing backend
- **Rust Implementation**: High-performance binaries with memory safety
- **Backend Selection**:
  - On **systemd** hosts: Uses `systemd-nspawn` for containerization
  - On **OpenRC** and other systems: Uses `bubblewrap` for secure sandboxing
- **Distribution-Agnostic Design**: Configuration files work across all distributions
- **Easy Installation**: Simple installation process adaptable to any distribution

For detailed setup instructions on your specific distribution, see: [CROSS_DISTRO_SETUP.md](CROSS_DISTRO_SETUP.md)

## Available Distro Jails

Trimorph supports the following distributions out of the box:

| Distribution | Wrapper Command | Configuration File | Bootstrap Required |
|--------------|-----------------|--------------------|-------------------|
| Debian/Ubuntu | `apt` | `deb.conf` | `sudo trimorph-bootstrap deb` |
| Arch Linux | `pacman` | `arch.conf` | `sudo trimorph-bootstrap arch` |
| Fedora | `dnf` | `fedora.conf` | `sudo trimorph-bootstrap fedora` |
| Alpine | `apk` | `alpine.conf` | `sudo trimorph-bootstrap alpine` |
| openSUSE | `zypper` | `opensuse.conf` | `sudo trimorph-bootstrap opensuse` |
| Gentoo | `emerge` | `gentoo.conf` | `sudo trimorph-bootstrap gentoo` |

## OpenRC Support and Rust Jail Runner

The Rust jail runner provides the cross-platform sandboxing functionality:

- **Automatic Init Detection**: Detects systemd vs OpenRC automatically
- **Secure Sandbox Backends**: Uses appropriate tools for each system
- **Resource Management**: Properly handles overlay filesystems and resource limits
- **Package Manager Integration**: Works with various package managers across distributions

## Conclusion

**Trimorph** delivers safe, modular, on-demand multi-distro package management with no compromises on host purity, security, or resource usage. It works consistently across all Linux distributions including Gentoo, Debian, Ubuntu, Fedora, Arch, Alpine, and others.
> Install once. Run anything. One jail at a time.

---

## Testing

This project uses `bats-core` for its test suite. The tests are located in the `test/` directory.

To run the tests, execute the following command from the project root:
```bash
./test/bats-core/bin/bats test/core.bats
```
A Continuous Integration (CI) workflow is also set up using GitHub Actions to automatically run the test suite on every push.

## Building and Development

Trimorph provides multiple ways to build and install:

### Using Make
```bash
# Build all Rust components
make build

# Install to system
sudo make install

# Install only binaries
sudo make install-bin

# Install only configurations
sudo make install-config

# Clean build artifacts
make clean

# Uninstall Trimorph
sudo make uninstall

# Verify dependencies
make check
```

### Using Installation Script (Recommended)
```bash
chmod +x install.sh
./install.sh
```

The installation script automatically:
- Checks for required dependencies
- Builds Rust components
- Installs all components to the system
- Sets up systemd slice if systemd is available
- Provides distribution-specific advice

### Distribution-Specific Setup
```bash
chmod +x setup-distro.sh
./setup-distro.sh
```

This script detects your Linux distribution and provides specific installation commands and advice.

### Manual Build
```bash
# Build TUI
cd tui && cargo build --release && cd ..

# Build jail runner
cd jail-runner && cargo build --release && cd ..

# Install manually using Make
sudo make install
# Or install manually with direct commands (see INSTALLATION.md)
```

### Documentation

Complete installation and usage documentation is available in the `doc/` directory:
- **INSTALLATION.md**: Detailed installation methods and Makefile targets
- **CROSS_DISTRO_SETUP.md**: Distribution-specific setup instructions
- **getting-started.md**: Step-by-step guide for new users
- **jail-runner.md**: Cross-platform jail runner details
- **COMPONENTS.md**: Complete component documentation
- **scripts.md**: Script-specific documentation
- **troubleshooting.md**: Common issues and solutions

For new users, we recommend starting with the [documentation index](doc/index.md) and [getting-started guide](doc/getting-started.md).