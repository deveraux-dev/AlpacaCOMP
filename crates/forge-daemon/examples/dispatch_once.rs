//! One-shot gated dispatch: chain snapshot -> oracle verdict -> select_strategy
//! -> purity -> capped structure -> credit floor -> dispatch_spread (all
//! seven gates). Dry by default (dead CLI proves gates); --send uses the
//! real CLI.
//! Usage: dispatch_once <chain.json> <spot> --bull <13 trits> --bear <13 trits> --ivr <0-100> [--send]

use forge_daemon::alpaca_cli::{AlpacaCli, CliRefusal};
use forge_daemon::config;
use forge_daemon::dispatch::{
    dispatch_spread, DispatchRefusal, CHAIN_PURITY_CEILING_PMY, CHAIN_PURITY_FLOOR_PMY, STATE_FLAT,
};
use forge_daemon::governor::{spawn_governor, AlpacaDaemonHealth};
use forge_gate::market_purity::NormalizedIpr;
use forge_gate::oracle_arbiter::{composite_gravity, AuditChain, Disposition, OracleArbiter};
use forge_gate::strategy::{
    build_bear_call_spread, build_bull_put_spread, build_iron_butterfly, build_iron_condor, select_strategy, ChainQuote, Leg, Side,
    StrategyKind,
};
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

