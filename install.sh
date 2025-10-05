#!/bin/bash
set -euo pipefail

# Trimorph installation script - works across all Linux distributions
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if running as root for some operations
is_root() {
    [[ $EUID -eq 0 ]]
}

# Print colored output
print_info() {
    echo -e "\033[1;34m[i] $1\033[0m"
}

print_success() {
    echo -e "\033[1;32m[✓] $1\033[0m"
}

print_warning() {
    echo -e "\033[1;33m[!] $1\033[0m"
}

print_error() {
    echo -e "\033[1;31m[✗] $1\033[0m" >&2
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    for cmd in bash sudo systemd-nspawn; do
        if ! command_exists "$cmd"; then
            missing_deps+=("$cmd")
        fi
    done
    
    # For building Rust components
    if ! command_exists cargo || ! command_exists rustc; then
        missing_deps+=("rust (cargo and rustc)")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing required dependencies:"
        for dep in "${missing_deps[@]}"; do
            print_error "  - $dep"
        done
        print_warning "Please install these dependencies before running this script."
        exit 1
    else
        print_success "All required dependencies are available."
    fi
}

# Function to build Rust components
build_rust_components() {
    print_info "Building Rust components..."
    
    # Build the TUI
    if [ -d "$PROJECT_ROOT/tui" ]; then
        cd "$PROJECT_ROOT/tui"
        cargo build --release
        print_success "TUI built successfully."
    fi
    
    # Build the jail runner
    if [ -d "$PROJECT_ROOT/jail-runner" ]; then
        cd "$PROJECT_ROOT/jail-runner"
        cargo build --release
        print_success "Jail runner built successfully."
    fi
    
    cd "$PROJECT_ROOT"
}

# Function to install files
install_files() {
    print_info "Installing Trimorph files..."
    
    # Create directories
    sudo mkdir -p /usr/local/bin
    sudo mkdir -p /usr/local/sbin
    sudo mkdir -p /etc/trimorph/jails.d
    sudo mkdir -p /var/cache/trimorph/packages
    sudo mkdir -p /var/log/trimorph
    sudo mkdir -p /var/lib/trimorph/host-installs
    
    # Install scripts from usr/local directories
    for script in "$PROJECT_ROOT/usr/local/sbin"/*; do
        if [[ -f "$script" ]]; then
            sudo install -m 755 "$script" /usr/local/sbin/
            print_info "Installed $(basename "$script") to /usr/local/sbin"
        fi
    done
    
    for script in "$PROJECT_ROOT/usr/local/bin"/*; do
        if [[ -f "$script" ]]; then
            sudo install -m 755 "$script" /usr/local/bin/
            print_info "Installed $(basename "$script") to /usr/local/bin"
        fi
    done
    
    # Install the main trimorph entry point
    sudo install -m 755 "$PROJECT_ROOT/trimorph" /usr/local/bin/
    print_info "Installed trimorph to /usr/local/bin"
    
    # Install jail configurations
    for conf in "$PROJECT_ROOT/etc/trimorph/jails.d"/*.conf; do
        if [[ -f "$conf" ]]; then
            sudo install -m 644 "$conf" /etc/trimorph/jails.d/
            print_info "Installed $(basename "$conf") to /etc/trimorph/jails.d"
        fi
    done
    
    print_success "All files installed successfully."
}

# Function to set up systemd slice (if systemd is available)
setup_systemd_slice() {
    if command_exists systemctl && [ -d "/run/systemd/system" ]; then
        print_info "Setting up systemd slice..."
        
        cat << 'EOF' | sudo tee /etc/systemd/system/trimorph.slice > /dev/null
[Unit]
Description=Trimorph Resource Limits
Before=slices.target

[Slice]
CPUQuota=50%
MemoryMax=120M
TasksMax=50

[Install]
WantedBy=slices.target
EOF
        
        sudo systemctl daemon-reload
        print_success "Systemd slice created and daemon reloaded."
    else
        print_warning "systemd not detected, skipping systemd slice setup."
        print_info "Trimorph will still work but without resource limits."
    fi
}

# Function to check for distribution-specific package manager dependencies
check_pkgmgr_deps() {
    print_info "Checking for package manager dependencies..."
    
    # On Debian/Ubuntu, we need debootstrap
    if [ -f /etc/debian_version ]; then
        if ! command_exists debootstrap; then
            print_warning "debootstrap not found. Install with: sudo apt install debootstrap"
        fi
    fi
    
    # On Arch, we need pacstrap (from arch-install-scripts)
    if [ -f /etc/arch-release ]; then
        if ! command_exists pacstrap; then
            print_warning "pacstrap not found. Install with: sudo pacman -S arch-install-scripts"
        fi
    fi
    
    # On RHEL/Fedora, we need dnf/yum
    if [ -f /etc/redhat-release ] || [ -f /etc/fedora-release ]; then
        if ! command_exists dnf && ! command_exists yum; then
            print_warning "dnf or yum not found. Install with your distribution's package manager."
        fi
    fi
    
    # On Alpine, we need apk (should already be present in Alpine)
    if [ -f /etc/alpine-release ]; then
        if ! command_exists apk; then
            print_warning "apk not found on Alpine system."
        fi
    fi
}

# Function to show post-installation instructions
show_post_install() {
    cat << EOF

Installation completed successfully!

Next steps:
1. If you haven't already, build the Rust components:
   cd "$PROJECT_ROOT/tui" && cargo build --release
   cd "$PROJECT_ROOT/jail-runner" && cargo build --release

2. Bootstrap your desired jails:
   sudo trimorph-bootstrap deb        # for Debian
   sudo trimorph-bootstrap arch       # for Arch Linux
   sudo trimorph-bootstrap fedora     # for Fedora
   sudo trimorph-bootstrap alpine     # for Alpine

3. Use the wrapper commands:
   apt update                         # runs in Debian jail
   pacman -Sy                         # runs in Arch jail
   dnf check-update                   # runs in Fedora jail
   apk update                         # runs in Alpine jail

4. Or use the TUI:
   trimorph                           # launches the terminal UI

For more information, see: $PROJECT_ROOT/README.md

EOF
}

# Main execution
main() {
    print_info "Starting Trimorph installation..."
    
    check_dependencies
    build_rust_components
    install_files
    setup_systemd_slice
    check_pkgmgr_deps
    show_post_install
    
    print_success "Trimorph installation completed!"
}

# Run main function
main "$@"