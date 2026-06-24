use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

use gtk::glib;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box as GtkBox, CssProvider, Label, Orientation};

use crate::config::{Config, Position};
use crate::dbus_client::KeyEvent;

/// Maximum time a key can stay in "held" state before auto-release.
/// Safety net for lost D-Bus release signals.
const MAX_HOLD_MS: u64 = 5000;

/// Modifier key names that should be removed when a combo key arrives.
const MODIFIER_NAMES: &[&str] = &[
    "LeftShift", "RightShift",
    "LeftControl", "RightControl",
    "LeftAlt", "RightAlt",
    "LeftMeta", "RightMeta",
];

/// A displayed key entry with its state.
struct KeyEntry {
    label: Label,
    /// When the key was pressed (for timeout detection).
    pressed_at: Instant,
    /// When the key was released (None = still held).
    released_at: Option<Instant>,
    /// Current opacity (1.0 → 0.0 during fade).
    opacity: f64,
}

/// Shared state for the overlay.
pub struct OverlayState {
    keys: VecDeque<KeyEntry>,
    config: Config,
    container: GtkBox,
    window: ApplicationWindow,
    position: Position,
    margin_x: i32,
    margin_y: i32,
}

impl OverlayState {
    /// Handle a key event (press or release).
    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::Press(name) => self.add_key(name),
            KeyEvent::Release(name) => self.release_key(name),
        }
    }

    /// Add a key to the overlay display.
    fn add_key(&mut self, key_name: &str) {
        // If this key is already displayed (held), don't add duplicate
        if self.keys.iter().any(|e| e.label.text().as_str() == key_name && e.released_at.is_none()) {
            return;
        }

        // When adding a combo key (e.g. "Ctrl+C"), remove standalone modifier entries
        // that would be redundant (e.g. "LeftControl")
        if key_name.contains('+') {
            self.remove_modifier_entries();
        }

        let label = Label::new(Some(key_name));
        label.add_css_class("key-label");
        label.set_xalign(0.5);
        label.set_opacity(1.0);

        self.container.append(&label);
        self.keys.push_back(KeyEntry {
            label,
            pressed_at: Instant::now(),
            released_at: None,
            opacity: 1.0,
        });

        // Enforce max_keys: remove oldest entries (including mid-fade)
        while self.keys.len() > self.config.max_keys {
            if let Some(entry) = self.keys.pop_front() {
                self.container.remove(&entry.label);
            }
        }

        reposition_window(&self.window, &self.container, self.position, self.margin_x, self.margin_y);
    }

    /// Remove standalone modifier entries (e.g. "LeftControl") from the queue.
    /// Called when a combo key is added to avoid showing both the modifier and the combo.
    fn remove_modifier_entries(&mut self) {
        let mut i = 0;
        while i < self.keys.len() {
            let label = self.keys[i].label.text();
            if MODIFIER_NAMES.iter().any(|&m| m == label.as_str()) {
                if let Some(entry) = self.keys.remove(i) {
                    self.container.remove(&entry.label);
                }
            } else {
                i += 1;
            }
        }
    }

    /// Mark a key as released (start fade animation).
    ///
    /// Matching strategy:
    /// 1. Exact match on label (e.g. release "Ctrl+C" matches label "Ctrl+C")
    /// 2. Fallback: match base key in combo (e.g. release "C" matches label "Ctrl+C")
    fn release_key(&mut self, key_name: &str) {
        // First try exact match
        if let Some(entry) = self.keys.iter_mut().rev().find(|e| {
            e.label.text().as_str() == key_name && e.released_at.is_none()
        }) {
            entry.released_at = Some(Instant::now());
            return;
        }

        // Fallback: find combo entry where the base key matches
        // e.g. "Ctrl+C" has base key "C", which matches release "C"
        if let Some(entry) = self.keys.iter_mut().rev().find(|e| {
            if e.released_at.is_some() {
                return false;
            }
            let label = e.label.text();
            label.contains('+') && label.split('+').next_back() == Some(key_name)
        }) {
            entry.released_at = Some(Instant::now());
        }
    }

    /// Tick the fade animation. Called periodically (~60fps).
    /// Returns `true` if any keys were removed (window needs reposition).
    fn tick_fade(&mut self) -> bool {
        let now = Instant::now();
        let fade_ms = self.config.fade_duration_ms as f64;
        let max_hold = std::time::Duration::from_millis(MAX_HOLD_MS);
        let mut removed = false;

        // Auto-release entries held too long (safety net for lost release signals)
        for entry in &mut self.keys {
            if entry.released_at.is_none() && now.duration_since(entry.pressed_at) >= max_hold {
                tracing::warn!(
                    "Auto-releasing held key '{}' after {}ms (release signal likely lost)",
                    entry.label.text(),
                    MAX_HOLD_MS
                );
                entry.released_at = Some(now);
            }
        }

        // Update opacity for released keys
        for entry in &mut self.keys {
            if let Some(released_at) = entry.released_at {
                let elapsed = now.duration_since(released_at).as_millis() as f64;
                let fade_progress = elapsed / fade_ms;
                entry.opacity = (1.0 - fade_progress).max(0.0);
                entry.label.set_opacity(entry.opacity);
            }
        }

        // Remove ALL fully faded keys (not just front — held keys may block the front)
        let mut i = 0;
        while i < self.keys.len() {
            if self.keys[i].opacity <= 0.0 {
                if let Some(entry) = self.keys.remove(i) {
                    self.container.remove(&entry.label);
                    removed = true;
                }
            } else {
                i += 1;
            }
        }

        removed
    }
}

