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
```bash
sudo emerge -av app-arch/dpkg dev-util/apt app-arch/pacman app-arch/debootstrap sys-apps/systemd
```

### 3.2 Directory Setup
```bash
sudo mkdir -p /etc/trimorph/{jails.d,runtime} /var/log/trimorph
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

## 10. OpenRC Support and Rust Jail Runner

**Trimorph** now includes experimental support for OpenRC-based systems through a new Rust-based jail runner:

- **Cross-Platform Compatibility**: Works on both systemd and OpenRC hosts
- **Rust Implementation**: High-performance binary with memory safety
- **Automatic Detection**: Automatically detects init system and uses appropriate sandboxing backend
- **Backend Selection**:
  - On **systemd** hosts: Uses `systemd-nspawn` (traditional approach)
  - On **OpenRC** hosts: Uses `bubblewrap` for secure sandboxing

## 11. Conclusion

**Trimorph** delivers safe, modular, on-demand multi-distro package management on Gentoo with no compromises on host purity, security, or resource usage.
> Install once. Run anything. One jail at a time.

---

## 12. Testing

This project uses `bats-core` for its test suite. The tests are located in the `test/` directory.

To run the tests, execute the following command from the project root:
```bash
./test/bats-core/bin/bats test/core.bats
```
A Continuous Integration (CI) workflow is also set up using GitHub Actions to automatically run the test suite on every push.