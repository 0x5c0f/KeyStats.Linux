use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use evdev::EventSummary;
use keystats_core::InputEvent;

use super::device::{DeviceKind, InputDevice};
use super::keymap;
use crate::stats::manager::{StatsManager, lock_stats};

/// Polling interval for non-blocking device reads (~125 Hz).
const POLL_INTERVAL: Duration = Duration::from_millis(8);

/// Tracks held modifier keys for combo display (e.g. Ctrl+C).
#[derive(Debug, Default)]
struct ModifierState {
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

impl ModifierState {
    /// Update state from a key code. Returns `true` if the code is a modifier.
    fn update(&mut self, code: u16, pressed: bool) -> bool {
        if keymap::is_shift(code) {
            self.shift = pressed;
            true
        } else if keymap::is_ctrl(code) {
            self.ctrl = pressed;
            true
        } else if keymap::is_alt(code) {
            self.alt = pressed;
            true
        } else if keymap::is_meta(code) {
            self.meta = pressed;
            true
        } else {
            false
        }
    }

    /// Build a combo name: prefix non-Shift modifiers to the base key name.
    /// Shift is excluded because it's already handled by `shifted_key_name()`.
    /// E.g. if Ctrl is held, returns "Ctrl+A". If Ctrl+Shift, returns "Ctrl+A" (Shift baked in).
    fn combo_name(&self, base: &str) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.meta {
            parts.push("Super");
        }
        if parts.is_empty() {
            base.to_string()
        } else {
            parts.push(base);
            parts.join("+")
        }
    }

    /// Build a modifier-only combo name from all currently held modifiers.
    /// Used when a modifier is pressed while other modifiers are held.
    /// E.g. Ctrl+Shift pressed together → "Ctrl+Shift".
    fn modifier_combo_name(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.meta {
            parts.push("Super");
        }
        parts.join("+")
    }

    /// Returns `true` if two or more modifiers are currently held.
    fn multiple_modifiers_held(&self) -> bool {
        let count = [self.ctrl, self.shift, self.alt, self.meta]
            .iter()
            .filter(|&&v| v)
            .count();
        count >= 2
    }

