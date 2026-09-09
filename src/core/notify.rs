//! Desktop notifications through the OS notification center. Fire-and-forget: a failure is
//! logged and never bubbles up, and the call runs on its own thread so callers on the tray's
//! main thread or an HTTP handler are never blocked.
//!
//! Windows (WinRT toasts) and Linux (XDG over D-Bus) go through notify-rust. macOS does not:
//! only app bundles own a notification identity, and on macOS 26 the legacy API a bare binary
//! could use no longer displays anything. `osascript`'s `display notification` is delivered
//! (under Script Editor's identity), so that is the macOS path until Switcheroo ships as a
//! `.app` bundle.

use std::thread;

pub const APP_NAME: &str = "Switcheroo";

pub fn notify(title: impl Into<String>, body: impl Into<String>) {
    let (title, body) = (title.into(), body.into());
    thread::spawn(move || {
        if let Err(e) = deliver(&title, &body) {
            log::warn!("desktop notification failed: {e}");
        }
    });
}

#[cfg(target_os = "macos")]
fn deliver(title: &str, body: &str) -> anyhow::Result<()> {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("display notification \"{}\" with title \"{}\"", esc(body), esc(title));
    let out = std::process::Command::new("osascript").args(["-e", &script]).output()?;
    if !out.status.success() {
        anyhow::bail!("osascript: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn deliver(title: &str, body: &str) -> anyhow::Result<()> {
    notify_rust::Notification::new().appname(APP_NAME).summary(title).body(body).show()?;
    Ok(())
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
