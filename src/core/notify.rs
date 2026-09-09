//! Desktop notifications through the OS notification center: notify-rust drives XDG/D-Bus on
//! Linux, WinRT toasts on Windows and the macOS notification center. Fire-and-forget: a
//! failure is logged and never bubbles up, and the call runs on its own thread so callers on
//! the tray's main thread or an HTTP handler are never blocked.
//!
//! macOS caveat: only app bundles own a notification identity. As a bare binary, Switcheroo's
//! notices are delivered through the Terminal's identity (its icon and name appear); a proper
//! `Switcheroo.app` bundle would fix that.

use std::thread;

pub const APP_NAME: &str = "Switcheroo";

pub fn notify(title: impl Into<String>, body: impl Into<String>) {
    let (title, body) = (title.into(), body.into());
    thread::spawn(move || {
        let result = notify_rust::Notification::new().appname(APP_NAME).summary(&title).body(&body).show();
        if let Err(e) = result {
            log::warn!("desktop notification failed: {e}");
        }
    });
}

/// "Claude Code · Switched to Work (alex@acme.dev)".
pub fn switched(provider_name: &str, label: &str, id: &str) {
    let who = if label == id { label.to_string() } else { format!("{label} ({id})") };
    notify(provider_name, format!("Switched to {who}"));
}

pub fn update_available(latest: &str) {
    notify(
        format!("{APP_NAME} v{latest} is available"),
        "Update from the tray menu, the web UI, or `switcheroo update`.",
    );
}

pub fn updated(version: &str) {
    notify(APP_NAME, format!("Updated to v{version}, restarting."));
}

#[cfg(test)]
mod tests {
    /// Manual check: `cargo test desktop_notification_smoke -- --ignored` shows one notice.
    #[test]
    #[ignore]
    fn desktop_notification_smoke() {
        super::switched("Claude Code", "Work", "alex@acme.dev");
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }
}
