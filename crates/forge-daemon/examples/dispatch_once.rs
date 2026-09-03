//! One-shot gated dispatch: chain snapshot -> purity -> capped condor ->
//! credit floor -> dispatch_spread (all six gates). Dry by default (dead CLI
//! proves gates); --send uses the real CLI.
//! Usage: dispatch_once <chain.json> <spot> --bull <13 trits> --bear <13 trits> [--send]

use forge_daemon::alpaca_cli::{AlpacaCli, CliRefusal};
use forge_daemon::config;
use forge_daemon::dispatch::{
    dispatch_spread, DispatchRefusal, CHAIN_PURITY_CEILING_PMY, CHAIN_PURITY_FLOOR_PMY, STATE_FLAT,
};
use forge_daemon::governor::{spawn_governor, AlpacaDaemonHealth};
use forge_gate::market_purity::NormalizedIpr;
use forge_gate::oracle_arbiter::{AuditChain, Disposition, OracleArbiter};
use forge_gate::strategy::{build_iron_condor, ChainQuote, Side};
use serde_json::Value;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

const QTY: u32 = 1;
const ROOT: &str = "SPY";
const YYMMDD: &str = "261016";
const ACCOUNT_BALANCE: f64 = 100_000.0;
const MAX_WING_WIDTH: f64 = 0.02 * ACCOUNT_BALANCE / 100.0;
const CREDIT_FLOOR: f64 = 2.50;
const LIMIT_SHAVE: f64 = 0.05;

