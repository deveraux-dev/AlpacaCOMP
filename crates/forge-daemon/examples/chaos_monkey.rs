//! Chaos monkey: replays the 6 known DispatchRefusal scenarios against the
//! real gate chain on a loop, confirms each still refuses as expected, and
//! writes a live JSON status report for the demo portal. Dead CLI only —
//! no live order ever reaches Alpaca. Net-new (no source primitive existed;
//! pattern only, per G:\E DRIVE\...\forge-envelope\src\bin\chaos_monkey.rs
//! false-code-match/real-pattern-match review).

use forge_daemon::dispatch::{
    dispatch_spread, DispatchRefusal, CHAIN_PURITY_FLOOR_PMY, STATE_FLAT, STATE_SPREAD_OPEN,
};
use forge_daemon::governor::AlpacaDaemonHealth;
use forge_daemon::{alpaca_cli::AlpacaCli, config::AlpacaCredentials, secrets::SecureSecret};
use forge_gate::market_purity::NormalizedIpr;
use forge_gate::oracle_arbiter::OracleVerdict;
use forge_gate::strategy::{Leg, Side};
use std::io::Write;

struct Scenario {
    name: &'static str,
    threat: &'static str,
    position_state: u32,
    verdict: OracleVerdict,
    purity: fn() -> NormalizedIpr,
    legs: fn() -> [Leg; 4],
    credit: f64,
    balance: f64,
    expect: fn(&DispatchRefusal) -> bool,
}

fn condor() -> [Leg; 4] {
    [
        Leg { strike: 795.0, is_call: true, side: Side::Sell },
        Leg { strike: 815.0, is_call: true, side: Side::Buy },
        Leg { strike: 719.0, is_call: false, side: Side::Sell },
        Leg { strike: 690.0, is_call: false, side: Side::Buy },
    ]
}

fn narrow_condor() -> [Leg; 4] {
    [
        Leg { strike: 795.0, is_call: true, side: Side::Sell },
        Leg { strike: 815.0, is_call: true, side: Side::Buy },
        Leg { strike: 719.0, is_call: false, side: Side::Sell },
        Leg { strike: 699.0, is_call: false, side: Side::Buy },
    ]
}

/// Both put legs on the sell side: no matched short/long pair, infinite width.
fn malformed_legs() -> [Leg; 4] {
    [
        Leg { strike: 795.0, is_call: true, side: Side::Sell },
        Leg { strike: 815.0, is_call: true, side: Side::Buy },
        Leg { strike: 719.0, is_call: false, side: Side::Sell },
        Leg { strike: 700.0, is_call: false, side: Side::Sell },
    ]
}

fn calm_book() -> NormalizedIpr {
    NormalizedIpr::compute_u16(&[500, 300, 200, 100, 80, 60, 40, 20])
}

fn uniform_book() -> NormalizedIpr {
    NormalizedIpr::compute_u16(&[100, 100, 100, 100])
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "Gate A: Illegal Transition",
        threat: "Order attempts SpreadOpen while already SpreadOpen — un-witnessed edge.",
        position_state: STATE_SPREAD_OPEN,
        verdict: OracleVerdict::StructuralEquilibrium,
        purity: calm_book,
        legs: narrow_condor,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::IllegalTransition),
    },
    Scenario {
        name: "Gate B: Verdict Veto",
        threat: "Dual-oracle disagreement escalates to CriticalEscalation — no trade.",
        position_state: STATE_FLAT,
        verdict: OracleVerdict::CriticalEscalation,
        purity: calm_book,
        legs: condor,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::VerdictVeto),
    },
    Scenario {
        name: "Gate C: Chaotic Book",
        threat: "Uniform volume across strikes — 0 pmy, volume-dead chain, below floor.",
        position_state: STATE_FLAT,
        verdict: OracleVerdict::StructuralEquilibrium,
        purity: uniform_book,
        legs: condor,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::ChaoticBook),
    },
    Scenario {
        name: "Gate D: Malformed Legs",
        threat: "Both put legs submitted sell-side — no short/long pair, unhedged risk.",
        position_state: STATE_FLAT,
        verdict: OracleVerdict::StructuralEquilibrium,
        purity: calm_book,
        legs: malformed_legs,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::MalformedLegs),
    },
    Scenario {
        name: "Gate E: Max-Loss Veto",
        threat: "29-wide put wing on $3.75 credit — max loss exceeds 2% of $100k balance.",
        position_state: STATE_FLAT,
        verdict: OracleVerdict::StructuralEquilibrium,
        purity: calm_book,
        legs: condor,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::MaxLossVeto),
    },
    Scenario {
        name: "Gate F: CLI Refusal",
        threat: "All five in-process gates pass — final seam is the API subprocess itself.",
        position_state: STATE_FLAT,
        verdict: OracleVerdict::StructuralEquilibrium,
        purity: calm_book,
        legs: narrow_condor,
        credit: 3.75,
        balance: 100_000.0,
        expect: |r| matches!(r, DispatchRefusal::Cli(_)),
    },
];

