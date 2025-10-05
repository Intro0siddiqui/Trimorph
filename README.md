# **Trimorph: Modular Multi-Distro Package Management on Gentoo**

---

## ✅ Key Features
- **No PID files** → uses `systemctl` unit state as source of truth.
- **No `eval`** → safe config parsing via associative arrays.
- **No nested scopes** → `systemd-nspawn` runs **directly under `systemd-run`**.
- **No host `/usr` bind-mount** → only GPU/X11/IPC paths are shared.
- **User-writable jails** → bootstrap ensures correct ownership.
- **Initialization guard** → prevents use of unbootstrapped jails.

---

## 1. Executive Summary

Trimorph enables on-demand use of foreign package managers (e.g., `apt`, `pacman`) on a pure Gentoo host, with **only one distro jail active at a time**. It uses **transient systemd scopes** for zero idle overhead (< 30 MB RAM, 0% CPU when idle) and is ideal for netbooks, SBCs, and embedded systems.

New distros are added via **drop-in `.conf` files**—no code changes required.

---

## 2. Architecture Overview

Trimorph now uses a layered, caching architecture to provide fast, ephemeral, and isolated environments.

```
/usr/local/trimorph/
├── base/
│   ├── deb/          → Read-only base image for Debian
│   └── arch/         → Read-only base image for Arch
└── deb/              → Ephemeral OverlayFS mount point for live jail
/var/cache/trimorph/
└── packages/         → Shared package cache for all jails
```

### 2.1 Smart Caching Workflow
1.  **Bootstrap:** `trimorph-bootstrap` creates a read-only base image in `/usr/local/trimorph/base/`.
2.  **Execution:** `trimorph-solo` uses `systemd-nspawn`'s native `--overlay` support to create a temporary, writable filesystem layer on top of the read-only base.
3.  **Launch:** The command runs inside this ephemeral overlay.
4.  **Cleanup:** On exit, the overlay's writable layer is automatically discarded, leaving the base image untouched.

This architecture provides:
-   **Speed:** Jails start almost instantly, as there's no need to copy a root filesystem.
-   **Efficiency:** A shared package cache (`/var/cache/trimorph/packages`) minimizes redundant downloads.
-   **Purity:** The base images remain clean, and all changes are temporary.

---

## 3. Installation

### 3.1 Host Dependencies

Trimorph requires different dependencies depending on your host distribution. Install the relevant packages:

**Gentoo:**
```bash
sudo emerge -av app-arch/dpkg dev-util/apt app-arch/pacman app-arch/debootstrap sys-apps/systemd rust cargo
```

**Debian/Ubuntu:**
```bash
sudo apt install debootstrap systemd-container systemd-sysv rustc cargo bubblewrap
```

**Fedora/RHEL:**
```bash
sudo dnf install debootstrap systemd-container systemd-rpm-macros rust cargo bubblewrap
# or for RHEL/CentOS:
sudo yum install debootstrap systemd-container rust cargo bubblewrap
```

**Arch Linux:**
```bash
sudo pacman -S debootstrap arch-install-scripts systemd rust bubblewrap
```

**Alpine:**
```bash
sudo apk add debootstrap systemd rust bubblewrap
```

### 3.2 Quick Installation

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

### 3.3 Manual Installation

If you prefer manual installation:

```bash
# Build the Rust components
cd tui && cargo build --release
cd ../jail-runner && cargo build --release
cd ..

# Install all components
sudo make install
```

### 3.4 Directory Setup (if installing manually)

```bash
sudo mkdir -p /etc/trimorph/{jails.d,runtime} /var/log/trimorph /var/cache/trimorph/packages /var/lib/trimorph/host-installs
sudo mkdir -p /usr/local/{bin,sbin}
```

### 3.3 Systemd Slice (Resource Caps)
Create `/etc/systemd/system/trimorph.slice` with the following content:
```ini
[Slice]
CPUQuota=50%
MemoryMax=120M
TasksMax=50
```
Then, reload the systemd daemon:
```bash
sudo systemctl daemon-reload
```

---

## 4. Core Utilities

This section provides an overview of the scripts that power Trimorph.

### 4.1 Safe Config Parser (`/usr/local/sbin/trimorph-parse-conf`)
A safe INI parser that reads `.conf` files without using `eval`, preventing code injection.

### 4.2 Solo Launcher (`/usr/local/sbin/trimorph-solo`)
The core script that manages jails. It ensures only one jail is active at a time, loads the correct configuration, and launches the package manager inside a transient `systemd-nspawn` container.

### 4.3 Bootstrap Tool (`/usr/local/sbin/trimorph-bootstrap`)
A one-time setup script that prepares a jail's root directory and runs the bootstrap command defined in its `.conf` file.

### 4.4 Component Overview
This section details each file's specific function in the Trimorph system:

