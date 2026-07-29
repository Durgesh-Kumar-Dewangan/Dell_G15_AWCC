//! Logger module.
//!
//! A single place that configures `tracing`'s output format/filtering, so the
//! daemon, CLI, and GUI all log consistently instead of each hand-rolling a
//! `tracing_subscriber::fmt()` call with slightly different defaults.
//!
//! Log level is controlled by the standard `RUST_LOG` environment variable
//! (e.g. `RUST_LOG=debug g15-fancontrold`); it defaults to `info` when unset.
//! Under systemd (`Type=notify`), output goes to stderr, which systemd
//! captures into the journal automatically — see `journalctl -u g15-fancontrold`.

/// Initialize the global tracing subscriber. Call this exactly once, as early
/// as possible in `main()`, before any other module logs anything.
///
/// Safe to call from a binary that never sets `RUST_LOG`: the environment
/// filter falls back to `info` and above, matching what an unattended systemd
/// service should log by default (state transitions, warnings, errors — not
/// per-tick sensor chatter).
pub fn init(default_level: &str) {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}
