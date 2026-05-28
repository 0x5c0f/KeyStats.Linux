use evdev::Device;
use std::collections::HashMap;
use zbus::zvariant::Value;

const BUS_NAME: &str = "io.github.x0x5c0f.KeyStats";
const OBJ_PATH: &str = "/io/github/0x5c0f/KeyStats";
const IFACE: &str = "io.github.x0x5c0f.KeyStats1";

fn get_u64(map: &HashMap<String, Value<'_>>, key: &str) -> u64 {
    match map.get(key) {
        Some(Value::U64(v)) => *v,
        _ => 0,
    }
}

fn get_f64(map: &HashMap<String, Value<'_>>, key: &str) -> f64 {
    match map.get(key) {
        Some(Value::F64(v)) => *v,
        _ => 0.0,
    }
}

fn get_u32(map: &HashMap<String, Value<'_>>, key: &str) -> u32 {
    match map.get(key) {
        Some(Value::U32(v)) => *v,
        _ => 0,
    }
}

pub fn status() {
    let conn = match zbus::blocking::Connection::session() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to D-Bus session bus: {}", e);
            return;
        }
    };

    let reply = conn.call_method(
        Some(BUS_NAME),
        OBJ_PATH,
        Some(IFACE),
        "GetTodayStats",
        &(),
    );

    let body = match reply {
        Ok(msg) => msg,
        Err(e) => {
            println!("keystats daemon status: not connected");
            eprintln!("  ({})", e);
            return;
        }
    };

    let body_inner = body.body();
    let stats: HashMap<String, Value<'_>> = match body_inner.deserialize() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse daemon response: {}", e);
            return;
        }
    };

    let key_presses = get_u64(&stats, "keyPresses");
    let total_clicks = get_u64(&stats, "totalClicks");
    let mouse_dist = get_f64(&stats, "mouseDistance");
    let scroll_dist = get_f64(&stats, "scrollDistance");
    let kps = get_u32(&stats, "currentKPS");
    let peak_kps = get_u32(&stats, "peakKPS");
    let cps = get_u32(&stats, "currentCPS");
    let peak_cps = get_u32(&stats, "peakCPS");

    println!("keystats daemon status: connected");
    println!("Today stats:");
    println!("  Key presses: {}", key_presses);
    println!("  Total clicks: {}", total_clicks);
    println!("  Mouse distance: {:.0}", mouse_dist);
    println!("  Scroll distance: {:.0}", scroll_dist);
    println!("  KPS: {} (peak {})  CPS: {} (peak {})", kps, peak_kps, cps, peak_cps);
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
