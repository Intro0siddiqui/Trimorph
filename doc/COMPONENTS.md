# Trimorph Component Documentation

This document provides a comprehensive overview of each file/component in the Trimorph project and its specific functionality.

## TUI Components

### `/tui/src/main.rs`
- **Primary Function**: Main entry point for the Trimorph Terminal UI
- **Responsibilities**:
  - Initializes and runs the terminal user interface
  - Handles user input and navigation between jails
  - Manages display of jail status, logs, and resource usage
  - Implements UI rendering using ratatui
  - Integrates with CLI tools via subprocess commands
  - Supports keyboard shortcuts for all Trimorph operations

### `/tui/src/components/log_view.rs`
- **Primary Function**: Renders the log viewing widget
- **Responsibilities**:
  - Displays live logs from selected jail
  - Formats log content for UI display
  - Handles log scrolling and presentation

### `/tui/src/components/resource_monitor.rs`
- **Primary Function**: Displays CPU and Memory usage gauges
- **Responsibilities**:
  - Visualizes resource consumption of active jails
  - Renders CPU and Memory usage as gauge widgets
  - Updates resource metrics in real-time

### `/tui/src/components/status_bar.rs`
- **Primary Function**: Renders the status bar at the bottom of the UI
- **Responsibilities**:
  - Shows current keyboard shortcuts and status information
  - Displays system information and user guidance
  - Provides visual separation between UI sections

### `/tui/src/components/title_bar.rs`
- **Primary Function**: Renders the title and logo area at the top of the UI
- **Responsibilities**:
  - Displays the Trimorph logo and title information
  - Provides branding and visual identity
  - Shows application name and version context

### `/tui/src/config.rs`
- **Primary Function**: Handles TUI configuration and theming
- **Responsibilities**:
  - Loads and manages color themes
  - Defines UI styling options
  - Provides theme serialization/deserialization
  - Handles user configuration preferences

## CLI Tools (usr/local/bin/ and usr/local/sbin/)

### `/usr/local/sbin/trimorph-solo`
- **Primary Function**: Core Trimorph execution engine
- **Responsibilities**:
  - Runs commands inside isolated jail environments
  - Manages overlay filesystems using systemd-nspawn native support
  - Supports `--check`, `--dry-run`, and `--parallel` modes
  - Handles package manager integration
  - Manages bind mounts and system access (X11, DRI, etc.)

### `/usr/local/bin/trimorph-install-to-host`
- **Primary Function**: Install packages from jail to the host system
- **Responsibilities**:
  - Extracts packages from jail environment
  - Installs them to host system with dependency tracking
  - Creates installation manifests for uninstallation
  - Manages file conflict detection
  - Updates dependency database

### `/usr/local/bin/trimorph-uninstall-from-host`
- **Primary Function**: Remove previously installed foreign packages
- **Responsibilities**:
  - Removes files installed by trimorph-install-to-host
  - Cleans up based on installation manifest
  - Updates dependency database

### `/usr/local/bin/trimorph-update-check`
- **Primary Function**: Check for updates to installed foreign packages
- **Responsibilities**:
  - Queries Trimorph package database
  - Checks for available updates to installed packages
  - Provides update status information

### `/usr/local/bin/trimorph-auto-update`
- **Primary Function**: Automated update processing
- **Responsibilities**:
  - Runs automated updates based on configuration
  - Manages update scheduling and execution
  - Handles security-only update options

### `/usr/local/bin/trimorph-config`
- **Primary Function**: Configuration management for Trimorph
- **Responsibilities**:
  - Gets and sets configuration values
  - Manages auto_update, check_updates, and update_security_only settings
  - Handles configuration file creation and updates
  - Provides configuration listing and validation

### `/usr/local/bin/trimorph-dry-run`
- **Primary Function**: Preview package installations
- **Responsibilities**:
  - Shows what files would be installed without installing
  - Provides dependency analysis
  - Simulates installation process safely

### `/usr/local/bin/trimorph-check-deps`
- **Primary Function**: Dependency conflict analysis
- **Responsibilities**:
  - Checks for potential dependency conflicts
  - Analyzes package dependencies for compatibility
  - Provides conflict resolution information

### `/usr/local/bin/trimorph-status`
- **Primary Function**: Check status of all configured jails
- **Responsibilities**:
  - Lists all configured jails and their activation status
  - Shows which jails are currently active
  - Provides quick status overview

### `/usr/local/bin/trimorph-cleanup`
- **Primary Function**: Stop all running Trimorph scopes
- **Responsibilities**:
  - Stops all active Trimorph jail scopes
  - Cleans up active processes quickly
  - Provides emergency cleanup functionality

### `/usr/local/bin/trimorph-export`
- **Primary Function**: Export packages from jail for offline use
- **Responsibilities**:
  - Exports packages and their dependencies from jails
  - Prepares packages for offline distribution
  - Creates export archives

