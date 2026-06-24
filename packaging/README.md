# KeyStats Linux Packaging

English | [简体中文](README.zh-CN.md)

All commands assume you are in the **repository root**. Paths are relative to the repo root unless stated otherwise.

```
KeyStats.Linux/                    ← you are here (repository root)
├── Cargo.toml                     ← Rust workspace
├── crates/ (keystats-core, keystats-daemon, keystatsctl, keystats-overlay)
├── gnome-extension/               ← GNOME extension source
├── packaging/ (systemd, udev)
└── ...
```

---

## Prerequisites

- **Rust** 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **GNOME Shell 45+**
- **input group** membership

```bash
sudo usermod -aG input $USER
# Log out and back in, or: newgrp input
```

---

## Development Build & Run

All cargo commands run from `KeyStats.Linux/`:

```bash
cd KeyStats.Linux

# Build
cargo build -p keystats-daemon -p keystatsctl

# Run daemon (foreground, Ctrl+C to stop)
cargo run -p keystats-daemon

# Check permissions
cargo run -p keystatsctl -- doctor

# Check daemon stats
cargo run -p keystatsctl -- status

# Build and run overlay (requires libgtk-4-dev)
cargo run -p keystats-overlay -- --help
```

---

## Installation

### 1. Build and install daemon + CLI + overlay

```bash
cd KeyStats.Linux

# Release build (requires libgtk-4-dev for overlay)
cargo build --release

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/keystats-daemon ~/.local/bin/
cp target/release/keystatsctl ~/.local/bin/
cp target/release/keystats-overlay ~/.local/bin/

# Ensure ~/.local/bin is on PATH
export PATH="$HOME/.local/bin:$PATH"
```

### 2. Install systemd user service

```bash
mkdir -p ~/.config/systemd/user
cp KeyStats.Linux/packaging/systemd/keystats.service ~/.config/systemd/user/
cp KeyStats.Linux/packaging/systemd/keystats-overlay.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now keystats.service

# Verify
systemctl --user status keystats.service

# Optional: auto-start overlay with session
systemctl --user enable --now keystats-overlay.service
```

### 3. Install GNOME Shell extension

The extension source is at `gnome-extension/`.

**Option A: pack as zip and install** (clean, recommended for distribution)

```bash
cd gnome-extension

# Build locale, create zip package
make zip

# Install
gnome-extensions install keystats@0x5c0f.github.io.zip
```

**Option B: install directly** (quick dev iteration)

```bash
cd gnome-extension

# Build locale, copy extension to ~/.local/share/gnome-shell/extensions/
make install
```

After either option:

```bash
# Restart GNOME Shell: press Alt+F2, type r, press Enter

# Enable the extension
gnome-extensions enable keystats@0x5c0f.github.io
```

### 4. Verify

```bash
keystatsctl doctor       # check devices
keystatsctl status       # check daemon stats
# Click the "K... C..." indicator in the top bar → popup should open
```

---

## Packaging for Distribution

All packaging commands run from the **repo root**.

### tarball

```bash
cd KeyStats.Linux

# Build release binaries (requires libgtk-4-dev for overlay)
cargo build --release

# Assemble tarball
mkdir -p dist
cp target/release/keystats-daemon dist/
cp target/release/keystatsctl dist/
cp target/release/keystats-overlay dist/
cp packaging/systemd/keystats.service dist/
cp packaging/systemd/keystats-overlay.service dist/
cp packaging/udev/60-keystats-input.rules dist/
cd dist && tar -czf ../keystats-linux-x86_64.tar.gz *
# Tarball at: KeyStats.Linux/keystats-linux-x86_64.tar.gz
```

### GNOME extension zip

```bash
cd gnome-extension
zip -r keystats@0x5c0f.github.io.zip \
    metadata.json extension.js prefs.js stylesheet.css schemas/
# Zip at: gnome-extension/keystats@0x5c0f.github.io.zip
```

Move the zip to the repo root for release upload:

```bash
cp gnome-extension/keystats@0x5c0f.github.io.zip .
```

### .deb

```bash
cargo install cargo-deb
cd KeyStats.Linux && cargo deb -p keystats-daemon
```

### .rpm

```bash
cargo install cargo-rpm
cd KeyStats.Linux && cargo rpm build -p keystats-daemon
```

---

## Permissions

Run `keystatsctl doctor` to check. If devices are blocked:

```bash
# Option A: input group (recommended)
sudo usermod -aG input $USER
# Re-login

# Option B: dedicated keystats group + udev rule
sudo cp KeyStats.Linux/packaging/udev/60-keystats-input.rules /etc/udev/rules.d/
sudo groupadd --system keystats
sudo usermod -aG keystats $USER
sudo udevadm control --reload-rules
sudo udevadm trigger
```

---

## Troubleshooting

| Symptom | Check |
|---------|-------|
| Panel shows "K--" "C--" | Is daemon running? `systemctl --user status keystats.service` |
| `keystatsctl doctor` shows blocked | Not in `input` group. See Permissions |
| Extension not in `gnome-extensions list` | Restart GNOME Shell: **Alt+F2 → r → Enter** |
| Extension loads but method missing | Daemon needs rebuild. Run `cargo build -p keystats-daemon` and restart |
| Preferences Reset/Clear fails | Daemon not running or not rebuilt after schema changes |

---

## Uninstall

```bash
# Daemon + overlay
systemctl --user disable --now keystats.service
systemctl --user disable --now keystats-overlay.service 2>/dev/null
rm ~/.config/systemd/user/keystats.service
rm ~/.config/systemd/user/keystats-overlay.service
rm ~/.local/bin/keystats-daemon ~/.local/bin/keystatsctl ~/.local/bin/keystats-overlay

# GNOME extension
gnome-extensions uninstall keystats@0x5c0f.github.io

# Data
rm -rf ~/.local/state/keystats/

# Optional: remove udev rule
sudo rm /etc/udev/rules.d/60-keystats-input.rules
```

---

## File Locations

| What | Where |
|------|-------|
| Daemon binary | `~/.local/bin/keystats-daemon` |
| CLI binary | `~/.local/bin/keystatsctl` |
| Overlay binary | `~/.local/bin/keystats-overlay` |
| systemd service | `~/.config/systemd/user/keystats.service` |
| Overlay systemd service | `~/.config/systemd/user/keystats-overlay.service` |
| GNOME extension | `~/.local/share/gnome-shell/extensions/keystats@0x5c0f.github.io/` |
| Stats database | `~/.local/state/keystats/stats.sqlite3` |
| Tarball (after build) | `KeyStats.Linux/keystats-linux-x86_64.tar.gz` |
| Extension zip (after build) | `gnome-extension/keystats@0x5c0f.github.io.zip` |
| Journal logs | `journalctl --user -u keystats.service` |
