//! Multi-leg order dispatch: every order clears ALL gates in-process before
//! the CLI subprocess ever spawns. Refusals are typed and feed the governor's
//! risk_gate_faults axis. JSON body enters `alpaca api POST /v2/orders` argv-free.

use forge_gate::market_purity::NormalizedIpr;
use forge_gate::oracle_arbiter::OracleVerdict;
use forge_gate::order_dag::{AllowedTransition, OrderStateDag};
use forge_gate::risk_router::exceeds_max_loss_veto;
use forge_gate::strategy::{Leg, Side};

use crate::alpaca_cli::{AlpacaCli, CliRefusal};
use crate::config::AlpacaCredentials;

/// Live position lifecycle states witnessed in [`LIVE_ORDER_DAG`].
pub const STATE_FLAT: u32 = 0;
pub const STATE_SPREAD_OPEN: u32 = 1;
pub const STATE_SPREAD_CLOSING: u32 = 2;

/// The only order-state transitions the live path will ever witness:
/// Flat -> SpreadOpen -> SpreadClosing -> Flat. Anything else (open on
/// top of open, close from flat, unknown state) is clamped to refusal.
pub const LIVE_ORDER_DAG: OrderStateDag = OrderStateDag::from_nodes(&[
    AllowedTransition::new(STATE_FLAT, 0xD15F1A7, &[STATE_SPREAD_OPEN]),
    AllowedTransition::new(STATE_SPREAD_OPEN, 0xD150FE2, &[STATE_SPREAD_CLOSING]),
    AllowedTransition::new(STATE_SPREAD_CLOSING, 0xD15C105, &[STATE_FLAT]),
]);

/// Chain-feed purity band (preauth 2026-09-03): mass = per-contract dailyBar
/// volume. Below floor = volume-dead/stale chain; above ceiling = panic
/// concentration. Calibrated 2026-09-02: calm live chain read 625 pmy on
/// volume mass vs 46-49 on quote sizes (quote sizes carry no signal).
pub const CHAIN_PURITY_FLOOR_PMY: u16 = 400;
pub const CHAIN_PURITY_CEILING_PMY: u16 = 7500;

#[derive(Debug, PartialEq)]
pub enum DispatchRefusal {
    /// Position-state transition not witnessed in [`LIVE_ORDER_DAG`].
    IllegalTransition,
    /// Arbiter verdict does not authorize neutral spreads.
    VerdictVeto,
    /// Chain volume mass outside [CHAIN_PURITY_FLOOR_PMY, CHAIN_PURITY_CEILING_PMY]:
    /// volume-dead/stale below, panic-concentrated above. Refuses execution.
    ChaoticBook,
    /// Wing width vs credit trips the 2%-of-balance max-loss veto.
    MaxLossVeto,
    /// Leg geometry is not a credit structure (short inside longs).
    MalformedLegs,
    /// CLI subprocess refused (spawn/exit/parse) — carries the inner refusal.
    Cli(CliRefusal),
}

/// OCC option symbol: root + YYMMDD + C/P + strike*1000 zero-padded to 8.
pub fn occ_symbol(root: &str, yymmdd: &str, is_call: bool, strike: f64) -> String {
    format!("{root}{yymmdd}{}{:08}", if is_call { 'C' } else { 'P' }, (strike * 1000.0).round() as u64)
}

/// Build the `order_class=mleg` limit-order body for a 4-leg spread.
/// `limit_price` sign per Alpaca mleg convention (verified 2026-09-01,
/// docs.alpaca.markets + alpaca.markets/learn): net credit = NEGATIVE,
/// net debit = positive. A credit condor entry MUST pass a negative value.
pub fn mleg_body(root: &str, yymmdd: &str, legs: &[Leg; 4], qty: u32, limit_price: f64) -> String {
    let leg_json: Vec<String> = legs
        .iter()
        .map(|l| {
            let side = if l.side == Side::Buy { "buy" } else { "sell" };
            let intent = if l.side == Side::Buy { "buy_to_open" } else { "sell_to_open" };
            format!(
                r#"{{"symbol":"{}","ratio_qty":"1","side":"{side}","position_intent":"{intent}"}}"#,
                occ_symbol(root, yymmdd, l.is_call, l.strike)
            )
        })
        .collect();
    format!(
        r#"{{"order_class":"mleg","qty":"{qty}","type":"limit","limit_price":"{limit_price:.2}","time_in_force":"day","legs":[{}]}}"#,
        leg_json.join(",")
    )
}

