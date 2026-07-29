//! `g15fanctl`: a small CLI for diagnostics, scripting, and testing the daemon
//! without needing the GTK4 GUI. This is also what the test procedures in
//! docs/05-testing.md drive.

use anyhow::Result;
use clap::{Parser, Subcommand};
use zbus::blocking::Connection;

const BUS_NAME: &str = "org.g15fanctl.Daemon";
const OBJECT_PATH: &str = "/org/g15fanctl/Daemon";
const IFACE_NAME: &str = "org.g15fanctl.Daemon1";

#[derive(Parser)]
#[command(name = "g15fanctl", about = "Dell G-Series Fan Control Center CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run standalone hardware detection (does not require the daemon to be running).
    Detect,
    /// Print the current sensor snapshot from the running daemon.
    Status,
    /// Print daemon-reported hardware capabilities.
    Capabilities,
    /// Set the active thermal profile: quiet | balanced | performance | gmode | custom
    SetProfile { profile: String },
    /// Set a fan's mode. channel: cpu|gpu. mode: auto|max|<0-255>
    SetFan { channel: String, mode: String },
    /// List saved custom fan curves.
    ListCurves,
}

fn call_daemon(method: &str, args: &[&str]) -> Result<String> {
    let conn = Connection::system()?;
    let proxy = zbus::blocking::Proxy::new(&conn, BUS_NAME, OBJECT_PATH, IFACE_NAME)?;
    let reply: String = match args.len() {
        0 => proxy.call(method, &())?,
        1 => proxy.call(method, &(args[0],))?,
        2 => proxy.call(method, &(args[0], args[1]))?,
        _ => anyhow::bail!("too many arguments"),
    };
    Ok(reply)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Detect => {
            let hw = g15_common::detect::detect_hardware()?;
            println!("{}", hw.report());
        }
        Command::Status => {
            let json = call_daemon("get_snapshot", &[])?;
            println!("{json}");
        }
        Command::Capabilities => {
            let json = call_daemon("get_capabilities", &[])?;
            println!("{json}");
        }
        Command::SetProfile { profile } => {
            let normalized = normalize_profile(&profile)?;
            let json = call_daemon("set_profile", &[normalized])?;
            println!("{json}");
        }
        Command::SetFan { channel, mode } => {
            let mode_json = match mode.as_str() {
                "auto" => "\"Auto\"".to_string(),
                "max" => "\"Maximum\"".to_string(),
                duty => {
                    let d: u8 = duty.parse().map_err(|_| anyhow::anyhow!("mode must be auto, max, or 0-255"))?;
                    format!("{{\"Manual\":{d}}}")
                }
            };
            let json = call_daemon("set_fan_mode", &[channel.as_str(), &mode_json])?;
            println!("{json}");
        }
        Command::ListCurves => {
            let json = call_daemon("list_fan_curves", &[])?;
            println!("{json}");
        }
    }
    Ok(())
}

fn normalize_profile(input: &str) -> Result<&'static str> {
    Ok(match input.to_lowercase().as_str() {
        "quiet" => "Quiet",
        "balanced" => "Balanced",
        "performance" => "Performance",
        "gmode" | "g-mode" | "game-shift" => "GMode",
        "custom" => "Custom",
        other => anyhow::bail!("unknown profile '{other}' (expected quiet|balanced|performance|gmode|custom)"),
    })
}
