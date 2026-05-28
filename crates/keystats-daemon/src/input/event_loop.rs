use std::sync::{Arc, Mutex};
use std::time::Duration;

use evdev::EventSummary;

use super::device::{DeviceKind, InputDevice};
use super::keymap;
use crate::stats::manager::StatsManager;

/// Process a batch of events from one device, calling the appropriate
/// StatsManager recording methods. Returns the number of events processed.
fn process_device(device: &mut InputDevice, stats: &mut StatsManager) -> usize {
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

                match value {
                    1 => {
                        // Check for mouse button first — some mice report as
                        // KeyboardPointer (they have both keys and relative axes),
                        // and BTN_LEFT etc. must always be clicks, not key presses.
                        if let Some(role) = keymap::button_role(code_u16) {
                            stats.record_click(role);
                        } else {
                            match device.kind {
                                DeviceKind::Keyboard | DeviceKind::KeyboardPointer => {
                                    let name = keymap::key_name(code_u16);
                                    stats.record_key_press(&name, true);
                                }
                                DeviceKind::Pointer | DeviceKind::Other => {
                                    // Non-button key on a pointer — ignore
                                }
                            }
                        }
                    }
                    2 => {
                        // Auto-repeat — skip (matching macOS behavior)
                    }
                    _ => {
                        // Key release — ignore for counting
                    }
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
pub fn run(stats: Arc<Mutex<StatsManager>>) -> ! {
    let (mut devices, _blocked) = InputDevice::discover();

    tracing::info!(
        "Event loop started with {} devices ({} keyboard, {} pointer)",
        devices.len(),
        devices
            .iter()
            .filter(|d| matches!(d.kind, DeviceKind::Keyboard))
            .count(),
        devices
            .iter()
            .filter(|d| matches!(d.kind, DeviceKind::Pointer | DeviceKind::KeyboardPointer))
            .count()
    );

    let poll_interval = Duration::from_millis(8); // ~125 Hz
    let mut last_rescan = std::time::Instant::now();

    loop {
        let mut total = 0usize;
        {
            let mut mgr = stats.lock().unwrap();
            for device in &mut devices {
                total += process_device(device, &mut mgr);
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
