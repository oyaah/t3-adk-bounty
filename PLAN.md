# PLAN — Win the $200 Terminal 3 Bug Bounty (street-smart)

**Goal:** Submit the most detailed, highest-volume bug + doc-gap report in the field (22 hackers).
**Deadline:** 2026-06-07 21:29 GMT+8.
**Philosophy:** Just-enough to win for sure. Maximize documented findings per hour. Error = prize.

## Insight (why this wins)
- $200 track judged on COUNT + DETAIL of bugs/doc gaps, not on a working agent.
- Beta docs = rough. Most gaps are findable by READING + cross-checking, no live run needed.
- Therefore: mine the whole docs site systematically NOW → 20+ verifiable gaps → unbeatable on volume.
- Live runtime errors are bonus, layered on top.

## Phases

### Phase 1 — Doc-gap mine (Claude does now; biggest lever) — verify: BUGLOG ≥ 20 items
- [ ] Pull full URL list from docs.terminal3.io/llms.txt
- [ ] Fetch every ADK page (overview, get-started, walkthrough, tips, T3N arch)
- [ ] Cross-check examples for: undefined symbols, missing imports, broken flow links,
      naming inconsistencies, dead steps, copy-paste-fail code, version mismatches
- [ ] Log each as BUG-NN with page/expected/actual/severity

### Phase 2 — Runtime scaffold (Claude writes; user runs) — verify: index.mjs + contract compile-attempt
- [ ] Write index.mjs (SDK config + register + invoke from docs) to trigger BUG-01/02/06 live
- [ ] Minimal Rust contract skeleton (Cargo.toml + wit + lib.rs) to trigger build-path bugs
- [ ] User runs Steps 1-4; pastes exact terminal errors into BUGLOG section B

### Phase 3 — Package submission — verify: pushed repo + writeup
- [ ] git init t3-bounty repo, push to github.com/oyaah
- [ ] Submission writeup = BUGLOG + summary table (counts by severity)
- [ ] Optional 60s screen-record of worst error

### Phase 4 — Ship + amplify — verify: DoraHacks submit confirmed
- [ ] Submit BUIDL on DoraHacks (GitHub link)
- [ ] Email same report to devrel@terminal3.io (signals "most detailed dev" = their exact scoring words)
- [ ] Submit before 21:29 GMT+8

## Street-smart edges
- Severity-tag everything → looks rigorous, easy for judge to skim.
- Include a fix suggestion per bug → "most detailed developer" framing.
- Group: A) doc gaps (reading), B) runtime (live). Shows breadth.
- CC the referral question if a friend also registers (extra prize hook).
