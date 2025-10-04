# Trimorph Documentation Index

Welcome to the Trimorph documentation. This directory contains all the project documentation organized for easy reference.

## Core Documentation

1. [README.md](README.md) - Main project documentation containing:
   - Executive Summary
   - Architecture Overview  
   - Installation instructions
   - Core utilities documentation
   - Cross-distribution package management features
   - Usage examples

2. [jail-runner.md](jail-runner.md) - Rust-based cross-platform jail runner documentation containing:
   - Init system detection capabilities
   - Dual backend support (systemd-nspawn and bubblewrap)
   - Usage instructions

## Configuration Examples

3. [config-examples/](config-examples/) - Sample configuration files for different distributions:
   - [alpine.conf](config-examples/alpine.conf) - Alpine Linux configuration
   - [arch.conf](config-examples/arch.conf) - Arch Linux configuration  
   - [deb.conf](config-examples/deb.conf) - Debian configuration
   - [fedora.conf](config-examples/fedora.conf) - Fedora configuration
   - [opensuse.conf](config-examples/opensuse.conf) - openSUSE configuration

## Quick Start

To get started with Trimorph, begin by reading the [README.md](README.md) file which contains the complete guide to installation, configuration, and usage.

For system integration, refer to the configuration examples in the [config-examples/](config-examples/) directory to create your own jail configurations.

For cross-platform support, see the [jail-runner.md](jail-runner.md) documentation.