/// Parse an S13 thesis token: exactly 13 chars of `+`/`0`/`-`.
/// The ONLY channel an LLM oracle has into this binary.
fn parse_s13(s: &str) -> Option<[i8; 13]> {
    let bytes = s.as_bytes();
    if bytes.len() != 13 {
        return None;
    }
    let mut lanes = [0i8; 13];
    for (i, b) in bytes.iter().enumerate() {
        lanes[i] = match b {
            b'+' => 1,
            b'0' => 0,
            b'-' => -1,
            _ => return None,
        };
    }
    Some(lanes)
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn args_token(lanes: &[i8; 13]) -> String {
    lanes.iter().map(|&t| match t { 1 => '+', -1 => '-', _ => '0' }).collect()
}

fn parse_occ(symbol: &str) -> Option<(f64, bool)> {
    let tail = &symbol[symbol.len().checked_sub(9)?..];
    let is_call = match tail.as_bytes()[0] {
        b'C' => true,
        b'P' => false,
        _ => return None,
    };
    let millis: f64 = tail[1..].parse().ok()?;
    Some((millis / 1000.0, is_call))
}

fn f(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for p in path {
        cur = &cur[p];
    }
    cur.as_f64().unwrap_or(0.0)
}

struct Contract {
    strike: f64,
    is_call: bool,
    delta: f64,
    bid: f64,
    ask: f64,
    volume: u64,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chain_path = args.get(1).expect("usage: dispatch_once <chain.json> <spot> [--send]");
    let spot: f64 = args.get(2).and_then(|s| s.parse().ok()).expect("spot price arg");
    let send = args.iter().any(|a| a == "--send");

    let raw = std::fs::read_to_string(chain_path).expect("chain json readable");
    let json: Value = serde_json::from_str(&raw).expect("valid json");
    let snapshots = json["snapshots"].as_object().expect("snapshots map");

    let mut contracts: Vec<Contract> = Vec::new();
    for (sym, snap) in snapshots {
        let Some((strike, is_call)) = parse_occ(sym) else { continue };
        let delta = f(snap, &["greeks", "delta"]);
        if delta == 0.0 {
            continue;
        }
        contracts.push(Contract {
            strike,
            is_call,
            delta,
            bid: f(snap, &["latestQuote", "bp"]),
            ask: f(snap, &["latestQuote", "ap"]),
            volume: f(snap, &["dailyBar", "v"]) as u64,
        });
    }
    println!("chain: {} contracts usable, spot {spot}", contracts.len());

    // Purity mass = per-contract daily volume (preauth 2026-09-03); ratios
    // preserved under a common divisor so the max fits compute_u16's u16 lane.
    let max_vol = contracts.iter().map(|c| c.volume).max().unwrap_or(0);
    let shift = (max_vol / u16::MAX as u64) + 1;
    let depth: Vec<u16> = contracts.iter().map(|c| (c.volume / shift) as u16).collect();
    let ipr = NormalizedIpr::compute_u16(&depth);
    let band = if ipr.pmy < CHAIN_PURITY_FLOOR_PMY {
        "BELOW-FLOOR volume-dead (gate will refuse)"
    } else if ipr.pmy > CHAIN_PURITY_CEILING_PMY {
        "ABOVE-CEILING panic-concentrated (gate will refuse)"
    } else {
        "IN-BAND"
    };
    println!(
        "purity: N*IPR = {} pmy over N={} (volume mass, band {}-{}) -> {band}",
        ipr.pmy, ipr.dimension, CHAIN_PURITY_FLOOR_PMY, CHAIN_PURITY_CEILING_PMY
    );

    let mut strikes: Vec<f64> = contracts.iter().map(|c| c.strike).collect();
    strikes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    strikes.dedup();
    let mut quotes: Vec<ChainQuote> = Vec::new();
    for &k in &strikes {
        let call = contracts.iter().find(|c| c.strike == k && c.is_call);
        let put = contracts.iter().find(|c| c.strike == k && !c.is_call);
        if let (Some(c), Some(p)) = (call, put) {
            quotes.push(ChainQuote { strike: k, call_delta: c.delta, put_delta: p.delta });
        }
    }

    let Some(legs) = build_iron_condor(&quotes, 0.16, 0.05, 0.05, MAX_WING_WIDTH) else {
        println!("REFUSED pre-gate: no capped condor buildable from this chain");
        std::process::exit(2);
    };

    let mid = |strike: f64, is_call: bool| -> f64 {
        contracts
            .iter()
            .find(|c| c.strike == strike && c.is_call == is_call)
            .map(|c| (c.bid + c.ask) / 2.0)
            .unwrap_or(0.0)
    };
    let credit: f64 = legs
        .iter()
        .map(|l| {
            let m = mid(l.strike, l.is_call);
            if l.side == Side::Sell { m } else { -m }
        })
        .sum();

    for l in &legs {
        println!(
            "  {:4} {:4} @ {:7.1}  (mid {:.2})",
            if l.side == Side::Sell { "SELL" } else { "BUY" },
            if l.is_call { "CALL" } else { "PUT" },
            l.strike,
            mid(l.strike, l.is_call)
        );
    }
    println!("net credit/contract: ${:.2}", credit * 100.0);

    if credit < CREDIT_FLOOR {
        println!("REFUSED pre-gate: credit {credit:.2} below floor {CREDIT_FLOOR:.2} (pre-authorized bound)");
        std::process::exit(2);
    }

    // Credit = NEGATIVE limit_price (Alpaca mleg convention, 3x verified).
    let limit_price = -(((credit - LIMIT_SHAVE) * 100.0).floor() / 100.0);
    println!("limit_price: {limit_price:.2} (credit mid {:.2} shaved {LIMIT_SHAVE:.2})", credit);

    // The live oracle seam: two S13 theses in, one arbitrated verdict out.
    // An LLM's ONLY influence on this order is 26 trits through this gate.
    let (Some(bull), Some(bear)) = (
        flag_value(&args, "--bull").and_then(parse_s13),
        flag_value(&args, "--bear").and_then(parse_s13),
    ) else {
        println!("REFUSED pre-gate: --bull/--bear S13 theses required (13 chars of +/0/-)");
        std::process::exit(2);
    };
    let mut audit = AuditChain::new();
    audit.append(
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        Disposition::Attested([0u8; 32]),
    );
    let verdict = OracleArbiter::arbitrate(&audit, &bull, &bear);
    println!("oracle seam: bull={} bear={} -> {verdict:?}", args_token(&bull), args_token(&bear));

    let creds = config::load_from_env().expect("APCA_API_KEY_ID/APCA_API_SECRET_KEY in env");
    let cli = if send {
        AlpacaCli::at_repo_root(Path::new("."))
    } else {
        AlpacaCli::new(r"Z:\nope\alpaca.exe")
    };
    println!("mode: {}", if send { "SEND (live paper order)" } else { "DRY (dead CLI proves gates)" });

    // Governor spawned for real here, not just a throwaway counter: this is
    // the actual live-order binary, so StrainScore/TrinaryState now compute
    // for every dispatch, not only in the governor_live/bifurcation_alpaca_loop
    // demo examples.
    let health = Arc::new(AlpacaDaemonHealth::default());
    spawn_governor(health.clone());
    let result = dispatch_spread(
        &cli,
        &creds,
        &health,
        STATE_FLAT,
        verdict,
        &ipr,
        &legs,
        credit,
        ACCOUNT_BALANCE,
        ROOT,
        YYMMDD,
        QTY,
        limit_price,
    );
    println!("governor risk_gate_faults this run: {}", health.risk_gate_faults.load(std::sync::atomic::Ordering::Relaxed));

    // Capture timestamp & verdict before match for ledger row.
    let ts_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let verdict_str = format!("{:?}", verdict);
    let bull_token = args_token(&bull);
    let bear_token = args_token(&bear);

    // Determine outcome for ledger before consuming result.
    let outcome_str = match &result {
        Ok(_) => "ACCEPTED".to_string(),
        Err(r) => match r {
            DispatchRefusal::IllegalTransition => "IllegalTransition".to_string(),
            DispatchRefusal::VerdictVeto => "VerdictVeto".to_string(),
            DispatchRefusal::ChaoticBook => "ChaoticBook".to_string(),
            DispatchRefusal::MaxLossVeto => "MaxLossVeto".to_string(),
            DispatchRefusal::MalformedLegs => "MalformedLegs".to_string(),
            DispatchRefusal::Cli(_) => "CliRefusal".to_string(),
        },
    };

    match &result {
        Ok(resp) => println!("ORDER ACCEPTED:\n{resp}"),
        Err(DispatchRefusal::Cli(CliRefusal::ExeNotFound(_))) if !send => {
            println!("ALL SIX GATES PASSED (dry run stopped at CLI spawn by design)");
        }
        Err(r) => {
            println!("REFUSED: {r:?}");
            std::process::exit(1);
        }
    }

    // Append to proof-ledger TSV: day, secs, crate, claim, ladder, oracle_1, oracle_2, receipt
    // day = days-since-epoch (secs/86400), matching every existing row's convention.
    let day = ts_secs / 86_400;
    let claim = format!("{} {} iron_condor dispatch", ROOT, YYMMDD);
    let row = format!("{day}\t{ts_secs}\tforge-daemon\t{claim}\t{outcome_str}\t{bull_token}\t{bear_token}\t{verdict_str}\n");

    if let Err(e) = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(".forge/proof-ledger.tsv")
        .and_then(|mut f| f.write_all(row.as_bytes()))
    {
        eprintln!("WARNING: ledger write failed: {e}");
    } else {
        eprintln!("ledger row written: {row:?}");
    }
}
