# Trimorph Troubleshooting Guide

This guide helps diagnose and resolve common issues with Trimorph.

## Common Issues

### 1. Unbound Variable Errors
**Symptoms**: `line X: 1: unbound variable` errors
**Cause**: Scripts run without required arguments
**Solution**: Ensure you provide all required arguments when calling scripts
- `trimorph-validate <jail> <package>`
- `trimorph-install <jail> <package> [package...]`
- `trimorph-bootstrap <jail>`

### 2. Configuration Directory Not Found
**Symptoms**: "Configuration directory does not exist" errors
**Cause**: `/etc/trimorph/jails.d/` directory is missing
**Solution**: Create the directory:
```bash
sudo mkdir -p /etc/trimorph/jails.d
```

### 3. Missing Dependencies
**Symptoms**: "Missing required dependencies" errors
**Cause**: Required tools (systemd-nspawn, sudo, etc.) are not installed
**Solution**: Install required dependencies:
- systemd with systemd-nspawn
- sudo command
- Appropriate package manager tools for target distros

### 4. Configuration Parse Errors
**Symptoms**: "Config missing required keys" errors
**Cause**: Configuration files missing required fields
**Solution**: Ensure each jail config has:
- `name=` field
- `root=` field (absolute path)
- `bootstrap=` field with `{root}` placeholder
- `pkgmgr=` field
- `pkgmgr_args=` field

### 5. Permission Issues
**Symptoms**: Permission denied errors during bootstrap or installation
**Cause**: Insufficient privileges or incorrect file ownership
**Solution**: 
- Run bootstrap commands with sudo: `sudo trimorph-bootstrap <jail>`
- Ensure user has sudo privileges for required commands
- Verify directory ownership after operations

### 6. TUI Not Working
**Symptoms**: TUI fails to start or crashes
**Cause**: trimorph-tui binary not built or missing dependencies
**Solution**:
1. Build the TUI: `cd tui && cargo build --release`
2. Verify Rust is installed
3. Check that all dependencies are available

## Debugging Tips

### Enable Debug Logging
Set the TRIMORPH_DEBUG environment variable:
```bash
export TRIMORPH_DEBUG=1
apt update  # Logs output to /var/log/trimorph/<jail>.log
```

### Check Jail Status
Use the status command to see which jails are active:
```bash
trimorph-status
```

### Dry Runs
Always use dry-run to preview installations before executing:
```bash
trimorph-dry-run <jail> <package>
```

### Validate Packages
Before installing, verify packages exist:
```bash
trimorph-validate <jail> <package>
```

## Dependency Chain Issues

When installing packages to the host, dependency resolution may cause conflicts. Use:
```bash
trimorph-check-deps <jail> <package>  # Before installation
trimorph-dry-run <jail> <package>     # To see what will be installed
```

## System Integration

### Portage Hook
The `/etc/portage/bashrc` hook automatically stops Trimorph jails during emerge operations. Verify it's properly installed:

```bash
cat /etc/portage/bashrc | grep trimorph
```

### systemd Slice
Ensure the resource limits are properly set in `/etc/systemd/system/trimorph.slice`.

## Configuration Validation

To validate a jail configuration:
1. Ensure the file exists at `/etc/trimorph/jails.d/<jail>.conf`
2. Verify it has all required fields (name, root, bootstrap, pkgmgr, pkgmgr_args)
3. Run `trimorph-solo --check <jail>` to verify the runtime configuration

## Performance Issues

If jails are slow to start:
1. Check disk space in base and cache directories
2. Ensure OverlayFS is properly supported
3. Verify the base image is properly cached

## Update Management

For automatic updates:
1. Enable via config: `trimorph-config set auto_update true`
2. Set up cron job: `0 2 * * * /usr/local/bin/trimorph-cron-check`
3. Monitor with `trimorph-update-check`