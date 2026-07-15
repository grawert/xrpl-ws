.PHONY: help build clean lint fmt-check clippy \
        test test-unit test-doc test-integration \
        docs-lint docs-html docs-api docs-clean version

.DEFAULT_GOAL := help

export CARGO_TARGET_DIR ?= target

CRATE_NAME := xrpl
DOC_JSON   := $(CARGO_TARGET_DIR)/doc/$(CRATE_NAME).json
API_MD     := docs/API.md

help: ## This help
	@awk 'BEGIN {FS = ":.*?## "} /^[0-9a-zA-Z_-]+:.*?## / {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the library
	cargo build

clean: ## Remove all cargo build artifacts
	cargo clean

lint: fmt-check clippy docs-lint ## Run all lint checks (fmt, clippy, doc build with warnings as errors)

fmt-check: ## Check code formatting
	cargo fmt --check

clippy: ## Run clippy with warnings as errors
	cargo clippy -- -D warnings

docs-lint: ## Build docs (incl. private items), treating warnings as errors
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

docs-html: ## Build HTML documentation for the library
	cargo doc --no-deps --lib

docs-api: ## Regenerate docs/API.md from rustdoc JSON output
	mkdir -p $(dir $(API_MD))
	cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json
	rustdoc-md --path $(DOC_JSON) --output $(API_MD)
	awk -f scripts/strip-std-traits.awk $(API_MD) > $(API_MD).tmp && mv $(API_MD).tmp $(API_MD)

docs-clean: ## Remove generated documentation artifacts
	rm -rf $(CARGO_TARGET_DIR)/doc

test: test-unit test-doc test-integration ## Run unit, doc, and integration tests

test-unit: ## Run unit tests (src/ #[cfg(test)] modules)
	cargo test --lib -- --nocapture

test-doc: ## Run documentation tests
	cargo test --doc

test-integration: ## Run integration tests (tests/ directory)
	cargo test --test '*' -- --nocapture

version: ## Bump crate version (usage: make version VERSION=x.y.z)
	@if [ -z "$(VERSION)" ]; then echo "Usage: make version VERSION=x.y.z"; exit 1; fi
	@OLD_VERSION=$$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/'); \
	if [ "$$OLD_VERSION" = "$(VERSION)" ]; then echo "Already at version $(VERSION)"; exit 1; fi; \
	sed -i "0,/^version = \".*\"/s//version = \"$(VERSION)\"/" Cargo.toml; \
	cargo check --quiet; \
	jq --arg tag "v$$OLD_VERSION" \
	   '.previousVersions = ((.previousVersions // []) + (if any(.previousVersions[]?; .tag == $$tag) then [] else [{"tag": $$tag}] end))' \
	   context7.json > context7.json.tmp && mv context7.json.tmp context7.json; \
	echo "Bumped $$OLD_VERSION -> $(VERSION). Review the diff, commit on a feature branch, and open a PR; the release workflow tags it once merged to main."
