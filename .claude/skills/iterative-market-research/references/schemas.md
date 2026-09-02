# Schemas

## Claim record JSONL

```json
{
  "claim_id": "MI-CL-YYYYMMDD-####",
  "domain": "composer.trade",
  "url": "https://...",
  "retrieved_at_utc": "2026-02-16T20:00:00Z",
  "page_title": "...",
  "claim_type": "strategy|risk_management|latency|execution|automation|backtesting|capital_allocation|options_workflow|api_access|pricing|compliance|autonomy",
  "primitive": "iron_condor_automation|risk_bounded_sizing|low_latency_telemetry|backtesting_engine|paper_trading_infra|broker_api_cli_mcp|volatility_triggered_strategy|dual_oracle_consensus|take_profit_time_stop|margin_exposure_guardrail|deterministic_risk_gate",
  "snippet": "<=25 words, verbatim",
  "paraphrase": "short paraphrase with no overclaim",
  "confidence": "high|medium|low",
  "evidence_pointer": {
    "cache_path": "F:\\AlpacaCOMP\\market-intel\\cache\\raw_html\\...",
    "sha256": "..."
  },
  "notes": "extraction caveats or missing context"
}
```

## Primitive summary

```json
{
  "date": "YYYY-MM-DD",
  "primitive": "deterministic_risk_gate",
  "competitor_domains": [],
  "evidence_count": 0,
  "confidence": "high|medium|low",
  "build_router_decision": "copy|invert|ignore|watch|weaponize",
  "why_it_matters": "",
  "alpacacomp_mapping": "",
  "next_action": ""
}
```

## Running report ledger

```json
{
  "report_date": "YYYY-MM-DD",
  "days_to_deadline": 0,
  "cadence": "every_other_day",
  "open_hypotheses": [],
  "validated_signals": [],
  "weakened_signals": [],
  "build_decisions": [],
  "writeup_decisions": [],
  "differentiator_wedges": [],
  "table_stakes_wedges": [],
  "next_report_focus": []
}
```
