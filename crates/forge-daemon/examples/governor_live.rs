//! Live governor hookup: env creds -> alpaca.exe -> real account equity/
//! maintenance_margin -> governor's AlpacaDaemonHealth atomics. Read-only
//! endpoint only (account_get); no orders placed. Run from repo root with
//! APCA_* env set:
//!   cargo run -p forge-daemon --example governor_live

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

use forge_daemon::alpaca_cli::AlpacaCli;
use forge_daemon::config;
use forge_daemon::governor::{spawn_governor, AlpacaDaemonHealth};

fn main() {
    let creds = match config::load_from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("REFUSED: {e}");
            std::process::exit(1);
        }
    };
    let cli = AlpacaCli::at_repo_root(std::path::Path::new("."));

    let health = Arc::new(AlpacaDaemonHealth::default());
    health.alive.store(true, Ordering::Relaxed);
    spawn_governor(health.clone());

    eprintln!("[governor_live] polling real account every 2s for 5 ticks (read-only)");

    for tick in 0..5 {
        match cli.account_get(&creds) {
            Ok(json) => {
                let v: serde_json::Value = serde_json::from_str(&json).expect("account JSON parses");
                let equity: f64 = v["equity"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let maint: f64 = v["maintenance_margin"].as_str().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                health.equity_bp.store((equity * 10000.0) as u64, Ordering::Relaxed);
                health.maintenance_bp.store((maint * 10000.0) as u64, Ordering::Relaxed);
                health.rss_mb.store(64, Ordering::Relaxed);
                eprintln!("[governor_live] tick {tick}: equity=${equity:.2} maintenance_margin=${maint:.2}");
            }
            Err(e) => eprintln!("[governor_live] tick {tick}: account_get refused: {e:?}"),
        }
        std::thread::sleep(Duration::from_secs(2));
    }

    eprintln!("[governor_live] done — see [governor] lines above for StrainScore/Trinary output");
}
