# Makefile for building marv
#
# This Makefile builds marv for the current platform.
# For cross-platform builds, use GitHub Actions.

# Configuration
BINARY_NAME = marv
BUILD_MODE = release
OUTPUT_DIR = bin
RUST_DIR = rust

# Detect operating system
OS := $(shell uname -s)
ARCH := $(shell uname -m)

# Set platform-specific values
ifeq ($(OS),Linux)
  PLATFORM_NAME = linux
  ifeq ($(ARCH),x86_64)
    PLATFORM_ARCH = amd64
    RUST_TARGET = x86_64-unknown-linux-gnu
  else ifeq ($(ARCH),aarch64)
    PLATFORM_ARCH = arm64
    RUST_TARGET = aarch64-unknown-linux-gnu
  endif
else ifeq ($(OS),Darwin)
  PLATFORM_NAME = darwin
  ifeq ($(ARCH),x86_64)
    PLATFORM_ARCH = amd64
    RUST_TARGET = x86_64-apple-darwin
  else ifeq ($(ARCH),arm64)
    PLATFORM_ARCH = arm64
    RUST_TARGET = aarch64-apple-darwin
  endif
endif

# Output binary path
OUTPUT_BINARY = $(OUTPUT_DIR)/$(BINARY_NAME)-$(PLATFORM_NAME)-$(PLATFORM_ARCH)

.PHONY: clean build dev dev.start dev.stop dev.tools lint lint.rust lint.lua help

# Build for current platform
build: dev.tools
	@if [ -z "$(PLATFORM_NAME)" ] || [ -z "$(PLATFORM_ARCH)" ]; then \
		echo "Unsupported platform: $(OS) $(ARCH)"; \
		exit 1; \
	fi
	@echo "Building for $(PLATFORM_NAME) $(PLATFORM_ARCH)..."
	@cd $(RUST_DIR) && cargo build --$(BUILD_MODE)
	@mkdir -p $(OUTPUT_DIR)
	@cp $(RUST_DIR)/target/$(BUILD_MODE)/$(BINARY_NAME) $(OUTPUT_BINARY)
	@chmod +x $(OUTPUT_BINARY)
	@echo "Build complete: $(OUTPUT_BINARY)"

# Use 'make build' to build for your current platform

# Check and install development tools
dev.tools:
	@echo "Checking and installing development tools..."
	@mkdir -p $(OUTPUT_DIR)
	
	@# No need for cross anymore
	
	@# Check for rustfmt and clippy
	@if ! rustup component list | grep installed | grep -q rustfmt; then \
		echo "Installing rustfmt..."; \
		rustup component add rustfmt; \
	else \
		echo "rustfmt is already installed"; \
	fi
	
	@if ! rustup component list | grep installed | grep -q clippy; then \
		echo "Installing clippy..."; \
		rustup component add clippy; \
	else \
		echo "clippy is already installed"; \
	fi
	
	@# Check for luarocks and luacheck
	@if ! command -v luarocks >/dev/null 2>&1; then \
		echo "Installing luarocks..."; \
		if command -v apt-get >/dev/null 2>&1; then \
			sudo apt-get update && sudo apt-get install -y luarocks; \
		elif command -v brew >/dev/null 2>&1; then \
			brew install luarocks; \
		else \
			echo "Please install luarocks manually"; \
		fi; \
	else \
		echo "luarocks is already installed"; \
	fi
	
	@if command -v luarocks >/dev/null 2>&1 && ! command -v luacheck >/dev/null 2>&1; then \
		echo "Installing luacheck..."; \
		sudo luarocks install luacheck; \
	elif command -v luacheck >/dev/null 2>&1; then \
		echo "luacheck is already installed"; \
	fi
	
	@# No need to set up specific targets for local builds
	
	@# Check system dependencies before installing
	@echo "Checking system dependencies..."
	@if command -v apt-get >/dev/null 2>&1; then \
		dpkg -l | grep -q "^ii.*build-essential" || { \
			echo "Installing build-essential..."; \
			sudo apt-get update && sudo apt-get install -y build-essential; \
		}; \
		dpkg -l | grep -q "^ii.*pkg-config" || { \
			echo "Installing pkg-config..."; \
			sudo apt-get install -y pkg-config; \
		}; \
		dpkg -l | grep -q "^ii.*libssl-dev" || { \
			echo "Installing libssl-dev..."; \
			sudo apt-get install -y libssl-dev; \
		}; \
	elif command -v yum >/dev/null 2>&1; then \
		rpm -q gcc >/dev/null 2>&1 || { \
			echo "Installing gcc..."; \
			sudo yum install -y gcc; \
		}; \
		rpm -q openssl-devel >/dev/null 2>&1 || { \
			echo "Installing openssl-devel..."; \
			sudo yum install -y openssl-devel; \
		}; \
		rpm -q pkg-config >/dev/null 2>&1 || { \
			echo "Installing pkg-config..."; \
			sudo yum install -y pkg-config; \
		}; \
	elif command -v brew >/dev/null 2>&1; then \
		brew list | grep -q openssl || { \
			echo "Installing openssl..."; \
			brew install openssl; \
		}; \
		brew list | grep -q pkg-config || { \
			echo "Installing pkg-config..."; \
			brew install pkg-config; \
		}; \
	fi
	
	@echo "Development environment setup complete"

