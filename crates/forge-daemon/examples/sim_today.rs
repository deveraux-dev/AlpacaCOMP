//! Dry-run sim over a saved real option-chain snapshot: purity gate ->
//! 5D collapse -> trit overlay matrix -> strategy leg selection. No orders.
//! Usage: cargo run -p forge-daemon --example sim_today [chain.json] [spot]

use forge_gate::market_collapse::{collapse_market_to_query, MarketPoint5D};
use forge_gate::market_purity::NormalizedIpr;
use forge_gate::strategy::{build_iron_butterfly, build_iron_condor, ChainQuote, Side};
use serde_json::Value;

const DEFAULT_CHAIN: &str = r".forge\sim\spy_chain_20261016.json";
const DEFAULT_SPOT: f64 = 762.15;
const DTE: i32 = 45;
const BAND_SLOTS: usize = 512 / 5;

struct Contract {
    strike: f64,
    is_call: bool,
    delta: f64,
    iv: f64,
    bid: f64,
    ask: f64,
    bid_size: u16,
    ask_size: u16,
}

/// OCC symbol tail: [C|P] + strike*1000 as 8 digits.
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let chain_path = args.get(1).map(String::as_str).unwrap_or(DEFAULT_CHAIN);
    let spot: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_SPOT);

    let raw = std::fs::read_to_string(chain_path).expect("chain json readable");
    let json: Value = serde_json::from_str(&raw).expect("valid json");
    let snapshots = json["snapshots"].as_object().expect("snapshots map");

    let mut contracts: Vec<Contract> = Vec::new();
    for (sym, snap) in snapshots {
        let Some((strike, is_call)) = parse_occ(sym) else { continue };
        let delta = f(snap, &["greeks", "delta"]);
        if delta == 0.0 {
            continue; // no greeks published -> unusable, skip loudly counted below
        }
        contracts.push(Contract {
            strike,
            is_call,
            delta,
            iv: f(snap, &["impliedVolatility"]),
            bid: f(snap, &["latestQuote", "bp"]),
            ask: f(snap, &["latestQuote", "ap"]),
            bid_size: f(snap, &["latestQuote", "bs"]) as u16,
            ask_size: f(snap, &["latestQuote", "as"]) as u16,
        });
    }
    let skipped = snapshots.len() - contracts.len();
    println!("chain: {} contracts usable, {} skipped (no greeks)", contracts.len(), skipped);

    // ── Purity over quoted size across strikes (the chain's own book basis) ─
    let mut depth: Vec<u16> = contracts.iter().map(|c| c.bid_size.saturating_add(c.ask_size)).collect();
    depth.sort_unstable();
    let ipr = NormalizedIpr::compute_u16(&depth);
    println!(
        "purity: N*IPR = {} pmy over N={} (landmark>=7500, diffuse<2500) -> {}",
        ipr.pmy,
        ipr.dimension,
        if ipr.is_landmark() { "LANDMARK (pinned)" } else if ipr.is_diffuse() { "DIFFUSE (chaotic gate trips)" } else { "NORMAL" }
    );

    // ── Per-strike ChainQuote pairs ─────────────────────────────────────────
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
    println!("paired strikes: {}", quotes.len());

    // ── ATM IV for skew axis ────────────────────────────────────────────────
    let atm_iv = contracts
        .iter()
        .filter(|c| c.is_call)
        .min_by(|a, b| (a.delta - 0.5).abs().partial_cmp(&(b.delta - 0.5).abs()).unwrap())
        .map(|c| c.iv)
        .unwrap_or(0.0);

    // ── Trit overlay matrix: every 5th strike, 5 dims, tercile trit per band ─
    println!("\ntrit overlay [strike | moneyness delta depth skew dte | fingerprint terciles]");
    for (i, &k) in strikes.iter().enumerate() {
        if i % 5 != 0 {
            continue;
        }
        let Some(c) = contracts.iter().find(|c| c.strike == k && c.is_call) else { continue };
        let point = MarketPoint5D {
            moneyness_pmy: (((k - spot) / spot) * 10_000.0) as i32,
            delta_pmy: (c.delta * 10_000.0) as i32,
            depth_pmy: ipr.pmy,
            iv_skew_pmy: ((c.iv - atm_iv) * 10_000.0) as i32,
            dte_days: DTE,
        };
        let q = collapse_market_to_query(point);
        let row: String = (0..5)
            .map(|d| {
                let fill = q[d * BAND_SLOTS..(d + 1) * BAND_SLOTS].iter().filter(|&&s| s == 1).count();
                match fill * 3 / BAND_SLOTS {
                    0 => '-',
                    1 => '0',
                    _ => '+',
                }
            })
            .collect();
        println!("  {k:7.1} | {row}");
    }

    // ── Strategy dry-run (verdict stubbed NEUTRAL: LLM oracles not in sim) ──
    println!("\nverdict: StructuralEquilibrium [SIMULATED — live oracles not in this dry-run]");
    let mid = |strike: f64, is_call: bool| -> f64 {
        contracts
            .iter()
            .find(|c| c.strike == strike && c.is_call == is_call)
            .map(|c| (c.bid + c.ask) / 2.0)
            .unwrap_or(0.0)
    };
    let describe = |name: &str, legs: Option<[forge_gate::strategy::Leg; 4]>| match legs {
        Some(legs) => {
            let credit: f64 = legs
                .iter()
                .map(|l| {
                    let m = mid(l.strike, l.is_call);
                    if l.side == Side::Sell { m } else { -m }
                })
                .sum();
            println!("{name}:");
            for l in &legs {
                println!(
                    "  {:4} {:4} @ {:7.1}  (mid {:.2})",
                    if l.side == Side::Sell { "SELL" } else { "BUY" },
                    if l.is_call { "CALL" } else { "PUT" },
                    l.strike,
                    mid(l.strike, l.is_call)
                );
            }
            println!("  net credit/contract: ${:.2}  take-profit exit at ${:.2}", credit * 100.0, credit * 50.0);
        }
        None => println!("{name}: REFUSED — no legs within delta deviation bound (correct refusal, not an error)"),
    };

    describe("iron condor (16d short / 5d wing)", build_iron_condor(&quotes, 0.16, 0.05, 0.05));
    if ipr.is_landmark() {
        describe("iron butterfly (landmark-triggered)", build_iron_butterfly(&quotes, 0.05, 0.05));
    } else {
        println!("iron butterfly: not triggered (book not landmark-pinned)");
    }
}
