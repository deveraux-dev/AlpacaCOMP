//! Options strategy leg selection: Iron Condor and Iron Butterfly.
//!
//! Net-new (drain confirmed absent, same as `greeks.rs`). Encodes RAMUSPRIME
//! spec rules 1-4. Strike selection reads ONLY from a caller-provided
//! `ChainQuote` snapshot — the model is never permitted to invent a strike
//! or a Greek; if a target delta isn't present in the snapshot, selection
//! returns `None` rather than guessing, same crucible-mask discipline as
//! `order_dag.rs`'s `ORDER_REJECT`.

/// Buy or sell side of a leg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

/// A single quoted strike's real market data — never model-generated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChainQuote {
    pub strike: f64,
    /// Call delta, in `(0, 1)`.
    pub call_delta: f64,
    /// Put delta, in `(-1, 0)`.
    pub put_delta: f64,
}

/// One leg of a constructed strategy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Leg {
    pub strike: f64,
    pub is_call: bool,
    pub side: Side,
}

/// Which structure the dual-oracle verdict + market purity authorizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    /// Neutral, diffuse (non-pinned) book: wide-body Iron Condor.
    IronCondor,
    /// Neutral, landmark-concentrated (pinned) book: narrow Iron Butterfly.
    IronButterfly,
    /// Mild directional consensus: reduced-allocation vertical spread (not
    /// yet leg-constructed here — placeholder for a future card).
    DirectionalVertical,
    /// Critical escalation or provenance breach: no trade.
    NoTrade,
}

/// Rule 1: trigger selection from the oracle verdict, implied-vol rank, and
/// market-purity concentration. `ivr` is implied-vol rank (0-100).
pub fn select_strategy(
    verdict: crate::oracle_arbiter::OracleVerdict,
    ivr: f64,
    purity: &crate::market_purity::NormalizedIpr,
) -> StrategyKind {
    use crate::oracle_arbiter::OracleVerdict;

    match verdict {
        OracleVerdict::StructuralEquilibrium if ivr > 30.0 => {
            if purity.is_landmark() {
                StrategyKind::IronButterfly
            } else {
                StrategyKind::IronCondor
            }
        }
        OracleVerdict::StructuralEquilibrium => StrategyKind::NoTrade, // IV too low to sell premium
        OracleVerdict::ScheduledMaintenance => StrategyKind::DirectionalVertical,
        OracleVerdict::CriticalEscalation | OracleVerdict::ProvenanceBreach => StrategyKind::NoTrade,
    }
}

/// Find the quote whose call delta is nearest `target`, among quotes with a
/// positive (OTM call) delta, and within `max_deviation` of it. `None` if
/// the snapshot is empty OR the nearest available delta is too far from
/// `target` to honestly call it that leg — the polysynthetic assembly rule:
/// a missing mandatory component aborts the build, it never substitutes.
fn nearest_call_delta<'a>(quotes: &'a [ChainQuote], target: f64, max_deviation: f64) -> Option<&'a ChainQuote> {
    quotes
        .iter()
        .filter(|q| q.call_delta > 0.0)
        .min_by(|a, b| {
            (a.call_delta - target).abs()
                .partial_cmp(&(b.call_delta - target).abs())
                .unwrap()
        })
        .filter(|q| (q.call_delta - target).abs() <= max_deviation)
}

/// Find the quote whose |put delta| is nearest `target`, among quotes with a
/// negative (OTM put) delta, and within `max_deviation` of it. Same
/// structural-rigidity rule as [`nearest_call_delta`].
fn nearest_put_delta<'a>(quotes: &'a [ChainQuote], target: f64, max_deviation: f64) -> Option<&'a ChainQuote> {
    quotes
        .iter()
        .filter(|q| q.put_delta < 0.0)
        .min_by(|a, b| {
            (a.put_delta.abs() - target).abs()
                .partial_cmp(&(b.put_delta.abs() - target).abs())
                .unwrap()
        })
        .filter(|q| (q.put_delta.abs() - target).abs() <= max_deviation)
}

/// Widest quoted call wing above `short_strike` within `max_width`; `None`
/// if no quoted strike sits in `(short_strike, short_strike + max_width]`.
fn widest_call_wing_within(quotes: &[ChainQuote], short_strike: f64, max_width: f64) -> Option<&ChainQuote> {
    quotes
        .iter()
        .filter(|q| q.strike > short_strike && q.strike - short_strike <= max_width)
        .max_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap())
}