/// Build the UI and return the shared overlay state.
pub fn build_ui(app: &Application, config: &Config) -> Rc<RefCell<OverlayState>> {
    let Some(display) = gtk::gdk::Display::default() else {
        tracing::error!("No display available, cannot build overlay UI");
        // Return a dummy state that won't render anything
        let window = ApplicationWindow::builder().application(app).build();
        let container = GtkBox::new(Orientation::Vertical, 4);
        window.set_child(Some(&container));
        return Rc::new(RefCell::new(OverlayState {
            keys: VecDeque::new(),
            config: config.clone(),
            container,
            window,
            position: config.position,
            margin_x: 20,
            margin_y: 40,
        }));
    };

    let provider = CssProvider::new();
    provider.load_from_data(&css_string(config));
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .title("KeyStats Overlay")
        .build();
    window.add_css_class("overlay-window");

    let container = GtkBox::new(Orientation::Vertical, 4);
    container.add_css_class("overlay-container");
    window.set_child(Some(&container));

    // Minimum size so the window is visible even when empty
    window.set_size_request(180, 40);

    let position = config.position;
    let margin_percent = config.margin_percent;

    // Calculate margins from screen size
    let (margin_x, margin_y) = if let Some(monitor) = display
        .monitors()
        .item(0)
        .and_then(|m| m.downcast::<gtk::gdk::Monitor>().ok())
    {
        let geo = monitor.geometry();
        let mx = geo.width() * margin_percent as i32 / 100 / 2;
        let my = geo.height() * margin_percent as i32 / 100;
        (mx, my)
    } else {
        (20, 40)
    };

    // Platform-specific window setup
    setup_platform_window(&window, &container, position, margin_x, margin_y);

    window.present();

    let state = Rc::new(RefCell::new(OverlayState {
        keys: VecDeque::new(),
        config: config.clone(),
        container,
        window: window.clone(),
        position,
        margin_x,
        margin_y,
    }));

    // Start the fade animation timer (~60fps)
    let state_ref = state.clone();
    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let reposition = state_ref.borrow_mut().tick_fade();
        if reposition {
            let s = state_ref.borrow();
            reposition_window(&s.window, &s.container, s.position, s.margin_x, s.margin_y);
        }
        glib::ControlFlow::Continue
    });

    state
}

/// Platform-specific window setup.
///
/// On X11: uses override_redirect to bypass window manager.
/// On Wayland: uses gtk4-layer-shell if available, falls back to keep_above.
fn setup_platform_window(
    window: &ApplicationWindow,
    container: &GtkBox,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    let Some(display) = gtk::gdk::Display::default() else {
        tracing::warn!("No display for platform detection, using fallback");
        setup_fallback_window(window, container, position, margin_x, margin_y);
        return;
    };
    let display_type = display.type_();

    if display_type == gdk4_x11::X11Display::static_type() {
        tracing::info!("Detected X11 display, using override_redirect");
        setup_x11_window(window, container, position, margin_x, margin_y);
    } else {
        tracing::info!("Detected Wayland display");
        #[cfg(feature = "wayland")]
        setup_wayland_window(window, position, margin_x, margin_y);

        #[cfg(not(feature = "wayland"))]
        {
            tracing::info!("Wayland layer-shell not available, using fallback");
            setup_fallback_window(window, container, position, margin_x, margin_y);
        }
    }
}

