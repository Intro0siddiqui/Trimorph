#!/bin/bash
set -euo pipefail

# Distribution detection and setup script for Trimorph
# Automatically detects the host distribution and sets up appropriate configurations

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

detect_distribution() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/redhat-release ]; then
        echo "rhel"
    elif [ -f /etc/gentoo-release ]; then
        echo "gentoo"
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    elif [ -f /etc/alpine-release ]; then
        echo "alpine"
    else
        echo "unknown"
    fi
}

get_pkg_commands() {
    local distro=$1
    
    case $distro in
        "debian"|"ubuntu")
            echo "sudo apt install -y debootstrap systemd-container rust"
            ;;
        "fedora")
            echo "sudo dnf install -y debootstrap systemd-container rust-toolset bubblewrap"
            ;;
        "rhel"|"centos"|"rocky"|"almalinux")
            echo "sudo yum install -y debootstrap systemd-container rust bubblewrap"
            ;;
        "arch"|"manjaro")
            echo "sudo pacman -S --noconfirm debootstrap arch-install-scripts systemd rust bubblewrap"
            ;;
        "alpine")
            echo "sudo apk add debootstrap systemd rust bubblewrap"
            ;;
        "gentoo")
            echo "sudo emerge -av app-arch/dpkg dev-util/apt app-arch/pacman app-arch/debootstrap sys-apps/systemd dev-lang/rust"
            ;;
        *)
            echo "# Install the following packages for your distribution:"
            echo "# - debootstrap or equivalent for your distro"
            echo "# - systemd-container or systemd-nspawn"
            echo "# - rust and cargo"
            echo "# - bubblewrap (for OpenRC support)"
            ;;
    esac
}

setup_paths() {
    # Ensure trimorph scripts are in PATH
    if ! command -v trimorph >/dev/null 2>&1; then
        echo "Trimorph is not in PATH. You may need to add /usr/local/bin to your PATH"
        echo "Add this to your shell configuration file (e.g., ~/.bashrc):"
        echo "export PATH=\$PATH:/usr/local/bin"
    fi
}

show_summary() {
    local distro=$1
    
    echo
    echo "===== TRIMORPH DISTRIBUTION SETUP SUMMARY ====="
    echo "Host Distribution: $distro"
    echo
    echo "Required packages to install:"
    get_pkg_commands "$distro"
    echo
    echo "After installing dependencies, build and install Trimorph:"
    echo "cd $SCRIPT_DIR && make build && sudo make install"
    echo
    echo "Then bootstrap your preferred jails:"
    echo "sudo trimorph-bootstrap deb      # Debian"
    echo "sudo trimorph-bootstrap arch     # Arch Linux"
    echo "sudo trimorph-bootstrap fedora   # Fedora"
    echo "sudo trimorph-bootstrap alpine   # Alpine"
    echo
    echo "Use the wrapper commands:"
    echo "apt update && apt install htop    # Uses Debian jail"
    echo "pacman -Sy htop                   # Uses Arch jail"
    echo "dnf install htop                  # Uses Fedora jail"
    echo "apk add htop                      # Uses Alpine jail"
    echo
    echo "Or launch the TUI:"
    echo "trimorph"
    echo "==============================================="
}

# Main execution
main() {
    local distro
    distro=$(detect_distribution)
    
    echo "Detected distribution: $distro"
    
    show_summary "$distro"
    
    # Provide specific advice based on distribution
    case $distro in
        "gentoo")
            echo
            echo "Note for Gentoo users:"
            echo "- You already have the concept of package management isolation"
            echo "- Trimorph provides similar functionality for foreign package managers"
            echo "- Make sure systemd is properly configured if using systemd-nspawn backend"
            ;;
        "debian"|"ubuntu")
            echo
            echo "Note for Debian/Ubuntu users:"
            echo "- You'll need to install debootstrap for Debian jail support"
            echo "- The systemd-nspawn backend should work out of the box"
            ;;
        "arch"|"manjaro")
            echo
            echo "Note for Arch users:"
            echo "- You'll need arch-install-scripts for Arch jail support"
            echo "- The systemd-nspawn backend should work out of the box"
            ;;
        "fedora"|"rhel"|"centos"|"rocky"|"almalinux")
            echo
            echo "Note for Red Hat family users:"
            echo "- You'll need dnf/yum for Fedora jail support"
            echo "- The systemd-nspawn backend should work out of the box"
            ;;
        "alpine")
            echo
            echo "Note for Alpine users:"
            echo "- Alpine is lightweight and perfect for Trimorph's approach"
            echo "- You may need to ensure cgroups are properly configured for systemd-nspawn"
            ;;
    esac
}

main "$@"