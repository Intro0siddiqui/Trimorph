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

```
/etc/trimorph/
├── jails.d/
│   ├── deb.conf      → defines root, pkgmgr, bootstrap, mounts
│   └── arch.conf
├── trimorph.slice    → resource limits (CPU, RAM)
└── runtime/          → tmpfs (auto-created)
```

**Workflow:**
1.  User runs `apt install foo`.
2.  The `apt` wrapper calls `trimorph-solo deb apt install foo`.
3.  The launcher stops all other active `trimorph-*.scope` units.
4.  It then starts a new scope with `systemd-run --scope systemd-nspawn ...`.
5.  On command exit, the scope vanishes, leaving **zero residue**.

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

---

## 6. Wrappers & Integration

### 6.1 Package Manager Wrappers
Simple wrappers in `/usr/local/bin` redirect calls for `apt` and `pacman` to the `trimorph-solo` launcher.

### 6.2 Portage Safety Hook (`/etc/portage/bashrc`)
An optional hook that automatically stops any active Trimorph jail before a Portage `emerge` process begins, preventing potential conflicts.

---

## 7. Utilities

### 7.1 Status & Cleanup
-   `trimorph-status`: Checks and displays the status of all configured jails (active or inactive).
-   `trimorph-cleanup`: Immediately stops all running Trimorph scopes.

### 7.2 Debugging & Logging
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

## 10. Conclusion

**Trimorph** delivers safe, modular, on-demand multi-distro package management on Gentoo with no compromises on host purity, security, or resource usage.
> Install once. Run anything. One jail at a time.