#### TUI Components
- **`/tui/src/main.rs`**: Main entry point for the Terminal UI, handles UI rendering, user input, and integration with CLI tools
- **`/tui/src/components/log_view.rs`**: Renders live jail logs in the UI
- **`/tui/src/components/resource_monitor.rs`**: Displays CPU and Memory usage gauges
- **`/tui/src/components/status_bar.rs`**: Shows status information and keyboard shortcuts
- **`/tui/src/components/title_bar.rs`**: Displays the Trimorph logo and title
- **`/tui/src/config.rs`**: Handles TUI configuration and theming

#### Core CLI Tools
- **`/usr/local/sbin/trimorph-solo`**: Core execution engine for running commands in isolated jails
- **`/usr/local/sbin/trimorph-parse-conf`**: Safe configuration file parser (no eval usage)
- **`/usr/local/sbin/trimorph-bootstrap`**: Jail initialization and setup tool
- **`/usr/local/sbin/trimorph-install`**: Core installation engine for packages

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

#### Cross-Platform Support
- **`/jail-runner/src/main.rs`**: Rust-based jail runner that automatically detects init system (systemd/OpenRC), uses systemd-nspawn for systemd systems and bubblewrap for OpenRC systems
- **`/jail-runner/Cargo.toml`**: Build configuration for the cross-platform jail runner
- **`/jail-runner/target/release/jail-runner`**: Built binary that provides consistent jail execution interface across different init systems

#### Wrapper Scripts
- **`/usr/local/bin/{apt,pacman,dnf,apk,zypper}`**: Package manager wrappers that redirect to trimorph-solo

#### Main Entry Points
- **`/trimorph`**: Project root entry point that launches the TUI when executed
- **`/tui/Cargo.toml`**: Build configuration for the Terminal UI

---


## 5. Default Jail Configurations

### 5.1 Debian (`/etc/trimorph/jails.d/deb.conf`)
```ini
name=deb
root=/usr/local/trimorph/deb
bootstrap=debootstrap --variant=minbase bookworm {root} http://deb.debian.org/debian
pkgmgr=/usr/bin/apt
pkgmgr_args=-o Root={root} -o Dir::State::status={root}/var/lib/dpkg/status -o Dir::Cache::Archives={root}/var/cache/apt/archives
mounts=/run/user/$(id -u)/bus
env=DEBIAN_FRONTEND=noninteractive,HOME=/tmp
```

### 5.2 Arch (`/etc/trimorph/jails.d/arch.conf`)
```ini
name=arch
root=/usr/local/trimorph/arch
bootstrap=curl -L https://geo.mirror.pkgbuild.com/iso/latest/archlinux-bootstrap-x86_64.tar.gz | tar -xz -C {root} --strip=1 && systemd-nspawn -q -D {root} sh -c "pacman-key --init && pacman-key --populate archlinux"
pkgmgr=/usr/bin/pacman
pkgmgr_args=--root {root} --dbpath {root}/var/lib/pacman --cachedir {root}/var/cache/pacman/pkg
env=LANG=C.UTF-8,HOME=/tmp
```

### 5.3 Expanded Support
Trimorph also includes configurations for other popular package managers:
-   **Fedora:** `etc/trimorph/jails.d/fedora.conf`
-   **Alpine:** `etc/trimorph/jails.d/alpine.conf`
-   **openSUSE:** `etc/trimorph/jails.d/opensuse.conf`

To use them, simply bootstrap the desired jail (e.g., `sudo trimorph-bootstrap fedora`) and then use the corresponding wrapper (`dnf`, `apk`, `zypper`).

---

## 6. Wrappers & Integration

### 6.1 Package Manager Wrappers
Simple wrappers in `/usr/local/bin` redirect calls for `apt` and `pacman` to the `trimorph-solo` launcher.

### 6.2 Portage Safety Hook (`/etc/portage/bashrc`)
An optional hook that automatically stops any active Trimorph jail before a Portage `emerge` process begins, preventing potential conflicts.

### 6.3 Cross-Distribution Package Installation
Trimorph now supports installing packages from foreign package managers directly to the host system using the new installation tools:

- `trimorph-install-to-host <jail> <package> [package...]`: Install packages from a jail to the host system
- `trimorph-uninstall-from-host <log_file>`: Remove previously installed packages
- `trimorph-validate <jail> <package>`: Check if a package exists in a jail's repository
- `trimorph-dry-run <jail> <package> [package...]`: Preview what would be installed (including dependencies)
- `trimorph-check-deps <jail> <package>`: Analyze potential dependency conflicts

### 6.4 Automatic Dependency Update Management
Trimorph now includes automatic dependency checking and updating functionality:

- `trimorph-update-check`: Check for available updates to installed foreign packages
- `trimorph-auto-update`: Manually run the auto-update process
- `trimorph-config {get|set} <setting> [value]`: Configure update settings
- `trimorph-cron-check`: Run from cron for periodic update checking

Configuration options:
- `auto_update`: Enable/disable automatic updates (default: false)
- `check_updates`: Enable/disable update checking (default: true)
- `update_security_only`: Only update security-related packages (default: false)

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

### 6.5 Local Package Installation
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

