# KeyStats.Linux

Privacy-first keyboard and mouse statistics for Linux with GNOME Shell integration.

## Prerequisites

- GNOME Shell 45+
- User must be in the `input` group to read `/dev/input/event*`:

```bash
sudo usermod -aG input $USER
# Log out and back in for the change to take effect
```

- `libgtk-4` (required by keystats-overlay, pre-installed on most GNOME desktops)

## Install

```bash
make install
```

This installs:
- `keystats-daemon`, `keystatsctl`, and `keystats-overlay` to `~/.local/bin/`
- systemd user services to `~/.config/systemd/user/`

## Enable the Daemon

```bash
make enable
```

## Install the GNOME Extension

```bash
gnome-extensions install keystats@0x5c0f.github.io.zip
```

Reload GNOME Shell (`Alt+F2` → `r` on X11, or log out/in on Wayland), then:

```bash
gnome-extensions enable keystats@0x5c0f.github.io
```

## Verify

```bash
keystatsctl status
```

## Uninstall

```bash
make uninstall
gnome-extensions uninstall keystats@0x5c0f.github.io
```
