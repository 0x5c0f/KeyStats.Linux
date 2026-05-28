# KeyStats.Linux — Build, Package & Install
# Usage:
#   make build    — build daemon + CLI (release) and compile locale
#   make install  — install daemon + systemd service + GNOME extension
#   make zip      — package extension as zip for distribution
#   make test     — cargo test (all crates)
#   make check    — cargo check + fmt + clippy
#   make clean    — remove build artifacts and generated files

PREFIX       := $(HOME)/.local
SYSTEMD_USER := $(HOME)/.config/systemd/user
EXT_DIR      := $(HOME)/.local/share/gnome-shell/extensions/keystats@0x5c0f.github.io

.PHONY: build install zip test check clean

# ── Build ──────────────────────────────────────────────

build: build-rust build-locale

build-rust:
	cargo build --release

build-locale:
	$(MAKE) -C gnome-extension mo

# ── Install ────────────────────────────────────────────

install: install-daemon install-systemd install-extension
	@echo "Install complete. Restart GNOME Shell and run:"
	@echo "  systemctl --user enable --now keystats"
	@echo "  gnome-extensions enable keystats@0x5c0f.github.io"

install-daemon: build-rust
	mkdir -p $(PREFIX)/bin
	cp target/release/keystats-daemon $(PREFIX)/bin/
	cp target/release/keystatsctl $(PREFIX)/bin/

install-systemd:
	mkdir -p $(SYSTEMD_USER)
	cp packaging/systemd/keystats.service $(SYSTEMD_USER)/
	systemctl --user daemon-reload

install-extension: build-locale
	mkdir -p $(EXT_DIR)
	cp gnome-extension/metadata.json $(EXT_DIR)/
	cp gnome-extension/extension.js $(EXT_DIR)/
	cp gnome-extension/prefs.js $(EXT_DIR)/
	cp gnome-extension/stylesheet.css $(EXT_DIR)/
	cp -r gnome-extension/schemas $(EXT_DIR)/
	cp -r gnome-extension/locale $(EXT_DIR)/

# ── Package ────────────────────────────────────────────

zip: build-locale
	cd gnome-extension && $(MAKE) zip

# ── Test & Check ───────────────────────────────────────

test:
	cargo test

check:
	cargo check
	cargo fmt -- --check
	cargo clippy -- -D warnings

# ── Clean ──────────────────────────────────────────────

clean:
	cargo clean
	$(MAKE) -C gnome-extension clean