/// Widest quoted put wing below `short_strike` within `max_width`; `None`
/// if no quoted strike sits in `[short_strike - max_width, short_strike)`.
fn widest_put_wing_within(quotes: &[ChainQuote], short_strike: f64, max_width: f64) -> Option<&ChainQuote> {
    quotes
        .iter()
        .filter(|q| q.strike < short_strike && short_strike - q.strike <= max_width)
        .min_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap())
}

/// Rule 2: build a 4-leg Iron Condor — sell the `short_delta` strikes, buy
/// the further-OTM `long_delta` strikes as wings. Polysynthetic assembly:
/// all 4 mandatory components (short call, long call, short put, long put)
/// must be present within `max_deviation` of their target delta AND the
/// wing-outside-short strike ordering must hold, or the whole assembly
/// aborts and returns `None` — never a partial or substituted structure.
///
/// `max_wing_width`: hard cap on strike distance from short to wing, sized
/// by the caller so worst-case loss `(width - credit) * 100` clears the
/// risk_router 2%-of-balance veto. A delta-selected wing wider than the cap
/// is pulled in to the widest quoted strike inside the cap (chain-supplied,
/// never invented); if no quoted strike fits, the whole assembly aborts.
pub fn build_iron_condor(
    quotes: &[ChainQuote],
    short_delta: f64,
    long_delta: f64,
    max_deviation: f64,
    max_wing_width: f64,
) -> Option<[Leg; 4]> {
    let short_call = nearest_call_delta(quotes, short_delta, max_deviation)?;
    let short_put = nearest_put_delta(quotes, short_delta, max_deviation)?;

    // Delta tolerance still gates existence; the cap only pulls an
    // over-wide (but honest) wing in — it never rescues a missing delta.
    let delta_call = nearest_call_delta(quotes, long_delta, max_deviation)?;
    let long_call = if delta_call.strike - short_call.strike <= max_wing_width {
        delta_call
    } else {
        widest_call_wing_within(quotes, short_call.strike, max_wing_width)?
    };
    let delta_put = nearest_put_delta(quotes, long_delta, max_deviation)?;
    let long_put = if short_put.strike - delta_put.strike <= max_wing_width {
        delta_put
    } else {
        widest_put_wing_within(quotes, short_put.strike, max_wing_width)?
    };

    if long_call.strike <= short_call.strike || long_put.strike >= short_put.strike {
        return None; // Wings must sit outside the short strikes; refuse otherwise.
    }

    Some([
        Leg { strike: short_call.strike, is_call: true, side: Side::Sell },
        Leg { strike: long_call.strike, is_call: true, side: Side::Buy },
        Leg { strike: short_put.strike, is_call: false, side: Side::Sell },
        Leg { strike: long_put.strike, is_call: false, side: Side::Buy },
    ])
}

/// Rule 2 (butterfly variant): sell the single nearest-ATM strike as both
/// call and put body, buy `wing_delta` wings on each side. Same
/// polysynthetic assembly rule as [`build_iron_condor`]: body + both wings
/// must each clear `max_deviation`, or the whole build aborts.
/// `max_wing_width` mirrors [`build_iron_condor`]: over-wide honest wings
/// pull in to the widest quoted strike inside the cap, never rescue a
/// missing delta. (Cap-on-butterfly idea: partner's 20a1ea8.)
pub fn build_iron_butterfly(quotes: &[ChainQuote], wing_delta: f64, max_deviation: f64, max_wing_width: f64) -> Option<[Leg; 4]> {
    // Body strike: the call delta closest to 0.50 (ATM).
    let body = nearest_call_delta(quotes, 0.50, max_deviation)?;
    let delta_call = nearest_call_delta(quotes, wing_delta, max_deviation)?;
    let long_call = if delta_call.strike - body.strike <= max_wing_width {
        delta_call
    } else {
        widest_call_wing_within(quotes, body.strike, max_wing_width)?
    };
    let delta_put = nearest_put_delta(quotes, wing_delta, max_deviation)?;
    let long_put = if body.strike - delta_put.strike <= max_wing_width {
        delta_put
    } else {
        widest_put_wing_within(quotes, body.strike, max_wing_width)?
    };

    if long_call.strike <= body.strike || long_put.strike >= body.strike {
        return None;
    }

    Some([
        Leg { strike: body.strike, is_call: true, side: Side::Sell },
        Leg { strike: body.strike, is_call: false, side: Side::Sell },
        Leg { strike: long_call.strike, is_call: true, side: Side::Buy },
        Leg { strike: long_put.strike, is_call: false, side: Side::Buy },
    ])
}