# Clean all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -f $(OUTPUT_DIR)/$(BINARY_NAME)-*
	@cd $(RUST_DIR) && cargo clean
	@echo "Clean complete"

# Removed cross-compilation target

# Note: For cross-platform builds, set up GitHub Actions workflows

# Build development version for current platform
dev: dev.tools
	@echo "Building for development on $(PLATFORM_NAME) $(PLATFORM_ARCH)..."
	@cd $(RUST_DIR) && cargo build
	@chmod +x $(RUST_DIR)/target/debug/$(BINARY_NAME)
	@echo "Development build complete: $(RUST_DIR)/target/debug/$(BINARY_NAME)"

# Start the development server with the README
dev.start: dev
	@echo "Starting marv to preview the project README.md..."
	@cd $(RUST_DIR) && cargo run -- --start ../README.md

# Stop the development server
dev.stop:
	@echo "Stopping any running marv preview server for README.md..."
	@-$(RUST_DIR)/target/debug/$(BINARY_NAME) --stop README.md 2>/dev/null || true


# Lint Rust code
lint.rust:
	@echo "Linting Rust code..."
	@cd $(RUST_DIR) && cargo fmt -- --check
	@cd $(RUST_DIR) && cargo clippy -- -D warnings
	@echo "Rust linting complete"

# Lint Lua code
lint.lua:
	@echo "Linting Lua code..."
	@if command -v luacheck >/dev/null 2>&1; then \
		luacheck lua/; \
	else \
		echo "luacheck not found. Please run 'make install_dev_tools' first."; \
		exit 1; \
	fi
	@echo "Lua linting complete"

# Lint all code
lint: lint.rust lint.lua
	@echo "All linting complete"

# These targets are now replaced by dev.start and dev.stop

# Help target
help:
	@echo "Marv Makefile"
	@echo ""
	@echo "Currently supported platforms:"
	@echo "  - Native build for current platform: $(PLATFORM_NAME) $(PLATFORM_ARCH)"
	@echo ""
	@echo "Available commands:"
	@echo "  make build            Build for current platform ($(PLATFORM_NAME) $(PLATFORM_ARCH))"
	@echo "  make clean            Remove all built binaries"
	@echo "  make dev              Build development version for local platform"
	@echo "  make dev.start        Start preview server for README.md"
	@echo "  make dev.stop         Stop any running preview server for README.md"
	@echo "  make dev.tools        Install or check development and linting tools"
	@echo "  make lint             Run all linters (Rust and Lua)"
	@echo "  make lint.rust        Run Rust linters (rustfmt and clippy)"
	@echo "  make lint.lua         Run Lua linter (luacheck)"
	@echo "  make help             Show this help message"
	@echo ""
	@echo "Output binary will be: $(OUTPUT_BINARY)"
	@echo ""
	@echo "For cross-platform builds, set up GitHub Actions workflows."