/// Widest wing of the spread in strike dollars — the max-loss driver.
fn max_wing_width(legs: &[Leg; 4]) -> f64 {
    let call_w = width(legs, true);
    let put_w = width(legs, false);
    if call_w > put_w { call_w } else { put_w }
}

fn width(legs: &[Leg; 4], calls: bool) -> f64 {
    let mut short = None;
    let mut long = None;
    for l in legs.iter().filter(|l| l.is_call == calls) {
        match l.side {
            Side::Sell => short = Some(l.strike),
            Side::Buy => long = Some(l.strike),
        }
    }
    match (short, long) {
        (Some(s), Some(l)) => (l - s).abs(),
        _ => f64::INFINITY,
    }
}

/// Gate, then dispatch. Order of gates is deliberate: cheapest verdict first,
/// subprocess last — a refused order costs zero syscalls.
#[allow(clippy::too_many_arguments)]
pub fn dispatch_spread(
    cli: &AlpacaCli,
    creds: &AlpacaCredentials,
    position_state: u32,
    verdict: OracleVerdict,
    purity: &NormalizedIpr,
    legs: &[Leg; 4],
    entry_credit: f64,
    account_balance: f64,
    root: &str,
    yymmdd: &str,
    qty: u32,
    limit_price: f64,
) -> Result<String, DispatchRefusal> {
    if !LIVE_ORDER_DAG.validate_path(&[position_state, STATE_SPREAD_OPEN]) {
        return Err(DispatchRefusal::IllegalTransition);
    }
    match verdict {
        OracleVerdict::StructuralEquilibrium | OracleVerdict::ScheduledMaintenance => {}
        _ => return Err(DispatchRefusal::VerdictVeto),
    }
    if purity.pmy < CHAIN_PURITY_FLOOR_PMY || purity.pmy > CHAIN_PURITY_CEILING_PMY {
        return Err(DispatchRefusal::ChaoticBook);
    }
    let wing = max_wing_width(legs);
    if !wing.is_finite() {
        return Err(DispatchRefusal::MalformedLegs);
    }
    if exceeds_max_loss_veto(wing, entry_credit, account_balance) {
        return Err(DispatchRefusal::MaxLossVeto);
    }

    let body = mleg_body(root, yymmdd, legs, qty, limit_price);
    cli.run_with_stdin(creds, &["api", "POST", "/v2/orders", "--body", "@-"], body.as_bytes())
        .map_err(DispatchRefusal::Cli)
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_gate::market_purity::NormalizedIpr;

    fn condor() -> [Leg; 4] {
        [
            Leg { strike: 795.0, is_call: true, side: Side::Sell },
            Leg { strike: 815.0, is_call: true, side: Side::Buy },
            Leg { strike: 719.0, is_call: false, side: Side::Sell },
            Leg { strike: 690.0, is_call: false, side: Side::Buy },
        ]
    }

    #[test]
    fn occ_symbol_matches_the_live_chain_format() {
        assert_eq!(occ_symbol("SPY", "261016", true, 795.0), "SPY261016C00795000");
        assert_eq!(occ_symbol("SPY", "261016", false, 690.0), "SPY261016P00690000");
        assert_eq!(occ_symbol("SPY", "261016", false, 762.5), "SPY261016P00762500");
    }

    #[test]
    fn mleg_body_carries_all_four_legs_with_intents() {
        let body = mleg_body("SPY", "261016", &condor(), 1, 3.50);
        assert!(body.contains(r#""order_class":"mleg""#));
        assert!(body.contains(r#""symbol":"SPY261016C00795000","ratio_qty":"1","side":"sell","position_intent":"sell_to_open""#));
        assert!(body.contains(r#""symbol":"SPY261016C00815000","ratio_qty":"1","side":"buy","position_intent":"buy_to_open""#));
        assert!(body.contains(r#""symbol":"SPY261016P00719000""#));
        assert!(body.contains(r#""symbol":"SPY261016P00690000""#));
        assert!(body.contains(r#""limit_price":"3.50""#));
        assert!(serde_json::from_str::<serde_json::Value>(&body).is_ok(), "body is valid JSON");
    }

    #[test]
    fn credit_entry_limit_price_is_negative_per_alpaca_mleg_convention() {
        // Alpaca mleg: credit = negative limit_price, debit = positive
        // (verified 2026-09-01 against Alpaca docs + learn guide). A condor
        // entered for a $3.28 credit must serialize as "-3.28".
        let body = mleg_body("SPY", "261016", &condor(), 1, -3.28);
        assert!(body.contains(r#""limit_price":"-3.28""#));
        assert!(serde_json::from_str::<serde_json::Value>(&body).is_ok(), "body is valid JSON");
    }

    #[test]
    fn widest_wing_drives_max_loss() {
        // Call wing 20 wide, put wing 29 wide -> 29 governs.
        assert_eq!(max_wing_width(&condor()), 29.0);
    }

    /// 20-wide wings: clears the 2% veto on $100k where the 29-wide does not.
    fn narrow_condor() -> [Leg; 4] {
        [
            Leg { strike: 795.0, is_call: true, side: Side::Sell },
            Leg { strike: 815.0, is_call: true, side: Side::Buy },
            Leg { strike: 719.0, is_call: false, side: Side::Sell },
            Leg { strike: 699.0, is_call: false, side: Side::Buy },
        ]
    }

    /// Calm live-chain volume shape: concentrated near-money, tapering wings.
    /// Lands inside [CHAIN_PURITY_FLOOR_PMY, CHAIN_PURITY_CEILING_PMY].
    fn calm_book() -> NormalizedIpr {
        NormalizedIpr::compute_u16(&[500, 300, 200, 100, 80, 60, 40, 20])
    }

    /// Panic shape: one contract swallows the volume — above the ceiling.
    fn panic_spike_book() -> NormalizedIpr {
        NormalizedIpr::compute_u16(&[1000, 50, 30, 20])
    }

    fn refused_without_subprocess(
        verdict: OracleVerdict,
        purity: &NormalizedIpr,
        legs: &[Leg; 4],
        credit: f64,
        balance: f64,
    ) -> DispatchRefusal {
        // A nonexistent exe proves refusal happened BEFORE any spawn attempt:
        // reaching the CLI would return ExeNotFound instead of the gate error.
        let cli = AlpacaCli::new(r"Z:\nope\alpaca.exe");
        let creds = crate::config::AlpacaCredentials {
            key_id: crate::secrets::SecureSecret::new(b"k".to_vec()),
            secret_key: crate::secrets::SecureSecret::new(b"s".to_vec()),
            base_url: String::new(),
        };
        dispatch_spread(&cli, &creds, STATE_FLAT, verdict, purity, legs, credit, balance, "SPY", "261016", 1, 3.50)
            .unwrap_err()
    }

    #[test]
    fn open_on_top_of_open_is_refused_before_every_other_gate() {
        // SpreadOpen -> SpreadOpen is un-witnessed in LIVE_ORDER_DAG: refused
        // even though verdict/purity/legs/loss would all pass.
        let cli = AlpacaCli::new(r"Z:\nope\alpaca.exe");
        let creds = crate::config::AlpacaCredentials {
            key_id: crate::secrets::SecureSecret::new(b"k".to_vec()),
            secret_key: crate::secrets::SecureSecret::new(b"s".to_vec()),
            base_url: String::new(),
        };
        let r = dispatch_spread(
            &cli, &creds, STATE_SPREAD_OPEN, OracleVerdict::StructuralEquilibrium,
            &calm_book(), &narrow_condor(), 3.75, 100_000.0, "SPY", "261016", 1, -3.50,
        )
        .unwrap_err();
        assert_eq!(r, DispatchRefusal::IllegalTransition);
    }

    #[test]
    fn unwitnessed_position_state_is_refused() {
        let cli = AlpacaCli::new(r"Z:\nope\alpaca.exe");
        let creds = crate::config::AlpacaCredentials {
            key_id: crate::secrets::SecureSecret::new(b"k".to_vec()),
            secret_key: crate::secrets::SecureSecret::new(b"s".to_vec()),
            base_url: String::new(),
        };
        let r = dispatch_spread(
            &cli, &creds, 999, OracleVerdict::StructuralEquilibrium,
            &calm_book(), &narrow_condor(), 3.75, 100_000.0, "SPY", "261016", 1, -3.50,
        )
        .unwrap_err();
        assert_eq!(r, DispatchRefusal::IllegalTransition);
    }

    #[test]
    fn live_dag_witnesses_exactly_the_three_hop_lifecycle() {
        assert!(LIVE_ORDER_DAG.validate_path(&[STATE_FLAT, STATE_SPREAD_OPEN, STATE_SPREAD_CLOSING, STATE_FLAT]));
        assert!(!LIVE_ORDER_DAG.validate_path(&[STATE_FLAT, STATE_SPREAD_CLOSING]));
        assert!(!LIVE_ORDER_DAG.validate_path(&[STATE_SPREAD_OPEN, STATE_FLAT]));
    }

    #[test]
    fn critical_verdict_is_refused_before_spawn() {
        let r = refused_without_subprocess(OracleVerdict::CriticalEscalation, &calm_book(), &condor(), 3.75, 100_000.0);
        assert_eq!(r, DispatchRefusal::VerdictVeto);
    }

    #[test]
    fn chaotic_book_is_refused_before_spawn() {
        // Uniform volume = 0 pmy = below floor (volume-dead chain): refused.
        let uniform = NormalizedIpr::compute_u16(&[100, 100, 100, 100]);
        let r = refused_without_subprocess(OracleVerdict::StructuralEquilibrium, &uniform, &condor(), 3.75, 100_000.0);
        assert_eq!(r, DispatchRefusal::ChaoticBook);
    }

    #[test]
    fn panic_spike_above_ceiling_is_refused_before_spawn() {
        // One contract swallowing the volume reads above CHAIN_PURITY_CEILING_PMY:
        // the band refuses BOTH ends, not just diffuse.
        assert!(panic_spike_book().pmy > CHAIN_PURITY_CEILING_PMY);
        let r = refused_without_subprocess(OracleVerdict::StructuralEquilibrium, &panic_spike_book(), &condor(), 3.75, 100_000.0);
        assert_eq!(r, DispatchRefusal::ChaoticBook);
    }

    #[test]
    fn calm_book_sits_inside_the_chain_purity_band() {
        let p = calm_book().pmy;
        assert!(p >= CHAIN_PURITY_FLOOR_PMY && p <= CHAIN_PURITY_CEILING_PMY, "calm fixture pmy={p}");
    }

    #[test]
    fn tonights_simmed_condor_trips_the_veto_on_100k() {
        // The 2026-09-01 sim pick: 29-wide put wing, $3.75 credit.
        // Max loss (29 - 3.75) * 100 = $2,525 > 2% of $100k. REFUSED — the
        // strategy layer must cap wings, not the gate loosen.
        let r = refused_without_subprocess(OracleVerdict::StructuralEquilibrium, &calm_book(), &condor(), 3.75, 100_000.0);
        assert_eq!(r, DispatchRefusal::MaxLossVeto);
    }

    #[test]
    fn a_clean_gate_pass_reaches_the_cli_layer() {
        // 20-wide wings: max loss (20 - 3.75) * 100 = $1,625 < $2,000 -> all
        // gates pass and the refusal comes from the (deliberately dead) CLI.
        let r = refused_without_subprocess(OracleVerdict::StructuralEquilibrium, &calm_book(), &narrow_condor(), 3.75, 100_000.0);
        assert_eq!(r, DispatchRefusal::Cli(CliRefusal::ExeNotFound(r"Z:\nope\alpaca.exe".into())));
    }
}