**Features:**
- Automatic package format detection (supports .deb, .rpm, .pkg.tar.zst, .apk, .tbz, .pkg.tar.xz, .pkg.tar.gz)
- Dependency extraction and installation using appropriate Trimorph jails
- File conflict detection and handling
- Automatic desktop database updates for applications with URL scheme handlers
- SHA256 checksum verification
- Integration with Trimorph's database and logging system

**Important Notes:**
- For URL scheme handlers (like `warp://`) to work properly, you may need to restart your desktop environment or run `sudo update-desktop-database /usr/share/applications`
- Package dependencies are automatically identified and can be installed from appropriate jails if available
- The system maintains logs of all installed files for proper uninstallation

---

## 7. Utilities

### 7.1 Status & Cleanup
-   `trimorph-status`: Checks and displays the status of all configured jails (active or inactive).
-   `trimorph-cleanup`: Immediately stops all running Trimorph scopes.
-   `trimorph-export <jail> <package> [package...]`: Export packages from a jail for offline use

### 7.2 Diagnostics & Dry-Run
Trimorph includes flags for validating your setup and simulating commands without making any changes.

-   **`--check`**: Performs a series of environment checks to validate the jail's configuration, ensuring it's bootstrapped and that all required binaries are present.
    ```bash
    trimorph-solo --check <jail>
    ```

-   **`--dry-run`**: Simulates a command execution. It runs all the environment checks and then prints the exact `systemd-run` command that would be executed, but does not run it.
    ```bash
    trimorph-solo --dry-run <jail> <command>
    ```

### 7.3 Terminal UI (TUI)

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

### 7.4 Debugging & Logging
Trimorph includes an optional logging feature to help with troubleshooting. When enabled, all output from commands executed within a jail is saved to a log file.

**To enable logging:**
Set the `TRIMORPH_DEBUG` environment variable to any non-empty value.
```bash
export TRIMORPH_DEBUG=1
apt update
```

**Log Location:**
Logs are stored in `/var/log/trimorph/`, with a separate file for each jail (e.g., `/var/log/trimorph/deb.log`). The `/var/log/trimorph` directory is owned by `root`, but the `trimorph-solo` script will create the individual log file and set its ownership to the current user, allowing for secure, user-specific logging.

---

## 8. Bootstrap & Usage

```bash
# One-time setup for each jail
sudo trimorph-bootstrap deb
sudo trimorph-bootstrap arch

# Daily use
apt update
pacman -Sy firefox
emerge -avuDN @world # auto-kills foreign scopes
trimorph-status

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
```

---

## 9. Security & Isolation

-   ✅ **No host `/usr` mount** → no ABI conflicts.
-   ✅ **Jails owned by user** → no permission errors.
-   ✅ **No PID files** → systemd is the source of truth.
-   ✅ **No `eval`** → config parsing is injection-safe.
-   ✅ **GPU/X11 works** via selective bind mounts (`/dev/dri`, `/tmp/.X11-unix`).
-   ✅ **Zero idle footprint** — scopes vanish on exit.

---

## 10. Cross-Distribution Support

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

## 11. Available Distro Jails

Trimorph supports the following distributions out of the box:

| Distribution | Wrapper Command | Configuration File | Bootstrap Required |
|--------------|-----------------|--------------------|-------------------|
| Debian/Ubuntu | `apt` | `deb.conf` | `sudo trimorph-bootstrap deb` |
| Arch Linux | `pacman` | `arch.conf` | `sudo trimorph-bootstrap arch` |
| Fedora | `dnf` | `fedora.conf` | `sudo trimorph-bootstrap fedora` |
| Alpine | `apk` | `alpine.conf` | `sudo trimorph-bootstrap alpine` |
| openSUSE | `zypper` | `opensuse.conf` | `sudo trimorph-bootstrap opensuse` |

## 12. OpenRC Support and Rust Jail Runner

The Rust jail runner provides the cross-platform sandboxing functionality:

- **Automatic Init Detection**: Detects systemd vs OpenRC automatically
- **Secure Sandbox Backends**: Uses appropriate tools for each system
- **Resource Management**: Properly handles overlay filesystems and resource limits
- **Package Manager Integration**: Works with various package managers across distributions

## 13. Conclusion

**Trimorph** delivers safe, modular, on-demand multi-distro package management with no compromises on host purity, security, or resource usage. It works consistently across all Linux distributions.
> Install once. Run anything. One jail at a time.

---


## 14. Testing

This project uses `bats-core` for its test suite. The tests are located in the `test/` directory.

To run the tests, execute the following command from the project root:
```bash
./test/bats-core/bin/bats test/core.bats
```
A Continuous Integration (CI) workflow is also set up using GitHub Actions to automatically run the test suite on every push.

## 15. Building and Development

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
```

### Using Installation Script
```bash
chmod +x install.sh
./install.sh
```

### Manual Build
```bash
# Build TUI
cd tui && cargo build --release && cd ..

# Build jail runner
cd jail-runner && cargo build --release && cd ..

# Install manually
sudo make install
```