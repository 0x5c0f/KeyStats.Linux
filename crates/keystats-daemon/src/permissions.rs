use keystats_core::PermissionStatus;

/// Scan input devices and return a permission diagnostic.
pub fn diagnose() -> PermissionStatus {
    let (devices, blocked) = crate::input::device::InputDevice::discover();

    let can_read = !devices.is_empty();
    let (message, action) = if blocked.is_empty() && can_read {
        (
            "All input devices are readable.".to_string(),
            "none".to_string(),
        )
    } else if !blocked.is_empty() {
        (
            format!(
                "{} device(s) blocked. Add user to 'input' group or install a udev rule. Try: sudo usermod -aG input $USER && newgrp input",
                blocked.len()
            ),
            "add_group".to_string(),
        )
    } else {
        (
            "No input devices found. Check that evdev kernel module is loaded.".to_string(),
            "check_driver".to_string(),
        )
    };

    PermissionStatus {
        can_read_any_input: can_read,
        readable_devices: devices.len() as u32,
        blocked_devices: blocked.len() as u32,
        recommended_action: action,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnose_works_on_real_system() {
        let status = diagnose();
        // On a real Linux desktop, we expect at least readable devices
        // or a meaningful error message
        eprintln!(
            "Diagnose: readable={} blocked={} action={}",
            status.readable_devices, status.blocked_devices, status.recommended_action
        );
        assert!(
            status.readable_devices > 0,
            "No readable input devices — check permissions"
        );
    }
}
