# Trimorph Scripts Documentation

This document provides comprehensive information about all Trimorph scripts and their functionality.

## System Scripts (usr/local/sbin/)

- **trimorph-bootstrap** - One-time setup script that prepares a jail's root directory
- **trimorph-solo** - Core script managing jails, ensuring only one is active
- **trimorph-parse-conf** - Safe INI parser for configuration files
- **trimorph-validate** - Check if a package exists in a jail's repository
- **trimorph-install** - Install packages with dependencies in an overlay
- **trimorph-export** - Export packages from a jail for offline use
- **trimorph-init-db** - Initialize the package dependency database

## Utility Scripts (usr/local/bin/)

- **trimorph-install-to-host** - Install packages from a jail to the host system
- **trimorph-uninstall-from-host** - Remove previously installed packages from host
- **trimorph-dry-run** - Preview what would be installed (including dependencies)
- **trimorph-check-deps** - Analyze potential dependency conflicts
- **trimorph-update-check** - Check for available updates to installed foreign packages
- **trimorph-auto-update** - Manually run the auto-update process
- **trimorph-config** - Configure update settings (get/set)
- **trimorph-cron-check** - Run from cron for periodic update checking
- **trimorph-db** - Database management for tracking installed packages
- **trimorph-status** - Check and display status of all configured jails
- **trimorph-cleanup** - Immediately stop all running Trimorph scopes

## Package Manager Wrappers (usr/local/bin/)

- **apt** - Wrapper redirecting to trimorph-solo for Debian-based packages
- **pacman** - Wrapper redirecting to trimorph-solo for Arch-based packages
- **apk** - Wrapper for Alpine packages
- **dnf** - Wrapper for Fedora packages
- **zypper** - Wrapper for openSUSE packages

## Main Interface

- **trimorph** - Entry point that runs the TUI (trimorph-tui)

## Script Relationships

The scripts work together in the following ways:
1. High-level commands like `trimorph-install-to-host` use lower-level commands
2. All configuration reading goes through `trimorph-parse-conf`
3. Most operations interact with jails using `trimorph-solo`
4. Package validation typically uses `trimorph-validate`
5. Database operations are handled by `trimorph-db`
6. Installation tracking is managed by `trimorph-config` and related scripts