# Makefile for Trimorph - Cross-distribution package management

# Default target
.PHONY: all
all: help

.PHONY: help
help:
	@echo "Trimorph Makefile - Cross-distribution package management"
	@echo ""
	@echo "Usage:"
	@echo "  make build          - Build all Rust components"
	@echo "  make install        - Install Trimorph to the system"
	@echo "  make install-bin    - Install only binaries"
	@echo "  make install-config - Install configuration files only"
	@echo "  make uninstall      - Remove Trimorph from the system"
	@echo "  make test           - Run tests"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make distclean      - Clean all including build directories"

.PHONY: build
build: build-tui build-jail-runner build-c-core

.PHONY: build-tui
build-tui:
	@echo "Building TUI..."
	@cd tui && cargo build --release

.PHONY: build-jail-runner
build-jail-runner:
	@echo "Building jail runner..."
	@cd jail-runner && cargo build --release

.PHONY: build-c-core
build-c-core:
	@echo "Building C-based core..."
	@$(MAKE) -f Makefile_c

.PHONY: install
install: install-bin install-config setup-systemd

.PHONY: install-bin
install-bin: build
	@echo "Installing Trimorph binaries..."
	@sudo mkdir -p /usr/local/bin
	@sudo mkdir -p /usr/local/sbin
	@sudo install -m 755 trimorph /usr/local/bin/
	@if [ -f tui/target/release/trimorph-tui ]; then \
		sudo install -m 755 tui/target/release/trimorph-tui /usr/local/bin/; \
	fi
	@if [ -f jail-runner/target/release/jail-runner ]; then \
		sudo install -m 755 jail-runner/target/release/jail-runner /usr/local/bin/; \
	fi
	@if [ -f trimorph-core ]; then \
		sudo install -m 755 trimorph-core /usr/local/sbin/; \
	fi
	@for script in usr/local/sbin/*; do \
		if [ -f "$script" ]; then \
			sudo install -m 755 "$script" /usr/local/sbin/; \
		fi \
	done
	@for script in usr/local/bin/*; do \
		if [ -f "$script" ]; then \
			sudo install -m 755 "$script" /usr/local/bin/; \
		fi \
	done

.PHONY: install-config
install-config:
	@echo "Installing Trimorph configurations..."
	@sudo mkdir -p /etc/trimorph/jails.d
	@sudo mkdir -p /var/cache/trimorph/packages
	@sudo mkdir -p /var/log/trimorph
	@sudo mkdir -p /var/lib/trimorph/host-installs
	@for conf in etc/trimorph/jails.d/*.conf; do \
		if [ -f "$$conf" ]; then \
			sudo install -m 644 "$$conf" /etc/trimorph/jails.d/; \
		fi \
	done

.PHONY: setup-systemd
setup-systemd:
	@if command -v systemctl >/dev/null 2>&1 && [ -d "/run/systemd/system" ]; then \
		echo "Setting up systemd slice..."; \
		echo '[Unit]' | sudo tee /etc/systemd/system/trimorph.slice > /dev/null && \
		echo 'Description=Trimorph Resource Limits' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo 'Before=slices.target' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo '' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo '[Slice]' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo 'CPUQuota=50%' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo 'MemoryMax=120M' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo 'TasksMax=50' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo '' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo '[Install]' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		echo 'WantedBy=slices.target' | sudo tee -a /etc/systemd/system/trimorph.slice >> /dev/null && \
		sudo systemctl daemon-reload; \
	else \
		echo "Systemd not detected, skipping slice setup"; \
	fi

.PHONY: uninstall
uninstall:
	@echo "Uninstalling Trimorph..."
	@sudo rm -f /usr/local/bin/trimorph-tui
	@sudo rm -f /usr/local/bin/jail-runner
	@sudo rm -f /usr/local/bin/trimorph
	@for script in usr/local/bin/*; do \
		if [ -f "$$script" ]; then \
			sudo rm -f "/usr/local/bin/$$(basename "$$script")"; \
		fi \
	done
	@for script in usr/local/sbin/*; do \
		if [ -f "$$script" ]; then \
			sudo rm -f "/usr/local/sbin/$$(basename "$$script")"; \
		fi \
	done
	@sudo rm -rf /etc/trimorph
	@sudo rm -f /etc/systemd/system/trimorph.slice
	@if command -v systemctl >/dev/null 2>&1; then \
		sudo systemctl daemon-reload; \
	fi

.PHONY: test
test:
	@echo "Running tests..."
	@./test/bats-core/bin/bats test/core.bats

.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@cd tui && cargo clean
	@cd jail-runner && cargo clean

.PHONY: distclean
distclean: clean
	@echo "Cleaning all build directories..."
	@rm -rf tui/target
	@rm -rf jail-runner/target

.PHONY: check
check:
	@echo "Checking dependencies..."
	@if ! command -v cargo >/dev/null 2>&1; then \
		echo "Error: cargo not found. Please install Rust."; \
		exit 1; \
	fi
	@if ! command -v sudo >/dev/null 2>&1; then \
		echo "Error: sudo not found."; \
		exit 1; \
	fi
	@echo "All required dependencies found."