# ----- Tildr Makefile -----
#
#
# ----- Directories -----
DOCS_MD_DIR := docs/man
DOCS_MAN_DIR := docs/man/dist
DOCS_VERSION_SCRIPT := tools/scripts/sync-docs-version.sh
BRANCH := $(shell git branch --show-current)
REMOTES := $(shell git remote)
MD_FILES := $(wildcard $(DOCS_MD_DIR)/tildr*.md)

# Convert md → .1
MAN_FILES := $(patsubst $(DOCS_MD_DIR)/%.md,$(DOCS_MAN_DIR)/%.1,$(MD_FILES))

# ----- Targets -----

.PHONY: all build check release macos fmt fmt-check clippy test tests audit deny machete docs-version docs-version-check man man-gz clean push push-lease

# ----- Default -----
all: build

# ----- Builds (Linux) -----
build:
	cargo build --locked

check: fmt-check clippy test

release: docs-version man check
	cargo build --release --locked

# ----- Builds (macOS) -----

macos-x86_64: check
	cargo build -p tildr --target x86_64-apple-darwin --locked

macos-aarch64: check
	cargo build -p tildr --target aarch64-apple-darwin --locked

# ----- Rust -----
fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --locked

tests: test

audit:
	cargo audit

deny:
	cargo deny check

machete:
	cargo machete

# ----- Site docs -----
docs-version:
	$(DOCS_VERSION_SCRIPT) update

docs-version-check:
	$(DOCS_VERSION_SCRIPT) check

# ----- Man pages (Linux) -----
man: $(MAN_FILES)

$(DOCS_MAN_DIR)/%.1: $(DOCS_MD_DIR)/%.md
	@echo "Generating man page: $<"
	pandoc -s -t man $< -o $@

man-gz: man
	gzip -f $(DOCS_MAN_DIR)/tildr*.1

# ----- Clean -----
clean:
	cargo clean
	rm -f $(DOCS_MAN_DIR)/*.1

# ----- DEVELOPMENT (Push) -----
push:
	@echo "Push normal → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push $$remote $(BRANCH); \
	done

push-lease:
	@echo "Push --force-with-lease → branch: $(BRANCH)"
	@for remote in $(REMOTES); do \
		echo "  pushing to $$remote..."; \
		git push --force-with-lease $$remote $(BRANCH); \
	done