/// X11-specific setup: override_redirect + absolute positioning.
#[cfg(feature = "x11")]
fn setup_x11_window(
    window: &ApplicationWindow,
    container: &GtkBox,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    // Set override_redirect before the window is mapped
    window.connect_realize(|window| {
        set_x11_override_redirect(window);
    });

    // Position after mapping
    let win = window.clone();
    let ctr = container.clone();
    window.connect_map(move |_| {
        let w = win.clone();
        let c = ctr.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            reposition_window(&w, &c, position, margin_x, margin_y);
            glib::ControlFlow::Break
        });
    });
}

/// Set X11 override_redirect to bypass window manager.
///
/// Called in `connect_realize` — after the X11 window is created but before
/// it is mapped. This prevents the window manager from managing the window,
/// which eliminates GNOME Shell opening animation (white flash).
#[cfg(feature = "x11")]
#[allow(unsafe_code)]
fn set_x11_override_redirect(window: &ApplicationWindow) {
    use std::mem;

    let surface = match window.surface() {
        Some(s) => s,
        None => return,
    };
    let x11_surface = match surface.downcast_ref::<gdk4_x11::X11Surface>() {
        Some(s) => s,
        None => return,
    };

    let xid = x11_surface.xid();

    // SAFETY:
    // - `xlib.XOpenDisplay` returns a valid display pointer or null (checked below)
    // - `xid` is a valid X11 window ID obtained from GDK's X11Surface
    // - Display connection is closed after use to prevent resource leaks
    // - XChangeWindowAttributes only modifies the override_redirect flag
    unsafe {
        let xlib = match x11_dl::xlib::Xlib::open() {
            Ok(x) => x,
            Err(e) => {
                tracing::warn!("Failed to open Xlib: {e}");
                return;
            }
        };

        let display_ptr = (xlib.XOpenDisplay)(std::ptr::null());
        if display_ptr.is_null() {
            tracing::warn!("XOpenDisplay failed");
            return;
        }

        let mut attrs: x11_dl::xlib::XSetWindowAttributes = mem::zeroed();
        attrs.override_redirect = x11_dl::xlib::True;
        (xlib.XChangeWindowAttributes)(
            display_ptr,
            xid,
            x11_dl::xlib::CWOverrideRedirect,
            &mut attrs,
        );

        (xlib.XFlush)(display_ptr);
        (xlib.XCloseDisplay)(display_ptr);
    }

    tracing::info!("Set X11 override_redirect for overlay");
}

/// Wayland-specific setup using gtk4-layer-shell.
#[cfg(feature = "wayland")]
fn setup_wayland_window(
    window: &ApplicationWindow,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    gtk4_layer_shell::init_for_window(window);
    gtk4_layer_shell::set_layer(window, gtk4_layer_shell::Layer::Overlay);
    gtk4_layer_shell::set_keyboard_interactivity(window, false);

    // Set anchors based on position
    let (top, bottom, left, right) = match position {
        Position::TopLeft => (true, false, true, false),
        Position::TopRight => (true, false, false, true),
        Position::BottomLeft => (false, true, true, false),
        Position::BottomRight => (false, true, false, true),
    };
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Top, top);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Bottom, bottom);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Left, left);
    gtk4_layer_shell::set_anchor(window, gtk4_layer_shell::Edge::Right, right);

    gtk4_layer_shell::set_margin(window, gtk4_layer_shell::Edge::Top, margin_y);
    gtk4_layer_shell::set_margin(window, gtk4_layer_shell::Edge::Bottom, margin_y);
    gtk4_layer_shell::set_margin(window, gtk4_layer_shell::Edge::Left, margin_x);
    gtk4_layer_shell::set_margin(window, gtk4_layer_shell::Edge::Right, margin_x);

    tracing::info!("Configured Wayland layer-shell overlay");
}