/// Rule 3: pick the available expiration closest to the 45 DTE target.
/// `None` on an empty list.
pub fn nearest_target_dte(available_dtes: &[u32], target_dte: u32) -> Option<u32> {
    available_dtes
        .iter()
        .copied()
        .min_by_key(|&dte| (dte as i64 - target_dte as i64).abs())
}

/// Rule 4a: take-profit trips at 50% of max credit captured.
pub fn should_take_profit(entry_credit: f64, current_value: f64) -> bool {
    current_value <= 0.5 * entry_credit
}

/// Rule 4b: time-stop trips at or before 21 DTE, to cut tail-end gamma risk.
pub fn should_time_stop(current_dte: u32) -> bool {
    current_dte <= 21
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_purity::NormalizedIpr;
    use crate::oracle_arbiter::OracleVerdict;

    fn synthetic_chain() -> [ChainQuote; 7] {
        // Strikes ascending; call delta decreases, put delta (abs) increases
        // as strike rises, as in a real chain.
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
    fn selects_condor_on_equilibrium_high_iv_diffuse_book() {
        let purity = NormalizedIpr::compute_u16(&[10, 10, 10, 10]); // uniform: not landmark
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 45.0, &purity);
        assert_eq!(kind, StrategyKind::IronCondor);
    }

    #[test]
    fn selects_butterfly_on_equilibrium_high_iv_pinned_book() {
        let purity = NormalizedIpr::compute_u16(&[100, 0, 0, 0]); // spike: landmark
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 45.0, &purity);
        assert_eq!(kind, StrategyKind::IronButterfly);
    }

    #[test]
    fn no_trade_on_equilibrium_low_iv() {
        let purity = NormalizedIpr::compute_u16(&[10, 10, 10, 10]);
        let kind = select_strategy(OracleVerdict::StructuralEquilibrium, 20.0, &purity);
        assert_eq!(kind, StrategyKind::NoTrade);
    }

    #[test]
    fn no_trade_on_critical_escalation_and_breach() {
        let purity = NormalizedIpr::compute_u16(&[10, 10, 10, 10]);
        assert_eq!(select_strategy(OracleVerdict::CriticalEscalation, 45.0, &purity), StrategyKind::NoTrade);
        assert_eq!(select_strategy(OracleVerdict::ProvenanceBreach, 45.0, &purity), StrategyKind::NoTrade);
    }

    #[test]
    fn builds_iron_condor_from_16_5_delta_wings() {
        let chain = synthetic_chain();
        let legs = build_iron_condor(&chain, 0.16, 0.05, 0.01, 100.0).expect("condor should build");

        assert_eq!(legs[0], Leg { strike: 105.0, is_call: true, side: Side::Sell }); // short 16d call
        assert_eq!(legs[1], Leg { strike: 110.0, is_call: true, side: Side::Buy });  // long 5d call
        assert_eq!(legs[2], Leg { strike: 95.0, is_call: false, side: Side::Sell }); // short 16d put
        assert_eq!(legs[3], Leg { strike: 90.0, is_call: false, side: Side::Buy });  // long 5d put
    }

    #[test]
    fn builds_iron_butterfly_from_atm_body_and_wings() {
        let chain = synthetic_chain();
        let legs = build_iron_butterfly(&chain, 0.05, 0.01, 100.0).expect("butterfly should build");

        assert_eq!(legs[0], Leg { strike: 100.0, is_call: true, side: Side::Sell });
        assert_eq!(legs[1], Leg { strike: 100.0, is_call: false, side: Side::Sell });
        assert_eq!(legs[2], Leg { strike: 110.0, is_call: true, side: Side::Buy });
        assert_eq!(legs[3], Leg { strike: 90.0, is_call: false, side: Side::Buy });
    }

    #[test]
    fn refuses_condor_on_empty_snapshot_never_guesses() {
        assert!(build_iron_condor(&[], 0.16, 0.05, 0.01, 100.0).is_none());
    }

    #[test]
    fn refuses_condor_when_wing_delta_missing_from_book() {
        // Chain has no strike anywhere near 5-delta (closest is 16-delta) —
        // the polysynthetic assembly must abort, not substitute the 16-delta
        // strike for the missing 5-delta wing.
        let thin_chain = [
            ChainQuote { strike: 95.0, call_delta: 0.80, put_delta: -0.16 },
            ChainQuote { strike: 100.0, call_delta: 0.50, put_delta: -0.50 },
            ChainQuote { strike: 105.0, call_delta: 0.16, put_delta: -0.80 },
        ];
        assert!(build_iron_condor(&thin_chain, 0.16, 0.05, 0.02, 100.0).is_none());
    }

    #[test]
    fn refuses_butterfly_when_wing_delta_missing_from_book() {
        let thin_chain = [
            ChainQuote { strike: 95.0, call_delta: 0.80, put_delta: -0.16 },
            ChainQuote { strike: 100.0, call_delta: 0.50, put_delta: -0.50 },
            ChainQuote { strike: 105.0, call_delta: 0.16, put_delta: -0.80 },
        ];
        assert!(build_iron_butterfly(&thin_chain, 0.05, 0.02, 100.0).is_none());
    }

    #[test]
    fn accepts_wing_delta_within_tolerance_but_not_exact() {
        // 7-delta wing available where 5-delta was requested; within a 3pt
        // tolerance this is an honest substitute, not a guess.
        let chain = [
            ChainQuote { strike: 95.0, call_delta: 0.80, put_delta: -0.16 },
            ChainQuote { strike: 100.0, call_delta: 0.50, put_delta: -0.50 },
            ChainQuote { strike: 105.0, call_delta: 0.16, put_delta: -0.80 },
            ChainQuote { strike: 108.0, call_delta: 0.07, put_delta: -0.93 },
            ChainQuote { strike: 92.0, call_delta: 0.93, put_delta: -0.07 },
        ];
        let legs = build_iron_condor(&chain, 0.16, 0.05, 0.03, 100.0).expect("7-delta clears a 3pt tolerance");
        assert_eq!(legs[1].strike, 108.0);
        assert!(build_iron_condor(&chain, 0.16, 0.05, 0.01, 100.0).is_none(), "same book fails a tight 1pt tolerance");
    }

    #[test]
    fn wing_cap_pulls_delta_selected_wing_inside_cap() {
        // Mirrors the 2026-09-01 sim finding: 29-wide put wing (719 short /
        // 690 long, $3.755 credit) trips exceeds_max_loss_veto on $100k
        // ($2,525 > $2,000). Under a 20-point cap the build must swap the
        // 5-delta wing for the widest quoted strike inside the cap.
        let chain = [
            ChainQuote { strike: 690.0, call_delta: 0.99, put_delta: -0.05 }, // 29 out: over cap
            ChainQuote { strike: 700.0, call_delta: 0.97, put_delta: -0.08 }, // 19 out: widest legal
            ChainQuote { strike: 710.0, call_delta: 0.90, put_delta: -0.11 },
            ChainQuote { strike: 719.0, call_delta: 0.84, put_delta: -0.16 }, // short put
            ChainQuote { strike: 795.0, call_delta: 0.16, put_delta: -0.84 }, // short call
            ChainQuote { strike: 815.0, call_delta: 0.05, put_delta: -0.95 }, // 20 out: exactly at cap
        ];
        let legs = build_iron_condor(&chain, 0.16, 0.05, 0.01, 20.0).expect("capped condor should build");
        assert_eq!(legs[1], Leg { strike: 815.0, is_call: true, side: Side::Buy });
        assert_eq!(legs[3], Leg { strike: 700.0, is_call: false, side: Side::Buy });
    }

    #[test]
    fn wing_cap_refuses_when_no_quoted_strike_fits() {
        // Only wing quotes sit beyond the cap on the put side — abort, never
        // invent an intermediate strike.
        let chain = [
            ChainQuote { strike: 690.0, call_delta: 0.99, put_delta: -0.05 }, // 29 out: over 20 cap
            ChainQuote { strike: 719.0, call_delta: 0.84, put_delta: -0.16 },
            ChainQuote { strike: 795.0, call_delta: 0.16, put_delta: -0.84 },
            ChainQuote { strike: 815.0, call_delta: 0.05, put_delta: -0.95 },
        ];
        assert!(build_iron_condor(&chain, 0.16, 0.05, 0.01, 20.0).is_none());
    }

    #[test]
    fn nearest_dte_picks_closest_to_45() {
        let available = [7u32, 30, 45, 60, 90];
        assert_eq!(nearest_target_dte(&available, 45), Some(45));
        let available2 = [21u32, 40, 90];
        assert_eq!(nearest_target_dte(&available2, 45), Some(40));
    }

    #[test]
    fn exit_rules_trip_correctly() {
        assert!(should_take_profit(2.00, 1.00)); // 50% captured
        assert!(!should_take_profit(2.00, 1.50)); // only 25% captured
        assert!(should_time_stop(21));
        assert!(should_time_stop(10));
        assert!(!should_time_stop(22));
    }
}
