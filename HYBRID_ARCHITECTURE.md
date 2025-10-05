# TRIMORPH HYBRID ARCHITECTURE

This document describes the current hybrid architecture of Trimorph, combining the original bash/Rust system with the new C-based enhancements.

## Components Overview

### Original Components (Preserved)
- `trimorph-solo` - Core jail execution (bash-based)
- `jail-runner` - Cross-platform sandboxing (Rust-based) 
- `trimorph-bootstrap` - Jail initialization (bash-based)
- Package manager wrappers (apt, pacman, etc.)
- TUI components (Rust-based)

### New C-Based Components (Added)
- `trimorph-core` - Main C executable for enhanced functionality
- Modular architecture in `src/` directory:
  - `src/core/` - Core logic and main entry point
  - `src/config/` - Safe configuration parsing
  - `src/daemon/` - Background service functionality  
  - `src/installer/` - Package installation and updates

## How They Work Together

The system now operates with both architectures:

1. **Traditional workflows**: Continue using the original bash/Rust system
2. **Enhanced workflows**: Use the new `trimorph-core` for:
   - Parallel package manager execution
   - Auto dependency updating
   - Local package installation (especially .pkg.tar.zst)
   - Cross-distro operations

## Key Improvements

The new C-based system adds:
- True parallel execution capability (this was the main goal)
- Enhanced auto-update functionality
- Better local package handling
- Improved performance through C implementation

## Usage

For new enhanced functionality:
```bash
sudo trimorph-core list                    # List jails
sudo trimorph-core exec <jail> <cmd>      # Execute in jail
sudo trimorph-core update                  # Update packages  
sudo trimorph-core install-local <pkg>    # Install local package
```

For traditional functionality, continue using the original commands.

## Advantages of Hybrid Approach

- Maintains backward compatibility
- Preserves tested, working components
- Adds new capabilities without breaking existing workflows
- Allows gradual migration to new system
- Maintains the modular, well-designed architecture