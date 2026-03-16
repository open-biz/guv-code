BINARY_NAME := guv
VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d '"' -f 2)
COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

# Release architecture mappings
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH := $(shell uname -m)
ifeq ($(ARCH),x86_64)
	ARCH := amd64
endif
ifeq ($(ARCH),aarch64)
	ARCH := arm64
endif

.PHONY: build install test lint clean release release-tag release-push help

build:
	cargo build --release
	cp target/release/guv-code ./$(BINARY_NAME)

install: build
	cp $(BINARY_NAME) /usr/local/bin/$(BINARY_NAME)

test:
	cargo test

lint:
	cargo clippy -- -D warnings
	cargo fmt --all -- --check

clean:
	cargo clean
	rm -f $(BINARY_NAME)

# --- Release Helpers ---

release-tag:
	@echo "Creating tag v$(VERSION)..."
	@git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo "Tag created. Push with: git push origin v$(VERSION)"

release-push:
	git push origin "v$(VERSION)"

# This is a local helper to package a release for the current platform
release-local: build
	tar -czf $(BINARY_NAME)_$(VERSION)_$(OS)_$(ARCH).tar.gz $(BINARY_NAME) README.md LICENSE
	@echo "Packaged: $(BINARY_NAME)_$(VERSION)_$(OS)_$(ARCH).tar.gz"

help:
	@echo "Available targets:"
	@echo "  build         - Build the release binary locally"
	@echo "  install       - Build and install to /usr/local/bin"
	@echo "  test          - Run Rust tests"
	@echo "  lint          - Run clippy and fmt checks"
	@echo "  clean         - Remove build artifacts"
	@echo "  release-tag   - Create a git tag for the current version in Cargo.toml"
	@echo "  release-push  - Push the current version tag to origin"
	@echo "  release-local - Build and package the binary for the current system"
