mod db;
mod dbus;
mod input;
mod permissions;
mod stats;

use std::sync::{Arc, Mutex, mpsc};

use keystats_core::InputEvent;
use stats::manager::StatsManager;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("keystats-daemon v{} starting...", env!("CARGO_PKG_VERSION"));

    // Permission diagnostic on startup
    let perm = permissions::diagnose();
    tracing::info!(
        "Permissions: readable={} blocked={} action={}",
        perm.readable_devices,
        perm.blocked_devices,
        perm.recommended_action
    );
    if !perm.can_read_any_input {
        tracing::error!("Cannot read any input devices. Check 'input' group membership.");
    }

    let manager = match StatsManager::new() {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to initialize StatsManager: {e}");
            std::process::exit(1);
        }
    };

    let snapshot = manager.snapshot();
    tracing::info!(
        "Loaded today stats — keys:{} clicks:{} distance:{:.0} scroll:{:.0}",
        snapshot.key_presses,
        snapshot.total_clicks(),
        snapshot.mouse_distance,
        snapshot.scroll_distance
    );

    let stats = Arc::new(Mutex::new(manager));

    // Channel for real-time input events (event loop → D-Bus signals)
    let (event_tx, event_rx) = mpsc::channel::<InputEvent>();

    // Start D-Bus service on background thread with signal forwarding
    dbus::service::KeyStatsService::start(stats.clone(), event_rx);

    // Run input event loop on main thread (blocks until killed)
    tracing::info!("Entering input event loop...");
    input::event_loop::run(stats, Some(event_tx));
}
