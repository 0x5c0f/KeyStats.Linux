mod config;
mod dbus_client;
mod overlay;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

use gtk::glib;
use gtk::prelude::*;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("keystats-overlay v{} starting...", env!("CARGO_PKG_VERSION"));

    let config = config::Config::from_args();
    tracing::info!(
        "Config: position={:?} max_keys={} fade_duration={}ms font_size={} opacity={}%",
        config.position,
        config.max_keys,
        config.fade_duration_ms,
        config.font_size,
        config.opacity
    );

    // Channel for key events: D-Bus thread → GTK main loop
    let (tx, rx) = mpsc::channel::<dbus_client::KeyEvent>();

    // Spawn D-Bus signal listener on a background thread
    thread::spawn(move || {
        dbus_client::subscribe_key_events(tx);
    });

    // Make rx available to the GTK callback via Rc<RefCell<>>
    let rx = Rc::new(RefCell::new(rx));

    // Create and run GTK application
    let app = gtk::Application::builder()
        .application_id("io.github.x0x5c0f.KeyStats.Overlay")
        .build();

    let rx_for_activate = rx.clone();
    app.connect_activate(move |app| {
        let state = overlay::build_ui(app, &config);

        // Poll the mpsc channel every 16ms (~60fps) for key events
        let rx_ref = rx_for_activate.clone();
        let state_ref = state.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            // Drain all pending key events
            while let Ok(event) = rx_ref.borrow().try_recv() {
                state_ref.borrow_mut().handle_key_event(&event);
            }
            glib::ControlFlow::Continue
        });
    });

    app.run_with_args::<String>(&[]);
}
