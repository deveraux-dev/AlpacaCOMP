//! Live smoke: env creds -> bridge -> alpaca.exe -> paper account + clock.
//! Run from repo root with APCA_* env set. Read-only endpoints only.

use forge_daemon::alpaca_cli::AlpacaCli;
use forge_daemon::config;

fn main() {
    let creds = match config::load_from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("REFUSED: {e}");
            std::process::exit(1);
        }
    };
    let cli = AlpacaCli::at_repo_root(std::path::Path::new("."));

    match cli.account_get(&creds) {
        Ok(json) => println!("ACCOUNT OK:\n{json}"),
        Err(e) => {
            eprintln!("ACCOUNT REFUSED: {e:?}");
            std::process::exit(1);
        }
    }
    match cli.clock(&creds) {
        Ok(json) => println!("CLOCK OK:\n{json}"),
        Err(e) => {
            eprintln!("CLOCK REFUSED: {e:?}");
            std::process::exit(1);
        }
    }
}
