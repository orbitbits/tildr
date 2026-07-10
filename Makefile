# ----- Tildr Makefile -----
#
#
# ----- Directories -----
DOCS_MD_DIR := docs/man
DOCS_MAN_DIR := docs/man/dist
BRANCH := $(shell git branch --show-current)
REMOTES := $(shell git remote)
MD_FILES := $(wildcard $(DOCS_MD_DIR)/tildr*.md)

# Convert md → .1
MAN_FILES := $(patsubst $(DOCS_MD_DIR)/%.md,$(DOCS_MAN_DIR)/%.1,$(MD_FILES))

# ----- Targets -----

.PHONY: all build release macos fmt tests man man-gz clean push push-lease

# ----- Default -----
all: build

# ----- Builds (Linux) -----
build: fmt
	cargo build

release: man fmt tests
	cargo build --release

# ----- Builds (macOS) -----

macos-x86_64: fmt tests
	cargo build -p tildr --target x86_64-apple-darwin

macos-aarch64: fmt tests
	cargo build -p tildr --target aarch64-apple-darwin

# ----- Rust -----
fmt:
	cargo fmt --all

tests:
	cargo test

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
