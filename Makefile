.PHONY: help proto-gen clean-proto install-buf check-buf

# Default target
help:
	@echo "Available targets:"
	@echo "  proto-gen    - Generate protobuf code from CometBFT protos"
	@echo "  clean-proto  - Clean generated protobuf code"
	@echo "  install-buf  - Install buf CLI tool"
	@echo "  check-buf    - Check if buf is installed"
	@echo "  help         - Show this help message"

# Check if buf is installed
check-buf:
	@which buf > /dev/null || (echo "buf is not installed. Run 'make install-buf' to install it." && exit 1)

# Install buf CLI tool
install-buf:
	@echo "Installing buf CLI..."
	@if command -v brew >/dev/null 2>&1; then \
		brew install bufbuild/buf/buf; \
	elif command -v curl >/dev/null 2>&1; then \
		curl -sSL https://github.com/bufbuild/buf/releases/latest/download/buf-$$(uname -s)-$$(uname -m) -o /tmp/buf && \
		sudo mv /tmp/buf /usr/local/bin/buf && \
		sudo chmod +x /usr/local/bin/buf; \
	else \
		echo "Please install buf manually from https://github.com/bufbuild/buf/releases"; \
		exit 1; \
	fi
	@echo "buf installed successfully!"

# Generate protobuf code
proto-gen: check-buf
	@echo "Generating protobuf code from CometBFT protos..."
	@mkdir -p crates/astro-proto-types/src/codegen
	buf generate
	@echo "Protobuf code generation completed!"
	@echo "Generated files are located in: crates/astro-proto-types/src/codegen/"

# Clean generated protobuf code
clean-proto:
	@echo "Cleaning generated protobuf code..."
	rm -rf crates/astro-proto-types/src/codegen
	@echo "Cleaned generated protobuf code!"

# Full rebuild: clean and generate
rebuild-proto: clean-proto proto-gen

# Build the proto crate after generation
build-proto: proto-gen
	@echo "Building astro-proto-types crate..."
	cargo build -p astro-proto-types

# Run tests for the proto crate
test-proto: proto-gen
	@echo "Testing astro-proto-types crate..."
	cargo test -p astro-proto-types
