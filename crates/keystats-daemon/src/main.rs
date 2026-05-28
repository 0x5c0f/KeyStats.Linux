mod db;
mod dbus;
mod input;
mod permissions;
mod stats;

use std::sync::{Arc, Mutex};

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

    let manager = StatsManager::new().expect("Failed to initialize StatsManager");

    let snapshot = manager.snapshot();
    tracing::info!(
        "Loaded today stats — keys:{} clicks:{} distance:{:.0} scroll:{:.0}",
        snapshot.key_presses,
        snapshot.total_clicks(),
        snapshot.mouse_distance,
        snapshot.scroll_distance
    );

    let stats = Arc::new(Mutex::new(manager));

    // Start D-Bus service on background thread
    let _dbus_handle = dbus::service::KeyStatsService::start(stats.clone());

    // Run input event loop on main thread (blocks until killed)
    tracing::info!("Entering input event loop...");
    input::event_loop::run(stats);
}