    /// Returns `true` if any non-Shift modifier is currently held.
    fn combo_modifiers_held(&self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}

/// Process a batch of events from one device, calling the appropriate
/// StatsManager recording methods. Returns the number of events processed.
///
/// `event_tx` is an optional channel for real-time input events (overlay).
/// `modifiers` tracks held modifier keys for combo display.
/// `held_keys` tracks which names were sent on press, so release uses the same name.
fn process_device(
    device: &mut InputDevice,
    stats: &mut StatsManager,
    event_tx: &Option<mpsc::Sender<InputEvent>>,
    modifiers: &mut ModifierState,
    held_keys: &mut HashMap<u16, String>,
) -> usize {
    let mut count = 0;
    let mut pending_dx: f64 = 0.0;
    let mut pending_dy: f64 = 0.0;

    let events = match device.device.fetch_events() {
        Ok(iter) => iter,
        Err(_) => return 0,
    };

    for ev in events {
        match ev.destructure() {
            EventSummary::Key(_, code, value) => {
                count += 1;
                let code_u16 = code.0;

                if value == 2 {
                    // Auto-repeat — skip entirely
                    continue;
                }

                let is_mod = keymap::is_shift(code_u16)
                    || keymap::is_ctrl(code_u16)
                    || keymap::is_alt(code_u16)
                    || keymap::is_meta(code_u16);

                match value {
                    1 => {
                        // Press: update modifier state first, then send event
                        if is_mod {
                            modifiers.update(code_u16, true);
                        }

                        if let Some(role) = keymap::button_role(code_u16) {
                            stats.record_click(role);
                        } else {
                            match device.kind {
                                DeviceKind::Keyboard | DeviceKind::KeyboardPointer => {
                                    let base_name = if modifiers.shift && !is_mod {
                                        keymap::shifted_key_name(code_u16)
                                            .map(String::from)
                                            .unwrap_or_else(|| keymap::key_name(code_u16))
                                    } else {
                                        keymap::key_name(code_u16)
                                    };
                                    let name = if is_mod && modifiers.multiple_modifiers_held() {
                                        // Multiple modifiers held: show combo (e.g. "Ctrl+Shift")
                                        modifiers.modifier_combo_name()
                                    } else if !is_mod && modifiers.combo_modifiers_held() {
                                        // Regular key with modifiers: show combo (e.g. "Ctrl+A")
                                        modifiers.combo_name(&base_name)
                                    } else {
                                        base_name
                                    };
                                    stats.record_key_press(&name, true);
                                    if let Some(tx) = event_tx {
                                        let _ = tx.send(InputEvent::KeyPress { name: name.clone() });
                                    }
                                    // Track sent name so release uses the same name
                                    held_keys.insert(code_u16, name);
                                }
                                DeviceKind::Pointer | DeviceKind::Other => {}
                            }
                        }
                    }
                    0 => {
                        // Release: send event first, then update modifier state
                        if keymap::button_role(code_u16).is_none()
                            && let DeviceKind::Keyboard | DeviceKind::KeyboardPointer = device.kind
                            && let Some(tx) = event_tx
                        {
                            // Use stored name from press to ensure match
                            let name = held_keys.remove(&code_u16)
                                .unwrap_or_else(|| keymap::key_name(code_u16));
                            let _ = tx.send(InputEvent::KeyRelease { name });
                        }
                        // Update modifier state AFTER sending release event
                        if is_mod {
                            modifiers.update(code_u16, false);
                        }
                    }
                    _ => {}
                }
            }

            EventSummary::RelativeAxis(_, code, value) => {
                count += 1;
                let val = value as f64;
                match code.0 {
                    0 => pending_dx += val, // REL_X
                    1 => pending_dy += val, // REL_Y
                    8 => {
                        // REL_WHEEL
                        stats.add_scroll_distance(val);
                    }
                    6 => {
                        // REL_HWHEEL
                        stats.add_scroll_distance(val);
                    }
                    _ => {}
                }
            }

            EventSummary::Synchronization(_, code, _)
                if code.0 == 0 && (pending_dx != 0.0 || pending_dy != 0.0) =>
            {
                stats.add_mouse_distance(pending_dx, pending_dy);
                pending_dx = 0.0;
                pending_dy = 0.0;
            }

            _ => {}
        }

        // Also accumulate non-SYN relative values immediately
        // (many devices don't emit SYN_REPORT between every relative event)
        if pending_dx.abs() > 100.0 || pending_dy.abs() > 100.0 {
            stats.add_mouse_distance(pending_dx, pending_dy);
            pending_dx = 0.0;
            pending_dy = 0.0;
        }
    }

    // Flush any remaining pending movement
    if pending_dx != 0.0 || pending_dy != 0.0 {
        stats.add_mouse_distance(pending_dx, pending_dy);
    }

    count
}

/// Re-scan input devices every 30 seconds to handle hotplug (USB/BT reconnect,
/// suspend/resume, etc.) so that stale file descriptors are replaced.
const RESCAN_INTERVAL: Duration = Duration::from_secs(30);

/// Run the main input event loop. Blocks the current thread.
/// Reads from all discovered devices in non-blocking mode,
/// calling StatsManager for each processed event.
///
/// `event_tx` is an optional channel sender for real-time input events.
/// When `Some`, key press/release events are forwarded to the receiver
/// (used by the overlay process via D-Bus signals). When `None`, no events are sent.
pub fn run(stats: Arc<Mutex<StatsManager>>, event_tx: Option<mpsc::Sender<InputEvent>>) -> ! {
    let (mut devices, _blocked) = InputDevice::discover();

    tracing::info!(
        "Event loop started with {} devices ({} keyboard, {} pointer)",
        devices.len(),
        devices.iter().filter(|d| matches!(d.kind, DeviceKind::Keyboard)).count(),
        devices
            .iter()
            .filter(|d| matches!(d.kind, DeviceKind::Pointer | DeviceKind::KeyboardPointer))
            .count()
    );

    let poll_interval = POLL_INTERVAL;
    let mut last_rescan = std::time::Instant::now();
    let mut modifiers = ModifierState::default();
    let mut held_keys = HashMap::new();

    loop {
        let mut total = 0usize;
        {
            let Some(mut mgr) = lock_stats(&stats) else {
                continue;
            };
            for device in &mut devices {
                total += process_device(device, &mut mgr, &event_tx, &mut modifiers, &mut held_keys);
            }
        }

        // Periodic device re-scan for hotplug
        if last_rescan.elapsed() >= RESCAN_INTERVAL {
            let (new_devices, _) = InputDevice::discover();
            if new_devices.len() != devices.len() {
                tracing::info!(
                    "Device count changed: {} -> {} (reload)",
                    devices.len(),
                    new_devices.len()
                );
            }
            devices = new_devices;
            last_rescan = std::time::Instant::now();
        }

        if total > 0 {
            continue;
        }

        std::thread::sleep(poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_devices() {
        let (devices, _) = InputDevice::discover();
        assert!(!devices.is_empty(), "Need at least one input device");
    }

    #[test]
    fn keymap_covers_common_keys() {
        // Sanity check: common keys are mapped
        assert_eq!(keymap::key_name(30), "A");
        assert_eq!(keymap::key_name(16), "Q");
        assert_eq!(keymap::key_name(57), "Space");
        assert_eq!(keymap::button_role(0x110), Some("left"));
    }
}