struct ChaosLog {
    tick: u64,
    gate_name: &'static str,
    threat: &'static str,
    status: &'static str,
    detail: String,
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn write_report(tick: u64, gates_tested: u64, defended: u64, failed: u64, logs: &[ChaosLog]) {
    let status = if failed == 0 { "SECURE (ALL GATES HOLDING)" } else { "BREACH DETECTED" };
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"system_status\": \"{status}\",\n"));
    json.push_str(&format!("  \"tick\": {tick},\n"));
    json.push_str(&format!("  \"gates_tested\": {gates_tested},\n"));
    json.push_str(&format!("  \"gates_defended\": {defended},\n"));
    json.push_str(&format!("  \"gates_failed\": {failed},\n"));
    json.push_str("  \"live_sabotage_logs\": [\n");
    for (i, log) in logs.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"tick\": {},\n", log.tick));
        json.push_str(&format!("      \"gate_name\": \"{}\",\n", log.gate_name));
        json.push_str(&format!("      \"threat_description\": \"{}\",\n", log.threat));
        json.push_str(&format!("      \"status\": \"{}\",\n", log.status));
        json.push_str(&format!("      \"detail\": \"{}\"\n", json_escape(&log.detail)));
        json.push_str(if i + 1 < logs.len() { "    },\n" } else { "    }\n" });
    }
    json.push_str("  ]\n}\n");

    if let Ok(mut f) = std::fs::File::create(r".forge\sim\live_chaos_report.json") {
        let _ = f.write_all(json.as_bytes());
    }
}

fn main() {
    let cli = AlpacaCli::new(r"Z:\nope\alpaca.exe");
    let creds = AlpacaCredentials {
        key_id: SecureSecret::new(b"k".to_vec()),
        secret_key: SecureSecret::new(b"s".to_vec()),
        base_url: String::new(),
    };
    let max_ticks: Option<u64> = std::env::var("CHAOS_LOOP_TICKS").ok().and_then(|v| v.parse().ok());
    let tick_ms: u64 = std::env::var("CHAOS_TICK_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(500);

    eprintln!(
        "[chaos_monkey] replaying {} gate scenarios{}, purity floor {} pmy, dead CLI only, no live order ever reaches Alpaca",
        SCENARIOS.len(),
        max_ticks.map(|t| format!(", {t} ticks then exit")).unwrap_or_default(),
        CHAIN_PURITY_FLOOR_PMY
    );

    let mut logs: Vec<ChaosLog> = Vec::new();
    let mut gates_tested: u64 = 0;
    let mut defended: u64 = 0;
    let mut failed: u64 = 0;
    let mut tick: u64 = 0;

    loop {
        if let Some(max) = max_ticks {
            if tick >= max {
                eprintln!("[chaos_monkey] reached CHAOS_LOOP_TICKS={max}, exiting");
                break;
            }
        }

        let scenario = &SCENARIOS[(tick as usize) % SCENARIOS.len()];
        let health = AlpacaDaemonHealth::default();
        let result = dispatch_spread(
            &cli,
            &creds,
            &health,
            scenario.position_state,
            scenario.verdict,
            &(scenario.purity)(),
            &(scenario.legs)(),
            scenario.credit,
            scenario.balance,
            "SPY",
            "261016",
            1,
            3.50,
        );

        gates_tested += 1;
        let (status, detail) = match &result {
            Err(r) if (scenario.expect)(r) => {
                defended += 1;
                ("DEFENDED", format!("refused as expected: {r:?}"))
            }
            Err(r) => {
                failed += 1;
                (
                    "FAILED",
                    format!("refused, but with the WRONG reason (expected a different gate): {r:?}"),
                )
            }
            Ok(_) => {
                failed += 1;
                ("FAILED", "ORDER ACCEPTED — this scenario should have been refused".to_string())
            }
        };

        eprintln!("[chaos_monkey] tick {tick}: {} -> {status} ({detail})", scenario.name);
        logs.push(ChaosLog { tick, gate_name: scenario.name, threat: scenario.threat, status, detail });
        if logs.len() > 12 {
            logs.remove(0);
        }

        write_report(tick, gates_tested, defended, failed, &logs);
        tick += 1;
        std::thread::sleep(std::time::Duration::from_millis(tick_ms));
    }

    eprintln!("[chaos_monkey] final: {gates_tested} tested, {defended} defended, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}