/// Outcome of routing `select_strategy`'s pick to a real leg-builder.
enum LegBuild {
    Legs(&'static str, Vec<Leg>),
    NoTrade,
    /// `select_strategy` picked `DirectionalVertical` but `bias == 0` —
    /// structurally unreachable given `select_strategy`'s own verdict bands
    /// (`DirectionalVertical` only fires for `ScheduledMaintenance`, which
    /// requires `0 < |composite_gravity| <= 3`), kept as a safe refuse
    /// rather than an `unreachable!()` panic.
    DirectionalNotActionable,
    ChainCannotBuild(&'static str),
}

/// The actual wiring: `select_strategy`'s `StrategyKind` decides which
/// builder runs, not a hardcoded choice. `bias` is
/// `oracle_arbiter::composite_gravity(bull, bear)` — the same signed value
/// `arbitrate` already collapsed into the verdict, reused (not reinvented)
/// to pick bull vs bear when the pick is directional.
fn build_legs_for(kind: StrategyKind, quotes: &[ChainQuote], max_wing_width: f64, bias: i32) -> LegBuild {
    match kind {
        StrategyKind::NoTrade => LegBuild::NoTrade,
        StrategyKind::DirectionalVertical => {
            if bias > 0 {
                match build_bull_put_spread(quotes, 0.16, 0.05, 0.05, max_wing_width) {
                    Some(legs) => LegBuild::Legs("bull_put_spread", legs.to_vec()),
                    None => LegBuild::ChainCannotBuild("bull_put_spread"),
                }
            } else if bias < 0 {
                match build_bear_call_spread(quotes, 0.16, 0.05, 0.05, max_wing_width) {
                    Some(legs) => LegBuild::Legs("bear_call_spread", legs.to_vec()),
                    None => LegBuild::ChainCannotBuild("bear_call_spread"),
                }
            } else {
                LegBuild::DirectionalNotActionable
            }
        }
        StrategyKind::IronCondor => match build_iron_condor(quotes, 0.16, 0.05, 0.05, max_wing_width) {
            Some(legs) => LegBuild::Legs("iron_condor", legs.to_vec()),
            None => LegBuild::ChainCannotBuild("iron_condor"),
        },
        StrategyKind::IronButterfly => match build_iron_butterfly(quotes, 0.05, 0.05, max_wing_width) {
            Some(legs) => LegBuild::Legs("iron_butterfly", legs.to_vec()),
            None => LegBuild::ChainCannotBuild("iron_butterfly"),
        },
    }
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

    // The live oracle seam: two S13 theses in, one arbitrated verdict out.
    // An LLM's ONLY influence on this order is 26 trits through this gate.
    // Moved ahead of leg-building: select_strategy needs the verdict to
    // decide WHICH structure to build, not just gate an already-built one.
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

    // Implied-vol rank: no historical IV corpus exists in this repo to
    // derive it (same reasoning as regime_router's hand-authored LUT
    // archetypes) — caller-supplied, never invented from the snapshot.
    let Some(ivr) = flag_value(&args, "--ivr").and_then(|s| s.parse::<f64>().ok()) else {
        println!("REFUSED pre-gate: --ivr <0-100> required (implied-vol rank; no historical IV corpus in-repo to derive it)");
        std::process::exit(2);
    };

    let kind = select_strategy(verdict, ivr, &ipr);
    println!("select_strategy: verdict={verdict:?} ivr={ivr} purity={} pmy -> {kind:?}", ipr.pmy);

    let bias = composite_gravity(&bull, &bear);
    let (label, legs) = match build_legs_for(kind, &quotes, MAX_WING_WIDTH, bias) {
        LegBuild::Legs(label, legs) => (label, legs),
        LegBuild::NoTrade => {
            println!("REFUSED pre-gate: select_strategy picked NoTrade (verdict/IVR do not authorize selling premium)");
            std::process::exit(2);
        }
        LegBuild::DirectionalNotActionable => {
            println!("REFUSED pre-gate: select_strategy picked DirectionalVertical with bias=0 (should be unreachable given the verdict bands) — refusing rather than guessing");
            std::process::exit(2);
        }
        LegBuild::ChainCannotBuild(label) => {
            println!("REFUSED pre-gate: select_strategy picked {label}, but no capped structure buildable from this chain");
            std::process::exit(2);
        }
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
            DispatchRefusal::GovernorVent => "GovernorVent".to_string(),
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
            println!("ALL SEVEN GATES PASSED (dry run stopped at CLI spawn by design)");
        }
        Err(r) => {
            println!("REFUSED: {r:?}");
            std::process::exit(1);
        }
    }

    // Append to proof-ledger TSV: day, secs, crate, claim, ladder, oracle_1, oracle_2, receipt
    // day = days-since-epoch (secs/86400), matching every existing row's convention.
    let day = ts_secs / 86_400;
    let claim = format!("{ROOT} {YYMMDD} {label} dispatch");
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

#[cfg(test)]
mod tests {
    use super::*;
    use forge_gate::market_purity::NormalizedIpr;
    use forge_gate::oracle_arbiter::OracleVerdict;

    fn synthetic_chain() -> [ChainQuote; 7] {
        [
            ChainQuote { strike: 90.0, call_delta: 0.95, put_delta: -0.05 },
            ChainQuote { strike: 95.0, call_delta: 0.80, put_delta: -0.16 },
            ChainQuote { strike: 100.0, call_delta: 0.50, put_delta: -0.50 },
            ChainQuote { strike: 105.0, call_delta: 0.16, put_delta: -0.80 },
            ChainQuote { strike: 110.0, call_delta: 0.05, put_delta: -0.95 },
            ChainQuote { strike: 85.0, call_delta: 0.99, put_delta: -0.01 },
            ChainQuote { strike: 115.0, call_delta: 0.01, put_delta: -0.99 },
        ]
    }

    fn diffuse_purity() -> NormalizedIpr {
        NormalizedIpr::compute_u16(&[10, 10, 10, 10])
    }

    fn pinned_purity() -> NormalizedIpr {
        NormalizedIpr::compute_u16(&[100, 0, 0, 0])
    }

    #[test]
    fn wired_end_to_end_equilibrium_high_iv_diffuse_book_builds_a_condor() {
        // select_strategy's own decision, not a hardcoded builder call.
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 45.0, &diffuse_purity());
        assert_eq!(kind, StrategyKind::IronCondor);
        match build_legs_for(kind, &synthetic_chain(), 100.0, 0) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "iron_condor");
                assert_eq!(legs.len(), 4);
            }
            _ => panic!("expected a buildable condor"),
        }
    }

    #[test]
    fn wired_end_to_end_equilibrium_high_iv_pinned_book_builds_a_butterfly() {
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 45.0, &pinned_purity());
        assert_eq!(kind, StrategyKind::IronButterfly);
        match build_legs_for(kind, &synthetic_chain(), 100.0, 0) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "iron_butterfly");
                assert_eq!(legs.len(), 4);
            }
            _ => panic!("expected a buildable butterfly"),
        }
    }

    #[test]
    fn wired_end_to_end_low_iv_refuses_as_no_trade() {
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 20.0, &diffuse_purity());
        assert_eq!(kind, StrategyKind::NoTrade);
        assert!(matches!(build_legs_for(kind, &synthetic_chain(), 100.0, 0), LegBuild::NoTrade));
    }

    #[test]
    fn wired_end_to_end_bullish_scheduled_maintenance_builds_a_bull_put_spread() {
        // composite_gravity > 0: bullish thesis lanes dominate.
        let mut bull = [0i8; 13];
        let bear = [0i8; 13];
        bull[0] = 1;
        bull[5] = 1;
        bull[12] = 1; // sum 3, same fixture shape as oracle_arbiter's own test
        let kind = select_strategy(OracleVerdict::ScheduledMaintenance, 45.0, &diffuse_purity());
        assert_eq!(kind, StrategyKind::DirectionalVertical);
        let bias = composite_gravity(&bull, &bear);
        assert!(bias > 0);
        match build_legs_for(kind, &synthetic_chain(), 100.0, bias) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "bull_put_spread");
                assert_eq!(legs.len(), 2);
            }
            _ => panic!("expected a buildable bull put spread"),
        }
    }

    #[test]
    fn wired_end_to_end_bearish_scheduled_maintenance_builds_a_bear_call_spread() {
        let bull = [0i8; 13];
        let mut bear = [0i8; 13];
        bear[0] = -1;
        bear[5] = -1;
        bear[12] = -1; // sum -3
        let kind = select_strategy(OracleVerdict::ScheduledMaintenance, 45.0, &diffuse_purity());
        assert_eq!(kind, StrategyKind::DirectionalVertical);
        let bias = composite_gravity(&bull, &bear);
        assert!(bias < 0);
        match build_legs_for(kind, &synthetic_chain(), 100.0, bias) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "bear_call_spread");
                assert_eq!(legs.len(), 2);
            }
            _ => panic!("expected a buildable bear call spread"),
        }
    }

    #[test]
    fn directional_pick_with_zero_bias_refuses_rather_than_guesses() {
        // Structurally unreachable through select_strategy's own bands, but
        // build_legs_for must still refuse safely if it ever happens.
        let kind = StrategyKind::DirectionalVertical;
        assert!(matches!(build_legs_for(kind, &synthetic_chain(), 100.0, 0), LegBuild::DirectionalNotActionable));
    }

    #[test]
    fn a_condor_pick_on_a_chain_too_thin_to_build_refuses_honestly_not_silently() {
        let thin_chain = [
            ChainQuote { strike: 95.0, call_delta: 0.80, put_delta: -0.16 },
            ChainQuote { strike: 100.0, call_delta: 0.50, put_delta: -0.50 },
            ChainQuote { strike: 105.0, call_delta: 0.16, put_delta: -0.80 },
        ];
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 45.0, &diffuse_purity());
        assert_eq!(kind, StrategyKind::IronCondor);
        assert!(matches!(build_legs_for(kind, &thin_chain, 100.0, 0), LegBuild::ChainCannotBuild("iron_condor")));
    }
}
