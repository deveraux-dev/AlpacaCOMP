//! Regime-router dry run: chain snapshot -> 5D market point (4 fields
//! caller-supplied, depth_pmy fed from the real chain's market_purity, never
//! invented) -> RegimeRouter::route -> RegimeClass -> mapped strategy ->
//! dispatch_spread (all seven gates). DRY ONLY — no --send flag. This path
//! has zero live-chain validation yet (regime_router had zero call sites in
//! forge-daemon before this example; see 2026-09-03 ledger), unlike
//! dispatch_once.rs's condor path.
//! Usage: regime_dispatch_dry_run <chain.json> <spot> --bull <13 trits>
//!   --bear <13 trits> --moneyness <pmy> --delta <pmy> --iv-skew <pmy> --dte <days>

use forge_daemon::alpaca_cli::AlpacaCli;
use forge_daemon::config;
use forge_daemon::dispatch::{
    dispatch_spread, DispatchRefusal, CHAIN_PURITY_CEILING_PMY, CHAIN_PURITY_FLOOR_PMY, STATE_FLAT,
};
use forge_daemon::governor::{spawn_governor, AlpacaDaemonHealth};
use forge_gate::market_collapse::{collapse_market_to_query, MarketPoint5D};
use forge_gate::market_purity::NormalizedIpr;
use forge_gate::oracle_arbiter::{AuditChain, Disposition, OracleArbiter};
use forge_gate::regime_router::{default_lut, RegimeClass};
use forge_gate::strategy::{build_bear_call_spread, build_bull_put_spread, build_iron_butterfly, build_iron_condor, ChainQuote, Leg, Side};
use serde_json::Value;
use std::io::Write;
use std::sync::Arc;

const QTY: u32 = 1;
const ROOT: &str = "SPY";
const YYMMDD: &str = "261016";
const ACCOUNT_BALANCE: f64 = 100_000.0;
const MAX_WING_WIDTH: f64 = 0.02 * ACCOUNT_BALANCE / 100.0;
const CREDIT_FLOOR: f64 = 2.50;
const LIMIT_SHAVE: f64 = 0.05;

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

