# Target Policy — AlpacaCOMP

## Cost and spend guardrails

- No paid APIs.
- No SERP services.
- No proxy services.
- No cloud processing.
- No external LLM APIs.
- No broad crawling.
- No login walls.
- Fail closed when content cannot be fetched safely.

## Fetch limits

- Max pages per domain per run: 20
- Max total pages per run: 60
- Max HTML bytes per page: 2 MB
- Max total download bytes per run: 50 MB
- Timeout per request: 10 seconds connect, 20 seconds total
- Use a static, honest user-agent

## Pinned domains

- Alpaca (own ecosystem — docs, CLI, MCP server, API surface): `alpaca.markets`
- Composer (automated systematic/options strategy platform): `composer.trade`
- Option Alpha (automated options strategy bot, direct Iron Condor/Butterfly comparable): `optionalpha.com`
- QuantConnect (algo trading dev + backtest platform): `quantconnect.com`
- tastytrade (options strategy automation, industry benchmark for condor/butterfly mechanics): `tastytrade.com`
- Numerai (AI-agent trading tournament, comparable "AI agent trades capital" framing): `numer.ai`

## Allowlisted paths

Only same-domain links containing one of:

- `/docs`
- `/api`
- `/products`
- `/solutions`
- `/platform`
- `/strategies`
- `/pricing`
- `/security`
- `/trust`
- `/blog`
- `/case-studies`
- `/backtesting`
- `/automation`
- `/whitepaper`
- `/pdf`

## Disallowed paths

- `/login`
- `/signin`
- `/account`
- `/checkout`
- session-heavy or tracking-heavy querystrings
- unrelated subdomains unless explicitly added by the user

## Local output root

Default Windows root:

`F:\AlpacaCOMP\market-intel\`

Folders:

- `cache/raw_html/<domain>/<YYYY-MM-DD>/<sha256>.html`
- `cache/raw_pdf/<domain>/<YYYY-MM-DD>/<sha256>.pdf`
- `cache/headers/<domain>/<YYYY-MM-DD>/<sha256>.json`
- `extract/claims/<YYYY-MM-DD>/claims.jsonl`
- `extract/primitives/<YYYY-MM-DD>/primitives.json`
- `route/<YYYY-MM-DD>/router_output.md`
- `route/<YYYY-MM-DD>/implementation_tasks.md`
- `logs/<YYYY-MM-DD>/run_log.json`
- `logs/<YYYY-MM-DD>/errors.jsonl`

Note: this output root is a plain directory under AlpacaCOMP, not tracked by any VCS action from this skill (GIT=0 applies — no git commands are ever run here).
