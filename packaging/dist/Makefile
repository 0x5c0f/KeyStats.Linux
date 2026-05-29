# KeyStats.Linux — Binary Package Installer
# Usage:
#   make install    — install binaries and systemd service
#   make upgrade    — safe upgrade: stop service → install → restart
#   make uninstall  — remove installed files
#   make enable     — enable and start the daemon service

PREFIX       := $(HOME)/.local
SYSTEMD_USER := $(HOME)/.config/systemd/user

.PHONY: install upgrade uninstall enable

install:
	mkdir -p $(PREFIX)/bin
	install -m 755 bin/keystats-daemon $(PREFIX)/bin/
	install -m 755 bin/keystatsctl $(PREFIX)/bin/
	mkdir -p $(SYSTEMD_USER)
	install -m 644 systemd/keystats.service $(SYSTEMD_USER)/
	systemctl --user daemon-reload
	@echo ""
	@echo "Installed to $(PREFIX)/bin/"
	@echo ""
	@case ":$(PATH):" in *":$(PREFIX)/bin":*) ;; *) \
		echo "WARNING: $(PREFIX)/bin is not in your PATH."; \
		echo "  Add it by running:"; \
		echo "    echo 'export PATH=\"\$$HOME/.local/bin:\$$PATH\"' >> ~/.bashrc"; \
		echo "    source ~/.bashrc"; \
		echo "";; \
	esac
	@echo "Next steps:"
	@echo "  1. Install the GNOME extension from the .zip file:"
	@echo "     gnome-extensions install keystats@0x5c0f.github.io.zip"
	@echo "  2. Enable and start the daemon:"
	@echo "     make enable"
	@echo "  3. Reload GNOME Shell, then enable the extension:"
	@echo "     gnome-extensions enable keystats@0x5c0f.github.io"

upgrade:
	@echo "Stopping keystats service..."
	@systemctl --user stop keystats.service || true
	@sleep 1
	mkdir -p $(PREFIX)/bin
	install -m 755 bin/keystats-daemon $(PREFIX)/bin/
	install -m 755 bin/keystatsctl $(PREFIX)/bin/
	mkdir -p $(SYSTEMD_USER)
	install -m 644 systemd/keystats.service $(SYSTEMD_USER)/
	systemctl --user daemon-reload
	@echo "Restarting keystats service..."
	@systemctl --user restart keystats.service || true
	@echo "Upgrade complete."

enable:
	systemctl --user enable --now keystats
	@echo "keystats daemon started."

uninstall:
	rm -f $(PREFIX)/bin/keystats-daemon
	rm -f $(PREFIX)/bin/keystatsctl
	rm -f $(SYSTEMD_USER)/keystats.service
	systemctl --user daemon-reload
	@echo "Uninstalled. GNOME extension must be removed separately:"
	@echo "  gnome-extensions uninstall keystats@0x5c0f.github.io"
