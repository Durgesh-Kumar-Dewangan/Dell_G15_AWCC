//! Minimal `sd_notify(3)` client, implemented directly over the `NOTIFY_SOCKET`
//! unix datagram socket so we don't need to link libsystemd. This lets systemd
//! know the daemon has finished initializing (Type=notify in the unit file).

use std::os::unix::net::UnixDatagram;

pub fn notify_ready() {
    notify("READY=1");
}

#[allow(dead_code)]
pub fn notify_stopping() {
    notify("STOPPING=1");
}

#[allow(dead_code)]
pub fn notify_status(status: &str) {
    notify(&format!("STATUS={status}"));
}

fn notify(message: &str) {
    let Ok(socket_path) = std::env::var("NOTIFY_SOCKET") else {
        // Not running under systemd (e.g. developer running `cargo run` directly) —
        // this is expected and not an error.
        return;
    };
    let Ok(socket) = UnixDatagram::unbound() else { return };
    let target = if let Some(stripped) = socket_path.strip_prefix('@') {
        // Abstract namespace socket.
        format!("\0{stripped}")
    } else {
        socket_path
    };
    let _ = socket.send_to(message.as_bytes(), target);
}
