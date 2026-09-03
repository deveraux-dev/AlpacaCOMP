//! Live Alpaca trading-day loop with bifurcation circuit breaker: real
//! account equity/maintenance_margin -> governor -> trinary state, on a
//! poll interval, market-hours-gated via `clock`. Read-only; no orders.
//!
//!   cargo run -p forge-daemon --example bifurcation_alpaca_loop
//!
//! Env: APCA_API_KEY_ID / APCA_API_SECRET_KEY required.
//! Optional: GOVERNOR_LOOP_TICKS (default: run until killed),
//!           GOVERNOR_POLL_SECS (default: 30).

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use forge_daemon::alpaca_cli::AlpacaCli;
use forge_daemon::config;
use forge_daemon::governor::{spawn_governor, AlpacaDaemonHealth};

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn main() {
    let creds = match config::load_from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("REFUSED: {e}");
            std::process::exit(1);
        }
    };
    let cli = AlpacaCli::at_repo_root(std::path::Path::new("."));

    // alive/rss_mb stay at default (false/0): alpaca.exe is a per-call
    // subprocess, not a persistent one this loop supervises — setting
    // alive=true without a real RSS feed would manufacture a sensor fault
    // every tick (Signal Law would then be screaming about a lie, not a
    // real fault).
    let health = Arc::new(AlpacaDaemonHealth::default());
    spawn_governor(health.clone());

    let poll_secs = env_u64("GOVERNOR_POLL_SECS", 30);
    let max_ticks = std::env::var("GOVERNOR_LOOP_TICKS").ok().and_then(|v| v.parse::<u64>().ok());

    eprintln!(
        "[bifurcation_alpaca_loop] live poll every {poll_secs}s{}, market-hours gated",
        max_ticks.map(|t| format!(", {t} ticks then exit")).unwrap_or_default()
    );

    let mut tick: u64 = 0;
    loop {
        if let Some(max) = max_ticks {
            if tick >= max {
                eprintln!("[bifurcation_alpaca_loop] reached GOVERNOR_LOOP_TICKS={max}, exiting");
                break;
            }
        }

        let is_open = match cli.clock(&creds) {
            Ok(json) => serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .and_then(|v| v["is_open"].as_bool())
                .unwrap_or(false),
            Err(e) => {
                eprintln!("[bifurcation_alpaca_loop] clock refused: {e:?} — treating as closed");
                false
            }
        };

        if !is_open {
            eprintln!("[bifurcation_alpaca_loop] market closed — skipping account poll this tick");
        } else {
            match cli.account_get(&creds) {
                Ok(json) => {
                    let v: serde_json::Value = match serde_json::from_str(&json) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("[bifurcation_alpaca_loop] account JSON parse failed: {e}");
                            std::thread::sleep(Duration::from_secs(poll_secs));
                            tick += 1;
                            continue;
                        }
                    };
                    let equity: f64 = v["equity"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let maint: f64 = v["maintenance_margin"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    health.equity_bp.store((equity * 10000.0) as u64, Ordering::Relaxed);
                    health.maintenance_bp.store((maint * 10000.0) as u64, Ordering::Relaxed);
                    eprintln!(
                        "[bifurcation_alpaca_loop] tick {tick}: equity=${equity:.2} maintenance_margin=${maint:.2}"
                    );
                }
                Err(e) => eprintln!("[bifurcation_alpaca_loop] account_get refused: {e:?}"),
            }
        }

        tick += 1;
        std::thread::sleep(Duration::from_secs(poll_secs));
    }
}
