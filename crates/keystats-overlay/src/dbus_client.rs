use std::sync::mpsc;

use zbus::blocking::Connection;
use zbus::MatchRule;
use zbus::message::Type as MessageType;

/// D-Bus interface name for the KeyStats service.
const INTERFACE_NAME: &str = "io.github.x0x5c0f.KeyStats1";
/// D-Bus signal name for key press events.
const SIGNAL_PRESS: &str = "KeyPressed";

/// A key event received from D-Bus.
#[derive(Debug, Clone)]
pub enum KeyEvent {
    /// Key was pressed.
    Press(String),
    /// Key was released.
    Release(String),
}

/// Subscribe to D-Bus KeyPressed and KeyReleased signals.
/// Uses a single connection and iterator to avoid message loss from concurrent reads.
/// Blocks the calling thread.
pub fn subscribe_key_events(tx: mpsc::Sender<KeyEvent>) {
    let connection = match Connection::session() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to connect to D-Bus session: {e}");
            return;
        }
    };

    // Match ALL signals on our interface (both KeyPressed and KeyReleased)
    let rule = match MatchRule::builder()
        .msg_type(MessageType::Signal)
        .interface(INTERFACE_NAME)
    {
        Ok(b) => b.build(),
        Err(e) => {
            tracing::error!("Failed to create D-Bus match rule: {e}");
            return;
        }
    };

    let iter = match zbus::blocking::MessageIterator::for_match_rule(rule, &connection, None) {
        Ok(i) => i,
        Err(e) => {
            tracing::error!("Failed to create message iterator: {e}");
            return;
        }
    };

    tracing::info!("Subscribed to {INTERFACE_NAME} signals (KeyPressed + KeyReleased)");

    // Single iterator, dispatch by signal member name
    for msg in iter {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                tracing::trace!("D-Bus message error: {e}");
                continue;
            }
        };

        let member = msg.header().member().map(|m| m.as_str().to_string()).unwrap_or_default();
        let is_press = member == SIGNAL_PRESS;

        if let Ok(body) = msg.body().deserialize::<(String,)>() {
            let key_name = body.0;
            let event = if is_press {
                tracing::debug!("Key pressed: {key_name}");
                KeyEvent::Press(key_name)
            } else {
                tracing::debug!("Key released: {key_name}");
                KeyEvent::Release(key_name)
            };
            if tx.send(event).is_err() {
                break; // receiver dropped
            }
        }
    }
}