/// Fallback setup for Wayland without layer-shell.
/// Uses a basic borderless window. Positioning depends on the compositor.
fn setup_fallback_window(
    window: &ApplicationWindow,
    container: &GtkBox,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    // Position after mapping
    let win = window.clone();
    let ctr = container.clone();
    window.connect_map(move |_| {
        let w = win.clone();
        let c = ctr.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            reposition_window(&w, &c, position, margin_x, margin_y);
            glib::ControlFlow::Break
        });
    });
}

/// Reposition the overlay window to fit its content at the configured corner.
///
/// Uses XMoveResizeWindow on X11 for atomic resize+move.
/// On Wayland/fallback, uses GTK margin/alignment.
fn reposition_window(
    window: &ApplicationWindow,
    container: &GtkBox,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    let Some(display) = gtk::gdk::Display::default() else {
        return;
    };
    let display_type = display.type_();

    if display_type == gdk4_x11::X11Display::static_type() {
        #[cfg(feature = "x11")]
        reposition_x11(window, container, position, margin_x, margin_y);
    }
    // On Wayland with layer-shell, positioning is handled by the protocol
}

/// X11 reposition using XMoveResizeWindow.
///
/// Atomically resizes and moves the overlay window to keep it anchored
/// at the configured corner. Called when keys are added or removed.
#[cfg(feature = "x11")]
#[allow(unsafe_code)]
fn reposition_x11(
    window: &ApplicationWindow,
    container: &GtkBox,
    position: Position,
    margin_x: i32,
    margin_y: i32,
) {
    let surface = match window.surface() {
        Some(s) => s,
        None => return,
    };
    let x11_surface = match surface.downcast_ref::<gdk4_x11::X11Surface>() {
        Some(s) => s,
        None => return,
    };

    let (_, nat_size) = container.preferred_size();
    let content_w = nat_size.width().max(180);
    let content_h = nat_size.height().max(40);

    // Get screen dimensions from the window's display
    let display = WidgetExt::display(window);
    let monitors = display.monitors();
    let monitor = match monitors
        .item(0)
        .and_then(|m| m.downcast::<gtk::gdk::Monitor>().ok())
    {
        Some(m) => m,
        None => return,
    };
    let geo = monitor.geometry();
    let screen_w = geo.width();
    let screen_h = geo.height();

    let (x, y) = match position {
        Position::TopLeft => (margin_x, margin_y),
        Position::TopRight => (screen_w - content_w - margin_x, margin_y),
        Position::BottomLeft => (margin_x, screen_h - content_h - margin_y),
        Position::BottomRight => (screen_w - content_w - margin_x, screen_h - content_h - margin_y),
    };

    let xid = x11_surface.xid();

    // SAFETY:
    // - `xid` is a valid X11 window ID from GDK's X11Surface
    // - XOpenDisplay returns a valid pointer or null (checked below)
    // - XMoveResizeWindow is a standard X11 call with valid coordinates
    // - Display connection is closed immediately after use
    unsafe {
        let xlib = match x11_dl::xlib::Xlib::open() {
            Ok(x) => x,
            Err(_) => return,
        };
        let display_ptr = (xlib.XOpenDisplay)(std::ptr::null());
        if display_ptr.is_null() {
            return;
        }
        (xlib.XMoveResizeWindow)(display_ptr, xid, x, y, content_w as u32, content_h as u32);
        (xlib.XFlush)(display_ptr);
        (xlib.XCloseDisplay)(display_ptr);
    }
}

/// Generate CSS for the overlay.
fn css_string(config: &Config) -> String {
    let bg_alpha = config.opacity as f64 / 100.0;
    format!(
        r#"
        window, .overlay-window {{
            background-color: transparent;
        }}
        .overlay-container {{
            background-color: rgba(0, 0, 0, {bg_alpha:.2});
            border-radius: 8px;
            padding: 8px;
        }}
        .key-label {{
            color: white;
            font-size: {font_size}px;
            font-weight: bold;
            font-family: monospace;
            padding: 2px 8px;
            background-color: rgba(255, 255, 255, 0.15);
            border-radius: 4px;
            margin: 1px 0;
        }}
        "#,
        font_size = config.font_size,
    )
}
