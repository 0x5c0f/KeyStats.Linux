use evdev::Device;

pub fn status() {
    println!("keystats daemon status: not connected (stub)");
    println!("Today stats:");
    println!("  Key presses: 0");
    println!("  Total clicks: 0");
    println!("  Mouse distance: 0");
    println!("  Scroll distance: 0");
    println!("  KPS: 0 (peak 0)  CPS: 0 (peak 0)");
}

pub fn doctor() {
    println!("Permission diagnostic:\n");

    let mut readable = 0u32;
    let mut blocked = 0u32;
    let mut keyboards = Vec::new();
    let mut pointers = Vec::new();
    let mut other = Vec::new();

    for i in 0..64 {
        let path = format!("/dev/input/event{}", i);
        if !std::path::Path::new(&path).exists() {
            continue;
        }

        match Device::open(&path) {
            Ok(device) => {
                readable += 1;
                let name = device.name().unwrap_or("unknown").to_string();
                let caps = device.supported_events();
                let has_keys = caps.contains(evdev::EventType::KEY);
                let has_rel = caps.contains(evdev::EventType::RELATIVE);
                let key_count = device
                    .supported_keys()
                    .map(|k| k.iter().count())
                    .unwrap_or(0);

                match (has_keys, has_rel, key_count) {
                    (true, true, _) => pointers.push(format!("  {} (keyboard+pointer)", name)),
                    (true, false, k) if k > 10 => keyboards.push(format!("  {}", name)),
                    (true, false, _) => other.push(format!("  {} (buttons)", name)),
                    (false, true, _) => pointers.push(format!("  {}", name)),
                    _ => other.push(format!("  {} (other)", name)),
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                blocked += 1;
                eprintln!("  BLOCKED: {} — permission denied", path);
            }
            _ => {}
        }
    }

    println!("  Readable devices: {}", readable);
    if !keyboards.is_empty() {
        println!("\n  Keyboards:");
        for k in &keyboards {
            println!("{}", k);
        }
    }
    if !pointers.is_empty() {
        println!("\n  Pointers:");
        for p in &pointers {
            println!("{}", p);
        }
    }
    if !other.is_empty() {
        println!("\n  Other:");
        for o in &other {
            println!("{}", o);
        }
    }

    println!("\n  Blocked devices: {}", blocked);
    if blocked > 0 {
        println!("\n  Recommended action:");
        println!("    sudo usermod -aG input $USER");
        println!("    newgrp input  # or log out and back in");
    } else if readable > 0 {
        println!("\n  Status: OK — all input devices are readable.");
    } else {
        println!("\n  Status: No input devices found.");
        println!("  Check: ls /dev/input/event*");
    }
}
