export const meta = {
  name: '777-cascade-uiux',
  description: 'K00 recon -> 7 students -> 7 teachers -> K07 master synthesis for the AlpacaCOMP judge-facing front end (zero-claims doctrine: every pixel traces to a receipt)',
  phases: [
    { title: 'Recon' },
    { title: 'Students' },
    { title: 'Teachers' },
    { title: 'Master' },
  ],
}

// Optional: pass the front-end root as args (string path). Falls back to repo root scan.
const FRONTEND_ROOT = (typeof args === 'string' && args.length > 0) ? args : 'F:\\AlpacaCOMP'

const GOAL = `Sean's partner is building the judge-facing front end (demo UI/dashboard) for the Alpaca AI Trading Agents Hackathon submission (deadline 2026-09-04T09:00 MDT). The engine behind it is real and test-pinned: a Rust no_std gate lattice where every order passes, in order, position-state DAG -> oracle verdict veto -> market-purity chaos gate -> leg-geometry check -> 2%-max-loss veto -> CLI subprocess (crates/forge-daemon/src/dispatch.rs), 118 tests green, paper account PA3FMNQT9WDW.

THE ZERO-CLAIMS LAW governs this UI: every number, state, and badge shown on screen MUST trace to a machine receipt (Alpaca API response, .forge/proof-ledger.tsv entry, test output, or live gate refusal). No invented values, no aspirational copy, no feature named on screen that is not wired in code. The strongest demo asset is the machine REFUSING a bad order with real numbers (the $2,525 > $2,000 max-loss veto) -- refusals are content, not errors to hide.

Front-end root under review: ${FRONTEND_ROOT}
Engine ground truth (read-only, cite file:line): F:\\AlpacaCOMP\\crates\\forge-gate\\src\\, F:\\AlpacaCOMP\\crates\\forge-daemon\\src\\dispatch.rs, F:\\AlpacaCOMP\\CLAUDE.md (architect_reprime ledger), F:\\AlpacaCOMP\\.forge\\proof-ledger.tsv

F01 (verify before citing) and F09 (name gaps plainly) are mandatory. If the front end does not exist yet or is a stub, say so plainly -- the cascade then designs it from zero rather than reviewing it.`

phase('Recon')
const recon = await agent(
  `${GOAL}

You are the K00 recon pass for a 777-cascade. Produce ONE shared ground-truth document every downstream student/teacher/master cites instead of re-reading cold.

Inventory (file:line citations mandatory; a path that does not exist gets named as absent, never assumed):
1. The front-end root: what actually exists -- framework, pages/components, data flow, what values each screen element displays and WHERE each value comes from (API call, hardcode, invention).
2. The engine's real displayable surface: every receipt-producing seam -- DispatchRefusal variants, NormalizedIpr permyriad readings (landmark>=7500/diffuse<2500), the six-gate order in dispatch_spread, proof-ledger.tsv row format, live GET /v2/account fields.
3. The judging context from CLAUDE.md: P&L is the scored objective; the submission needs a demo URL, video, write-up.

Output: one structured document, sectioned as above, ending with a GAPS section naming everything the demo needs that neither the front end nor the engine currently surfaces.`,
  { label: 'K00-recon', phase: 'Recon' }
)

phase('Students')
const ARCHETYPES = [
  { name: 'Skeptic', brief: 'Receipt audit. For EVERY value/state/badge the UI shows (or plans to show), trace its source. Flag anything that could ever render an invented, stale, or hardcoded number as if live. A UI element with no receipt path is an overclaim in pixels -- name each one.' },
  { name: 'Builder', brief: 'What is the minimum real screen that ships before 2026-09-04T09:00 MDT? Name the concrete gap between what exists and ONE honest demo view: live account equity, purity meter, gate chain with pass/refuse states, and the sealed decision log. Cheapest real stack given what is already in the repo.' },
  { name: 'Scout', brief: 'Prior art. What do winning hackathon trading demos and Alpaca\'s own dashboard actually show judges? What do judges of AI-agent hackathons reward on screen (autonomy evidence, P&L, safety)? Cite real examples where possible; mark inference as [UNVERIFIED].' },
  { name: 'DevilsAdvocate', brief: 'Argue the dashboard is the wrong artifact entirely for THIS submission: a 3-minute screen recording of the terminal -- live order passing six gates, then the oversized condor being REFUSED with real dollar numbers -- may beat a webpage built in 2 days. Where does a rushed UI actively hurt (bugs on camera, implied features, judge distrust)?' },
  { name: 'AlternativeTheorist', brief: 'Reframe: the UI is not a live view, it is a REPLAY of the Merkle-sealed decision ledger -- a scrubber over proof-ledger.tsv rows where every frame is a receipt by construction. Zero live-wiring risk, zero invented pixels, works offline in the video. Ground it in the actual ledger row format and what forge-gate merkle_seal actually chains.' },
  { name: 'Bridge', brief: 'The judge slot. A judge gives this 3 minutes and may not know options trading. What must they SEE to understand in 30 seconds that (a) the agent trades autonomously, (b) a deterministic gate lattice can refuse the model, (c) P&L is real? Where does trading jargon on screen (condor, permyriad, mleg) lose them, and what plain label replaces each?' },
  { name: 'Economist', brief: 'Cost/sequencing against the hard deadline. Rank every proposed screen element by demo-impact per build-hour. Name the single highest-risk item (most likely to break on camera) and what gets cut first. The write-up and video are ALSO unbuilt and share the same clock -- account for that.' },
]

