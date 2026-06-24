# KeyStats.Linux — Build, Package & Install
# Usage:
#   make build    — build daemon + CLI + overlay (release) and compile locale
#   make install  — install daemon + overlay + systemd service + GNOME extension
#   make upgrade  — safe upgrade: daemon + overlay + systemd + extension (disable→install→enable)
#   make zip      — package extension as zip for distribution
#   make dist     — create binary tarball for GitHub releases
#   make test     — cargo test (all crates)
#   make check    — cargo check + fmt + clippy
#   make clean    — remove build artifacts and generated files

PREFIX       := $(HOME)/.local
SYSTEMD_USER := $(HOME)/.config/systemd/user
EXT_DIR      := $(HOME)/.local/share/gnome-shell/extensions/keystats@0x5c0f.github.io

VERSION      := $(shell cargo metadata --no-deps --format-version=1 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])" 2>/dev/null || echo "0.1.0")
DIST_NAME    := keystats-$(VERSION)-linux-x86_64
DIST_DIR     := target/dist/$(DIST_NAME)

.PHONY: build install upgrade zip dist test check clean

# ── Build ──────────────────────────────────────────────

build: build-rust build-locale

build-rust:
	cargo build --release

build-locale:
	$(MAKE) -C gnome-extension mo

# ── Install ────────────────────────────────────────────

install: install-daemon install-systemd install-extension
	@echo "Install complete. Reload GNOME Shell (Alt+F2 → r on X11, or log out/in on Wayland), then run:"
	@echo "  systemctl --user enable --now keystats"
	@echo "  gnome-extensions enable keystats@0x5c0f.github.io"
	@echo ""
	@echo "Optional: install the keystroke overlay for screencasting:"
	@echo "  make install-overlay"

install-daemon: build-rust
	mkdir -p $(PREFIX)/bin
	cp target/release/keystats-daemon $(PREFIX)/bin/
	cp target/release/keystatsctl $(PREFIX)/bin/

install-overlay: build-rust
	mkdir -p $(PREFIX)/bin
	cp target/release/keystats-overlay $(PREFIX)/bin/
	mkdir -p $(SYSTEMD_USER)
	cp packaging/systemd/keystats-overlay.service $(SYSTEMD_USER)/
	systemctl --user daemon-reload
	@echo "Overlay installed. Run 'keystats-overlay --help' for options."
	@echo "To auto-start with session: systemctl --user enable keystats-overlay"

install-systemd:
	mkdir -p $(SYSTEMD_USER)
	cp packaging/systemd/keystats.service $(SYSTEMD_USER)/
	cp packaging/systemd/keystats-overlay.service $(SYSTEMD_USER)/
	systemctl --user daemon-reload

install-extension: build-locale
	mkdir -p $(EXT_DIR)
	cp gnome-extension/metadata.json $(EXT_DIR)/
	cp gnome-extension/extension.js $(EXT_DIR)/
	cp gnome-extension/prefs.js $(EXT_DIR)/
	cp gnome-extension/stylesheet.css $(EXT_DIR)/
	cp -r gnome-extension/schemas $(EXT_DIR)/
	cp -r gnome-extension/locale $(EXT_DIR)/
	glib-compile-schemas $(EXT_DIR)/schemas/

upgrade:
	@echo "Stopping keystats service..."
	@systemctl --user stop keystats.service || true
	@systemctl --user stop keystats-overlay.service || true
	@echo "Disabling extension..."
	@gnome-extensions disable keystats@0x5c0f.github.io || true
	@sleep 1
	$(MAKE) install-daemon install-systemd
	@# Also install overlay if it was previously installed
	@if [ -f $(PREFIX)/bin/keystats-overlay ]; then \
		cp target/release/keystats-overlay $(PREFIX)/bin/; \
		echo "Overlay binary updated."; \
	fi
	$(MAKE) build-locale
	@mkdir -p $(EXT_DIR)
	@cp gnome-extension/metadata.json $(EXT_DIR)/
	@cp gnome-extension/extension.js $(EXT_DIR)/
	@cp gnome-extension/prefs.js $(EXT_DIR)/
	@cp gnome-extension/stylesheet.css $(EXT_DIR)/
	@cp -r gnome-extension/schemas $(EXT_DIR)/
	@cp -r gnome-extension/locale $(EXT_DIR)/
	@glib-compile-schemas $(EXT_DIR)/schemas/
	@echo "Restarting keystats service..."
	@systemctl --user restart keystats.service || true
	@systemctl --user restart keystats-overlay.service || true
	@sleep 1
	@echo "Re-enabling extension..."
	@gnome-extensions enable keystats@0x5c0f.github.io || true
	@echo "Upgrade complete."

upgrade-extension: build-locale
	@echo "Disabling extension..."
	@gnome-extensions disable keystats@0x5c0f.github.io || true
	@sleep 1
	mkdir -p $(EXT_DIR)
	cp gnome-extension/metadata.json $(EXT_DIR)/
	cp gnome-extension/extension.js $(EXT_DIR)/
	cp gnome-extension/prefs.js $(EXT_DIR)/
	cp gnome-extension/stylesheet.css $(EXT_DIR)/
	cp -r gnome-extension/schemas $(EXT_DIR)/
	cp -r gnome-extension/locale $(EXT_DIR)/
	glib-compile-schemas $(EXT_DIR)/schemas/
	@sleep 1
	@echo "Re-enabling extension..."
	@gnome-extensions enable keystats@0x5c0f.github.io || true

# ── Package ────────────────────────────────────────────

zip: build-locale
	cd gnome-extension && $(MAKE) zip

dist: build
	@rm -rf target/dist
	@mkdir -p $(DIST_DIR)/bin $(DIST_DIR)/systemd $(DIST_DIR)/udev
	cp target/release/keystats-daemon $(DIST_DIR)/bin/
	cp target/release/keystatsctl $(DIST_DIR)/bin/
	cp target/release/keystats-overlay $(DIST_DIR)/bin/
	cp packaging/systemd/keystats.service $(DIST_DIR)/systemd/
	cp packaging/systemd/keystats-overlay.service $(DIST_DIR)/systemd/
	cp packaging/udev/60-keystats-input.rules $(DIST_DIR)/udev/
	cp packaging/dist/Makefile $(DIST_DIR)/
	cp packaging/dist/README.md $(DIST_DIR)/
	cp packaging/dist/README.zh-CN.md $(DIST_DIR)/
	@tar -czf target/dist/$(DIST_NAME).tar.gz -C target/dist $(DIST_NAME)
	@rm -rf $(DIST_DIR)
	@echo "Created target/dist/$(DIST_NAME).tar.gz"

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
