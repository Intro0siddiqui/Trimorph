# Jail Runner

The Jail Runner is a Rust-based binary that provides cross-platform support for Trimorph, allowing it to work with both systemd and OpenRC init systems.

## Features

- **Init System Detection**: Automatically detects whether the host system uses systemd or OpenRC
- **Dual Backend Support**: 
  - On systemd systems: Uses `systemd-nspawn` for containerization
  - On OpenRC systems: Uses `bubblewrap` for sandboxing
- **Jail Configuration**: Reads jail configuration from `/etc/trimorph/jails.d/` directory
- **Package Manager Integration**: Supports various package managers with appropriate caching

## Usage

```bash
jail-runner <jail_name> <command> [args...]
```

## Implementation Details

The jail runner implements both backends:

1. **systemd-nspawn Backend**:
   - Uses OverlayFS for ephemeral containers
   - Provides hardware access (GPU, X11) through bind mounts
   - Maintains package manager cache integration

2. **bubblewrap Backend**:
   - Provides sandboxing using Linux namespaces
   - Secure isolation for OpenRC systems
   - Mounts necessary directories for functionality

## Configuration

The jail runner reads configuration from `/etc/trimorph/jails.d/{jail_name}.conf` files, supporting the same format as the original trimorph-solo implementation.