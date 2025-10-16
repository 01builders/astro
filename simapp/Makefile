# Proto build helpers for the astro module

SHELL := /bin/bash

PWD := $(shell pwd)
DOCKER ?= docker
WORKDIR ?= /workspace

# Use the published Cosmos SDK proto-builder image (has protoc, buf, plugins)
PROTO_IMAGE ?= ghcr.io/cosmos/proto-builder:0.14.0

# Local image tag if you want to build the wrapper Dockerfile in this repo
PROTO_IMAGE_LOCAL ?= astro-proto-builder:local

# Convenience for running inside the container with current UID/GID so files are writable
UID_GID := $(shell id -u):$(shell id -g)

.PHONY: proto-image proto-image-local proto-lint proto-format proto-gen proto-clean

proto-image:
	$(DOCKER) pull $(PROTO_IMAGE)

# Build a local image from Dockerfile.proto (optional)
proto-image-local:
	$(DOCKER) build -f Dockerfile.proto -t $(PROTO_IMAGE_LOCAL) .

proto-format:
	mkdir -p .cache
	$(DOCKER) run --rm \
	  -v $(PWD):$(WORKDIR) -w $(WORKDIR) \
	  -e HOME=$(WORKDIR) \
	  -e XDG_CACHE_HOME=$(WORKDIR)/.cache \
	  -e BUF_CACHE_DIR=$(WORKDIR)/.cache \
	  $(PROTO_IMAGE) buf format -w

proto-lint:
	mkdir -p .cache
	$(DOCKER) run --rm \
	  -v $(PWD):$(WORKDIR) -w $(WORKDIR) \
	  -e HOME=$(WORKDIR) \
	  -e XDG_CACHE_HOME=$(WORKDIR)/.cache \
	  -e BUF_CACHE_DIR=$(WORKDIR)/.cache \
	  $(PROTO_IMAGE) buf lint

proto-gen:
	mkdir -p .cache
	$(DOCKER) run --rm -u $(UID_GID) \
	  -v $(PWD):$(WORKDIR) -w $(WORKDIR) \
	  -e HOME=$(WORKDIR) \
	  -e XDG_CACHE_HOME=$(WORKDIR)/.cache \
	  -e BUF_CACHE_DIR=$(WORKDIR)/.cache \
	  $(PROTO_IMAGE) buf generate --template buf.gen.yaml

# Remove any previously generated code under gen/ or other temp dirs if you use them
proto-clean:
	rm -rf gen 2>/dev/null || true
