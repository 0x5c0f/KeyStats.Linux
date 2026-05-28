use evdev::Device;

/// Known device types for event pipeline routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    Keyboard,
    Pointer,
    /// Device has both key and relative capabilities
    KeyboardPointer,
    Other,
}

/// Wrapper around an evdev Device with classification info.
#[allow(dead_code)]
pub struct InputDevice {
    pub path: String,
    pub name: String,
    pub kind: DeviceKind,
    pub device: Device,
}

impl InputDevice {
    pub fn open(path: &str) -> Result<Option<Self>, std::io::Error> {
        let device = match Device::open(path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(None),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };

        let name = device.name().unwrap_or("unknown").to_string();
        let caps = device.supported_events();

        let has_keys = caps.contains(evdev::EventType::KEY);
        let has_relative = caps.contains(evdev::EventType::RELATIVE);
        let key_count = device
            .supported_keys()
            .map(|k| k.iter().count())
            .unwrap_or(0);

        // Filters: skip devices that can't contribute to our metrics
        if !has_keys && !has_relative {
            return Ok(None);
        }

        let kind = match (has_keys, has_relative, key_count) {
            (true, true, _) => DeviceKind::KeyboardPointer,
            (true, false, k) if k > 10 => DeviceKind::Keyboard,
            (true, false, _) => DeviceKind::Other, // buttons-only (sleep, power, etc.)
            (false, true, _) => DeviceKind::Pointer,
            (false, false, _) => DeviceKind::Other,
        };

        // Set non-blocking for event loop
        device.set_nonblocking(true).ok();

        Ok(Some(Self {
            path: path.to_string(),
            name,
            kind,
            device,
        }))
    }

    /// Scan all /dev/input/event* nodes and return classified devices.
    pub fn discover() -> (Vec<Self>, Vec<String>) {
        let mut devices = Vec::new();
        let mut blocked = Vec::new();

        for i in 0..64 {
            let path = format!("/dev/input/event{}", i);
            match Self::open(&path) {
                Ok(Some(d)) => devices.push(d),
                Ok(None) => {
                    // Try to determine if it's blocked
                    if std::path::Path::new(&path).exists() {
                        match Device::open(&path) {
                            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                                blocked.push(path);
                            }
                            _ => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }

        devices.sort_by_key(|d| match d.kind {
            DeviceKind::Keyboard => 0,
            DeviceKind::KeyboardPointer => 1,
            DeviceKind::Pointer => 2,
            _ => 3,
        });

        (devices, blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_devices() {
        let (devices, _blocked) = InputDevice::discover();
        // On a real Linux desktop, there should be at least a keyboard
        let keyboard_count = devices
            .iter()
            .filter(|d| d.kind == DeviceKind::Keyboard)
            .count();
        let pointer_count = devices
            .iter()
            .filter(|d| d.kind == DeviceKind::Pointer || d.kind == DeviceKind::KeyboardPointer)
            .count();
        // At minimum we expect something on a real system
        eprintln!(
            "Discovered {} keyboards, {} pointers",
            keyboard_count, pointer_count
        );
    }
}
