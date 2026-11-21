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
        let count = [self.ctrl, self.shift, self.alt, self.meta].iter().filter(|&&v| v).count();
        count >= 2
    }

    /// Returns `true` if any non-Shift modifier is currently held.
    fn combo_modifiers_held(&self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}

/// Tracks per-touchpad state for absolute coordinate deltas, scrolling, and tap-to-click.
#[derive(Debug, Default)]
struct TouchpadState {
    touching: bool,
    current_slot: usize,
    finger_count: u8,
    max_fingers: u8,
    moved: bool,
    physical_clicked: bool,
    start_x: Option<i32>,
    start_y: Option<i32>,
    last_x: Option<i32>,
    last_y: Option<i32>,
    touch_start: Option<std::time::Instant>,
}

impl TouchpadState {
    fn reset_finger(&mut self) {
        self.current_slot = 0;
        self.start_x = None;
        self.start_y = None;
        self.last_x = None;
        self.last_y = None;
        self.touch_start = None;
        self.moved = false;
        self.physical_clicked = false;
        self.max_fingers = 0;
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
    tp_state: &mut TouchpadState,
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

                match code_u16 {
                    0x14a => {
                        // BTN_TOUCH
                        if value == 1 {
                            tp_state.touching = true;
                            tp_state.touch_start = Some(std::time::Instant::now());
                            tp_state.moved = false;
                            tp_state.max_fingers = tp_state.finger_count.max(1);
                        } else if value == 0 {
                            // Detect quick tap-to-click (< 250ms without large movement or physical ClickPad press)
                            if !tp_state.physical_clicked
                                && let Some(start) = tp_state.touch_start
                                && !tp_state.moved
                                && start.elapsed() < Duration::from_millis(300)
                            {
                                let role = match tp_state.max_fingers {
                                    2 => "right",
                                    3 => "middle",
                                    _ => "left",
                                };
                                stats.record_click(role);
                            }
                            tp_state.touching = false;
                            tp_state.reset_finger();
                        }
                        continue;
                    }
                    0x145 => {
                        // BTN_TOOL_FINGER (1 finger)
                        tp_state.finger_count = if value == 1 { 1 } else { 0 };
                        tp_state.max_fingers = tp_state.max_fingers.max(tp_state.finger_count);
                        continue;
                    }
                    0x14d => {
                        // BTN_TOOL_DOUBLETAP (2 fingers)
                        tp_state.finger_count = if value == 1 { 2 } else { 0 };
                        tp_state.max_fingers = tp_state.max_fingers.max(tp_state.finger_count);
                        continue;
                    }
                    0x14e => {
                        // BTN_TOOL_TRIPLETAP (3 fingers)
                        tp_state.finger_count = if value == 1 { 3 } else { 0 };
                        tp_state.max_fingers = tp_state.max_fingers.max(tp_state.finger_count);
                        continue;
                    }
                    0x14f => {
                        // BTN_TOOL_QUADTAP (4 fingers)
                        tp_state.finger_count = if value == 1 { 4 } else { 0 };
                        tp_state.max_fingers = tp_state.max_fingers.max(tp_state.finger_count);
                        continue;
                    }
                    _ => {}
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
                            let effective_role = if role == "left" && tp_state.touching {
                                tp_state.physical_clicked = true;
                                match tp_state.max_fingers.max(tp_state.finger_count) {
                                    2 => "right",
                                    3 => "middle",
                                    _ => "left",
                                }
                            } else {
                                role
                            };
                            stats.record_click(effective_role);
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
                                        let _ =
                                            tx.send(InputEvent::KeyPress { name: name.clone() });
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
                            let name = held_keys
                                .remove(&code_u16)
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

            EventSummary::AbsoluteAxis(_, code, value) => {
                count += 1;
                match code.0 {
                    0x2f => {
                        // ABS_MT_SLOT: track active finger slot so slot 1+ never overwrites slot 0
                        tp_state.current_slot = value.max(0) as usize;
                    }
                    0x39 => {
                        // ABS_MT_TRACKING_ID: -1 means finger lifted from current slot
                        if value < 0 && tp_state.current_slot == 0 {
                            tp_state.last_x = None;
                            tp_state.last_y = None;
                        }
                    }
                    0x00 | 0x35 => {
                        // ABS_X (0x00) or ABS_MT_POSITION_X (0x35): only track primary slot 0
                        if code.0 == 0x00 || tp_state.current_slot == 0 {
                            if let Some(prev_x) = tp_state.last_x {
                                let dx = (value - prev_x) as f64;
                                // Scale down high-DPI touchpad units (~0.35x) to roughly match mouse pixels
                                let scaled_dx = dx * 0.35;
                                let sx = *tp_state.start_x.get_or_insert(prev_x);
                                if ((value - sx) as f64 * 0.35).abs() > 35.0 {
                                    tp_state.moved = true;
                                }
                                if tp_state.finger_count >= 2 {
                                    stats.add_scroll_distance(scaled_dx * 0.2);
                                } else {
                                    pending_dx += scaled_dx;
                                }
                            }
                            tp_state.last_x = Some(value);
                        }
                    }
                    0x01 | 0x36 => {
                        // ABS_Y (0x01) or ABS_MT_POSITION_Y (0x36): only track primary slot 0
                        if code.0 == 0x01 || tp_state.current_slot == 0 {
                            if let Some(prev_y) = tp_state.last_y {
                                let dy = (value - prev_y) as f64;
                                let scaled_dy = dy * 0.35;
                                let sy = *tp_state.start_y.get_or_insert(prev_y);
                                if ((value - sy) as f64 * 0.35).abs() > 35.0 {
                                    tp_state.moved = true;
                                }
                                if tp_state.finger_count >= 2 {
                                    stats.add_scroll_distance(scaled_dy.abs() * 0.005);
                                } else {
                                    pending_dy += scaled_dy;
                                }
                            }
                            tp_state.last_y = Some(value);
                        }
                    }
                    _ => {}
                }
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
    let mut touchpad_states: HashMap<String, TouchpadState> = HashMap::new();

    loop {
        let mut total = 0usize;
        {
            let Some(mut mgr) = lock_stats(&stats) else {
                continue;
            };
            for device in &mut devices {
                let tp_state = touchpad_states.entry(device.path.clone()).or_default();
                total += process_device(
                    device,
                    &mut mgr,
                    &event_tx,
                    &mut modifiers,
                    &mut held_keys,
                    tp_state,
                );
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