/// Outcome of routing `RegimeRouter::route`'s pick to a real leg-builder.
enum LegBuild {
    Legs(&'static str, Vec<Leg>),
    NoTrade,
    ChainCannotBuild(&'static str),
}

/// The actual wiring: the matched `RegimeClass`'s `routing_note` decides
/// which builder runs. `BullishContango`/`MeltUpBull` share a target
/// (`bull_put_spread`) since both are bullish regimes with no distinct
/// leg-construction logic yet — same collapse the ledger already named.
fn build_legs_for_regime(class: RegimeClass, quotes: &[ChainQuote], max_wing_width: f64) -> LegBuild {
    match class {
        RegimeClass::BearishPanic | RegimeClass::LiquidityVacuum => LegBuild::NoTrade,
        RegimeClass::SidewaysThetaBurn => match build_iron_condor(quotes, 0.16, 0.05, 0.05, max_wing_width) {
            Some(legs) => LegBuild::Legs("iron_condor", legs.to_vec()),
            None => LegBuild::ChainCannotBuild("iron_condor"),
        },
        RegimeClass::EarningsIvCrush => match build_iron_butterfly(quotes, 0.05, 0.05, max_wing_width) {
            Some(legs) => LegBuild::Legs("iron_butterfly", legs.to_vec()),
            None => LegBuild::ChainCannotBuild("iron_butterfly"),
        },
        RegimeClass::BullishContango | RegimeClass::MeltUpBull => {
            match build_bull_put_spread(quotes, 0.16, 0.05, 0.05, max_wing_width) {
                Some(legs) => LegBuild::Legs("bull_put_spread", legs.to_vec()),
                None => LegBuild::ChainCannotBuild("bull_put_spread"),
            }
        }
        RegimeClass::SlowBleedBear => match build_bear_call_spread(quotes, 0.16, 0.05, 0.05, max_wing_width) {
            Some(legs) => LegBuild::Legs("bear_call_spread", legs.to_vec()),
            None => LegBuild::ChainCannotBuild("bear_call_spread"),
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chain_path = args.get(1).expect("usage: regime_dispatch_dry_run <chain.json> <spot> ...");
    let spot: f64 = args.get(2).and_then(|s| s.parse().ok()).expect("spot price arg");

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

    // The oracle risk gate is orthogonal to strategy selection: regime_router
    // decides WHICH structure to build, the S13 verdict still gates WHETHER
    // execution is authorized at all (dispatch_spread's VerdictVeto).
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

    // 4 of the 5 MarketPoint5D dimensions have no unambiguous chain-derived
    // reference point (moneyness/delta/iv-skew need a strike already picked,
    // dte needs an expiration already chosen) — caller-supplied, same
    // discipline as --bull/--bear. depth_pmy is the exception: it has a
    // named, unambiguous source (market_collapse.rs's own doc comment: "feed
    // from market_purity::NormalizedIpr::pmy — not recomputed here"), so it
    // comes from the real `ipr` computed above, never a flag.
    let (Some(moneyness_pmy), Some(delta_pmy), Some(iv_skew_pmy), Some(dte_days)) = (
        flag_value(&args, "--moneyness").and_then(|s| s.parse::<i32>().ok()),
        flag_value(&args, "--delta").and_then(|s| s.parse::<i32>().ok()),
        flag_value(&args, "--iv-skew").and_then(|s| s.parse::<i32>().ok()),
        flag_value(&args, "--dte").and_then(|s| s.parse::<i32>().ok()),
    ) else {
        println!("REFUSED pre-gate: --moneyness/--delta/--iv-skew/--dte (permyriad ints, dte in days) required");
        std::process::exit(2);
    };
    let point = MarketPoint5D { moneyness_pmy, delta_pmy, depth_pmy: ipr.pmy, iv_skew_pmy, dte_days };
    let query = collapse_market_to_query(point);
    let router = default_lut();
    let Some((regime_id, margin)) = router.route(&query) else {
        println!("REFUSED pre-gate: no active regime centroid matched this market point");
        std::process::exit(2);
    };
    let Some(class) = RegimeClass::from_index(regime_id) else {
        println!("REFUSED pre-gate: route() returned an out-of-range regime_id {regime_id} — refusing rather than guessing");
        std::process::exit(2);
    };
    println!(
        "regime_router: point={point:?} -> {} (id={regime_id}, margin={margin}) -> {}",
        class.label(),
        class.routing_note()
    );

    let (label, legs) = match build_legs_for_regime(class, &quotes, MAX_WING_WIDTH) {
        LegBuild::Legs(label, legs) => (label, legs),
        LegBuild::NoTrade => {
            println!("REFUSED pre-gate: regime {} is a refuse-to-trade advisory ({})", class.label(), class.routing_note());
            std::process::exit(2);
        }
        LegBuild::ChainCannotBuild(label) => {
            println!("REFUSED pre-gate: regime {} picked {label}, but no capped structure buildable from this chain", class.label());
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

    let limit_price = -(((credit - LIMIT_SHAVE) * 100.0).floor() / 100.0);
    println!("limit_price: {limit_price:.2} (credit mid {:.2} shaved {LIMIT_SHAVE:.2})", credit);

    let creds = config::load_from_env().expect("APCA_API_KEY_ID/APCA_API_SECRET_KEY in env");
    let cli = AlpacaCli::new(r"Z:\nope\alpaca.exe"); // DRY ONLY: no --send on this path.
    println!("mode: DRY (dead CLI proves gates; regime_router path has no live-chain validation yet)");

    let health = Arc::new(AlpacaDaemonHealth::default());
    spawn_governor(health.clone());
    let result = dispatch_spread(
        &cli, &creds, &health, STATE_FLAT, verdict, &ipr, &legs, credit, ACCOUNT_BALANCE, ROOT, YYMMDD, QTY, limit_price,
    );
    println!("governor risk_gate_faults this run: {}", health.risk_gate_faults.load(std::sync::atomic::Ordering::Relaxed));

    let ts_secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let verdict_str = format!("{verdict:?}");
    let bull_token = args_token(&bull);
    let bear_token = args_token(&bear);
    let outcome_str = match &result {
        Ok(_) => "ACCEPTED".to_string(),
        Err(DispatchRefusal::GovernorVent) => "GovernorVent".to_string(),
        Err(DispatchRefusal::IllegalTransition) => "IllegalTransition".to_string(),
        Err(DispatchRefusal::VerdictVeto) => "VerdictVeto".to_string(),
        Err(DispatchRefusal::ChaoticBook) => "ChaoticBook".to_string(),
        Err(DispatchRefusal::MaxLossVeto) => "MaxLossVeto".to_string(),
        Err(DispatchRefusal::MalformedLegs) => "MalformedLegs".to_string(),
        Err(DispatchRefusal::Cli(_)) => "CliRefusal".to_string(),
    };

    match &result {
        Ok(resp) => println!("ORDER ACCEPTED:\n{resp}"),
        Err(DispatchRefusal::Cli(forge_daemon::alpaca_cli::CliRefusal::ExeNotFound(_))) => {
            println!("ALL SEVEN GATES PASSED (dry run stopped at CLI spawn by design)");
        }
        Err(r) => {
            println!("REFUSED: {r:?}");
            std::process::exit(1);
        }
    }

    let day = ts_secs / 86_400;
    let claim = format!("{ROOT} {YYMMDD} {} {label} dispatch (regime_router)", class.label());
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

    #[test]
    fn sideways_theta_burn_builds_a_condor() {
        match build_legs_for_regime(RegimeClass::SidewaysThetaBurn, &synthetic_chain(), 100.0) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "iron_condor");
                assert_eq!(legs.len(), 4);
            }
            _ => panic!("expected a buildable condor"),
        }
    }

    #[test]
    fn earnings_iv_crush_builds_a_butterfly() {
        match build_legs_for_regime(RegimeClass::EarningsIvCrush, &synthetic_chain(), 100.0) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "iron_butterfly");
                assert_eq!(legs.len(), 4);
            }
            _ => panic!("expected a buildable butterfly"),
        }
    }