const students = await parallel(ARCHETYPES.map((a, i) => () => agent(
  `You are Student ${i + 1}, archetype: ${a.name}, in a 777-cascade (7 independent students, zero cross-reads).

${GOAL}

GROUND TRUTH (K00 recon doc -- cite it; verify/extend specific claims with your own targeted Read/Grep, do not re-derive from scratch):
${recon}

YOUR ROLE: ${a.brief}

Be adversarial and specific, not diplomatic. Cite real file:line wherever you assert something exists. Mark unverifiable claims [UNVERIFIED]. Structured report, 300-600 words, for a synthesis agent cross-referencing 6 other independent students.`,
  { label: `student:${a.name}`, phase: 'Students' }
)))

phase('Teachers')
// Bridging pairs, indices: 0=Skeptic,1=Builder,2=Scout,3=DA,4=AT,5=Bridge,6=Economist
const PAIRS = [
  [3, 4], // DA x AT -- terminal-recording vs ledger-replay: same "no live wiring" instinct, which wins?
  [2, 4], // Scout x AT -- has anyone demoed a replay-as-UI before; does prior art validate or kill it?
  [0, 3], // Skeptic x DA -- if the UI can't be made receipt-pure in time, does DA's no-UI position win by default?
  [1, 3], // Builder x DA -- MVP screen vs no screen: what does the Builder's cheapest path have to prove to beat the recording?
  [5, 4], // Bridge x AT -- does a ledger scrubber make sense to a non-trading judge in 30 seconds, or is it engineer-bait?
  [6, 1], // Economist x Builder -- cost the MVP path against the shared video/write-up clock; what falls off?
  [5, 0], // Bridge x Skeptic -- plain-language labels vs receipt purity: does simplifying for judges create overclaims?
]

const teachers = await parallel(PAIRS.map(([i, j], k) => () => agent(
  `You are Teacher ${k + 1} in a 777-cascade. You get exactly ONE bridging question.

${GOAL}

GROUND TRUTH (K00 recon doc):
${recon}

Student ${i + 1} [${ARCHETYPES[i].name}] found:
${students[i]}

Student ${j + 1} [${ARCHETYPES[j].name}] found:
${students[j]}

BRIDGING QUESTION: These two students contradict, solve different problems, or combine via some mechanism. What is the resolution? Does their combination reveal an approach neither found alone? If you have tool access, directly verify (Read/Grep) the single most load-bearing factual claim either student made -- state whether you verified directly or reason from their quotes alone. 200-400 words.`,
  { label: `teacher:${k + 1}`, phase: 'Teachers' }
)))

phase('Master')
const master = await agent(
  `You are the Master synthesis agent closing a 777-cascade.

${GOAL}

K00 RECON:
${recon}

7 STUDENT REPORTS:
${ARCHETYPES.map((a, i) => `--- Student ${i + 1} [${a.name}] ---\n${students[i]}`).join('\n\n')}

7 TEACHER BRIDGING ANSWERS:
${PAIRS.map(([i, j], k) => `--- Teacher ${k + 1} (${ARCHETYPES[i].name} x ${ARCHETYPES[j].name}) ---\n${teachers[k]}`).join('\n\n')}

Produce the 8 mandatory Master sections, honestly, no hedging:
1. Verdict -- one paragraph: given the real front-end state and the deadline, what IS the demo artifact (live dashboard / ledger replay / terminal recording / hybrid)?
2. Critical Corrections -- cited to student/teacher, things earlier claims got wrong
3. High Priority -- what matters most, cited
4. Honest Inventory -- real file counts/components/receipt seams from the research, no round guesses, no calendar-time invention
5. MVP Path -- cheapest real build sequence for the chosen artifact, each step naming the receipt that backs each screen element
6. Inventions -- each real novel approach: which contradiction spawned it, which students/teachers contributed, why Scout's prior art doesn't cover it
7. Unconnected Collisions -- for every student pair NO teacher covered, one paragraph: missed finding there?
8. Open Questions

THEN, as a separate final deliverable, produce the DEMO SPEC the partner builds from: screen-by-screen (or scene-by-scene, if a recording won), where EVERY element carries its receipt source in the spec line itself (element -> file:line or API field or ledger column). Plain-language labels per the Bridge student for anything a non-trading judge sees. Every gap named plainly, zero coverage invented. Scoped to the spec -- do not rewrite engine code (diff-floor).`,
  { label: 'K07-master', phase: 'Master' }
)

return { recon, students, teachers, master }