### `/usr/local/bin/trimorph-cron-check`
- **Primary Function**: Cron-based update checking
- **Responsibilities**:
  - Runs update checks from cron jobs
  - Handles periodic update notifications
  - Integrates with system scheduling

### `/usr/local/bin/trimorph-validate`
- **Primary Function**: Validate packages in jail repositories
- **Responsibilities**:
  - Checks if packages exist in specified jail
  - Validates package availability
  - Provides package verification

### `/usr/local/sbin/trimorph-bootstrap`
- **Primary Function**: Initialize new jail environments
- **Responsibilities**:
  - Creates base images for new jails
  - Runs jail-specific bootstrap commands
  - Sets up initial jail configuration
  - Handles different distro initialization methods

### `/usr/local/sbin/trimorph-parse-conf`
- **Primary Function**: Safe configuration file parser
- **Responsibilities**:
  - Parses .conf files without using eval
  - Prevents code injection in config files
  - Safely extracts configuration variables
  - Handles configuration validation

### `/usr/local/sbin/trimorph-install`
- **Primary Function**: Core installation engine
- **Responsibilities**:
  - Handles actual extraction and installation of packages
  - Manages file copying and permissions
  - Creates installation manifests
  - Updates dependency tracking database

### `/usr/local/bin/trimorph-db`
- **Primary Function**: Package database management
- **Responsibilities**:
  - Manages Trimorph package database
  - Handles package metadata storage
  - Provides database query functionality
  - Maintains installed package tracking

## Rust Components

### `/jail-runner/src/main.rs`
- **Primary Function**: Cross-platform jail execution engine
- **Responsibilities**:
  - Automatically detects init system (systemd/OpenRC/Other)
  - Uses systemd-nspawn for systemd systems
  - Uses bubblewrap for OpenRC and other systems
  - Provides consistent jail execution interface
  - Handles different sandboxing backends based on system
  - Improved argument parsing using shell-words crate
  - Enhanced error handling and resource cleanup
  - Support for multiple package managers across distributions

### `/jail-runner/Cargo.toml`
- **Primary Function**: Build configuration for jail-runner
- **Responsibilities**:
  - Defines Rust package metadata
  - Configures dependencies including shell-words crate

### `/tui/Cargo.toml`
- **Primary Function**: Build configuration for TUI
- **Responsibilities**:
  - Defines Rust package metadata for TUI
  - Configures UI dependencies (ratatui, crossterm, etc.)
  - Manages TUI-specific dependencies

## Wrapper Scripts

### `/usr/local/bin/apt`, `/usr/local/bin/pacman`, `/usr/local/bin/dnf`, `/usr/local/bin/apk`, `/usr/local/bin/zypper`, `/usr/local/bin/emerge`
- **Primary Function**: Package manager wrappers
- **Responsibilities**:
  - Redirect package manager commands to trimorph-solo
  - Provide transparent integration with existing workflows
  - Enable using standard package managers with Trimorph jails

## Configuration and Support Files

### `/trimorph` (in project root)
- **Primary Function**: Main entry point launcher
- **Responsibilities**:
  - Launches the Trimorph TUI when executed
  - Dynamically finds TUI binary in multiple locations including PATH
  - Provides appropriate error messages if TUI not found
  - Serves as main user entry point to Trimorph

### `/etc/trimorph/jails.d/*.conf`
- **Primary Function**: Jail configuration files
- **Responsibilities**:
  - Define jail-specific configuration
  - Specify package manager, root directory, mounts, and environment variables
  - Handle different distro-specific settings

### `/install.sh`
- **Primary Function**: Cross-distribution installation script
- **Responsibilities**:
  - Detects system requirements and dependencies
  - Builds Rust components (TUI and jail-runner)
  - Installs all Trimorph binaries and scripts
  - Sets up required directories and configurations
  - Configures systemd slice if systemd is available
  - Provides post-installation instructions

### `/setup-distro.sh`
- **Primary Function**: Distribution detection and setup helper
- **Responsibilities**:
  - Detects the host Linux distribution
  - Provides distribution-specific package installation commands
  - Offers setup guidance for different distributions
  - Helps with PATH configuration

### `/Makefile`
- **Primary Function**: Build and installation automation
- **Responsibilities**:
  - Builds all Rust components (TUI and jail-runner)
  - Installs binaries and configurations to system
  - Uninstalls Trimorph from the system
  - Cleans build artifacts
  - Runs tests

### `/doc/COMPONENTS.md` (this file)
- **Primary Function**: Component documentation
- **Responsibilities**:
  - Provides comprehensive overview of all components
  - Documents each file's specific functionality
  - Serves as reference for new contributors and users

## Testing and Development

### `/test/core.bats`
- **Primary Function**: Core functionality tests
- **Responsibilities**:
  - Tests basic Trimorph functionality
  - Validates command execution
  - Ensures system reliability

This documentation provides a complete overview of each component in the Trimorph project and its role in the overall system architecture.