    #[test]
    fn bullish_contango_and_melt_up_bull_both_build_a_bull_put_spread() {
        for class in [RegimeClass::BullishContango, RegimeClass::MeltUpBull] {
            match build_legs_for_regime(class, &synthetic_chain(), 100.0) {
                LegBuild::Legs(label, legs) => {
                    assert_eq!(label, "bull_put_spread");
                    assert_eq!(legs.len(), 2);
                }
                _ => panic!("expected a buildable bull put spread for {class:?}"),
            }
        }
    }

    #[test]
    fn slow_bleed_bear_builds_a_bear_call_spread() {
        match build_legs_for_regime(RegimeClass::SlowBleedBear, &synthetic_chain(), 100.0) {
            LegBuild::Legs(label, legs) => {
                assert_eq!(label, "bear_call_spread");
                assert_eq!(legs.len(), 2);
            }
            _ => panic!("expected a buildable bear call spread"),
        }
    }

    #[test]
    fn bearish_panic_and_liquidity_vacuum_both_refuse_as_no_trade() {
        for class in [RegimeClass::BearishPanic, RegimeClass::LiquidityVacuum] {
            assert!(matches!(build_legs_for_regime(class, &synthetic_chain(), 100.0), LegBuild::NoTrade));
        }
    }

    #[test]
    fn route_end_to_end_matches_default_lut_archetypes_to_themselves() {
        // Dual-oracle-style check against regime_router's own archetype test:
        // routing a real 5D point through collapse_market_to_query must land
        // on an active regime, and RegimeClass::from_index must round-trip it.
        let router = default_lut();
        let point = MarketPoint5D { moneyness_pmy: 0, delta_pmy: 0, depth_pmy: 5000, iv_skew_pmy: 0, dte_days: 45 };
        let query = collapse_market_to_query(point);
        let (regime_id, _margin) = router.route(&query).expect("default_lut has all 7 active");
        assert!(RegimeClass::from_index(regime_id).is_some());
    }
}
