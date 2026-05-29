use evdev::Device;
use keystats_core::DailyStats;
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

// ── History chart ──────────────────────────────────────

const BLOCKS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

fn fmt_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

fn render_bar(value: u64, max_value: u64, bar_width: usize) -> String {
    if max_value == 0 || bar_width == 0 {
        return " ".repeat(bar_width);
    }
    let ratio = value as f64 / max_value as f64;
    let total_eighths = (ratio * bar_width as f64 * 8.0).round() as usize;
    let full_blocks = total_eighths / 8;
    let remainder = total_eighths % 8;

    let mut s = String::with_capacity(bar_width);
    for _ in 0..full_blocks.min(bar_width) {
        s.push('█');
    }
    if full_blocks < bar_width && remainder > 0 {
        s.push(BLOCKS[remainder - 1]);
    }
    while s.chars().count() < bar_width {
        s.push(' ');
    }
    s
}

fn render_chart(title: &str, data: &[(String, u64)]) {
    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
    let width = terminal_width();
    // date(6) + gap(2) + bar + gap(1) + value(6)
    let bar_width = width.saturating_sub(15).max(10);

    println!("{}:", title);
    for (date, value) in data {
        let date_short = if date.len() >= 5 { &date[date.len() - 5..] } else { date };
        let bar = render_bar(*value, max_value, bar_width);
        let val_str = fmt_num(*value);
        println!("{}  {} {}", date_short, bar, val_str);
    }
}

fn fetch_history(days: u32) -> Result<Vec<DailyStats>, String> {
    let conn = zbus::blocking::Connection::session()
        .map_err(|e| format!("Failed to connect to D-Bus: {}", e))?;

    let reply = conn
        .call_method(
            Some(BUS_NAME),
            OBJ_PATH,
            Some(IFACE),
            "GetHistory",
            &(days,),
        )
        .map_err(|e| format!("D-Bus call failed: {}", e))?;

    let json: String = reply
        .body()
        .deserialize()
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    serde_json::from_str(&json).map_err(|e| format!("Failed to parse history JSON: {}", e))
}

pub fn history(days: u32, show_keys: bool, show_clicks: bool) {
    let data = match fetch_history(days) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };

    if data.is_empty() {
        println!("No history data available.");
        return;
    }

    let show_mixed = !show_keys && !show_clicks;

    // Summary totals
    let total_keys: u64 = data.iter().map(|d| d.key_presses).sum();
    let total_clicks: u64 = data.iter().map(|d| d.total_clicks()).sum();
    let date_range = format!("{} ~ {}", data.last().unwrap().date, data.first().unwrap().date);

    println!("History ({} days, {})", days, date_range);
    if show_mixed || show_keys {
        print!("  Keys: {}", fmt_num(total_keys));
    }
    if show_mixed || show_clicks {
        if show_mixed || show_keys {
            print!("  |  ");
        }
        print!("Clicks: {}", fmt_num(total_clicks));
    }
    println!("\n");

    // Prepare key press data (newest first)
    let mut key_data: Vec<(String, u64)> = data
        .iter()
        .map(|d| (d.date.clone(), d.key_presses))
        .collect();
    key_data.reverse();

    // Prepare click data
    let mut click_data: Vec<(String, u64)> = data
        .iter()
        .map(|d| (d.date.clone(), d.total_clicks()))
        .collect();
    click_data.reverse();

    if show_mixed || show_keys {
        render_chart("Key presses", &key_data);
    }
    if show_mixed {
        println!();
    }
    if show_mixed || show_clicks {
        render_chart("Mouse clicks", &click_data);
    }
}
