# Design review: benchmark-run

Phase 2 hardening of the `givn` design review for `benchmark-run`. Phase 1
(fresh-context grilling) produced 20 ranked findings; the orchestrator put four
genuine decisions to the operator and recorded the answers below. This
document records the decision and the applied resolution for each finding. No
`tasks.md` was written, no production code was changed, no real evidence or
cassette/report was committed, and no `@e2e` tag was removed.

## Review metadata

| Field | Value |
|---|---|
| Change | `benchmark-run` |
| Capability | `benchmark-comparison` |
| Reviewed at stage | `arc42-docs-done` (proposal, specs, design, and arc42 docs complete; tasks and review absent) |
| Grilling | `DesignGrill`, fresh context, high effort |
| Hardening | `DesignHarden`, fresh context |
| Final sign-off | Ortsinn maintainer/orchestrator — pending; this file is the hardened input to that sign-off |
| Date | 2026-10-27 |
| Artifacts read | proposal, delta feature, delta/durable usecase, design, arc42 marker and chapters 01–12, predecessor design/review, benchmark source, fixtures, manifests, glossary, instructions |
| Tasks written | none |

## Operator decisions (2026-10-27)

| Q | Question | Answer applied |
|---|---|---|
| Q1 | Which model endpoints may the real record run use? | **Hybrid.** Loopback embedding endpoint for benchmark embeddings and candidate-internal embeddings; an external OpenAI-compatible explanation endpoint only for the at-most-80 reviewed gold explanation queries. Both localities, both model names, and the budget are recorded in the setup lock, provenance record, and report. Candidate-internal embedding calls are not benchmark model dependencies and are bounded only by the index deadline. No unbounded candidate-side external call. |
| Q2 | What is the minimum evidence a real report may publish? | **Per-family floor.** At least one measured candidate cell for typo, natural-language, explanation, and concept, and at least one measured cell for every manifest repository. The runner exits 2 before writing either report file when the floor is unmet; unavailable candidates remain in a published report. |
| Q3 | How deep is the gold set? | **Primary-deep, secondary-shallow.** Primary public repository: 50 typo, 30 natural-language, 20 explanation, 50 concept, 20 concept-equivalence (170 reviewed tasks). Each secondary public repository: 25/15/10/25/10 (85 reviewed tasks each), labelled lower-power and excluded from cross-repository aggregate claims. Minimums are per repository power. Synthetic gold is generated and reviewed with its generator. |
| Q4 | Must the 10M-line synthetic monorepo be attempted this round? | **Bounded attempt.** Every candidate is attempted on the synthetic monorepo with a 30-minute (1,800-second) index deadline. A breach is `index_timeout` with elapsed time, never zero or omitted. Manifest deadlines are updated: public 14,400 seconds, synthetic 1,800 seconds. |

## Grilling findings and dispositions

| ID | Finding | Disposition | Resolution in the artifacts |
|---|---|---|---|
| P0-1 | Real-run scenarios had no executable driver. | Closed | `design.md` "Scenario drivers" adds the scenario→driver contract for all 18 scenarios: 1/2/12 read committed manifest/setup lock/gold; 3/4 drive `benchmark/src/setup.rs` validation against a temporary manifest/checkout and assert exit 2/no run; 5/6/13/14 drive adapter normalization through `FakeSession`; 7 asserts committed recording mode, cassette fingerprints, model dependencies, and provenance (not a live record); 8/11/15 use `run_replay` over committed real cassettes. Scenario 7 is reworded to the committed evidence observable. |
| P0-2 | Unreviewed-draft semantics were contradictory and not RED-able. | Closed | Chosen semantic: a manifest-referenced gold set without `reviewed_by` fails before scoring with the typed `unreviewed gold set` error and no report; a draft the manifest does not reference is never loaded. Updated `design.md` "Gold sets", the delta usecase extension 21/R11, scenario "An unreviewed gold set referenced by the manifest stops before scoring", and QS-018. The scenario fails RED without the loader check. |
| P0-3 | Applying the published minimums to the shared loader would break the fixture suite. | Closed | Manifest-level `gold_minimums: {mode: published | fixture, primary: {...}, secondary: {...}}` plus `repositories[].power`; the fixture manifest sets `mode: fixture` and keeps its 5/4/4/5/4 sets, so the permanent suite stays green. The real manifest enforces per-power counts. Added scenario "A real gold set below the published minimum stops before scoring" and the typed `gold_below_minimum` error. |
| P0-4 | The publication floor was unrealizable and untested; the replay probe was undefined. | Closed | Per-family floor in `design.md` D3, "Report changes", and the runner; it exits 2 before either report file is written. Added scenario "A real run below the per-family measurement floor publishes no report". The replay probe is defined as `run_replay` over the committed real cassettes, asserting each installed candidate's measured slice replays identically (drivers 8/11/15). |
| P1-1 | Candidate pins were circular and reversed the predecessor's rejected option. | Closed | `design.md` D1 and "Version freshness": the manifest is the only pin origin, authored with evidence and committed; `setup.sh` installs exactly those pins and exits non-zero on drift; `setup.lock.json` records probes/toolchains only; `--refresh` is an explicit new pin and new record run. Chapter 2 constraints updated. |
| P1-2 | `benchmark/targets.yaml` was proposed then dropped. | Closed (kept) | `design.md` "Targets" defines loading, precedence (`--targets` > `targets_file` > inline), single-source validation, and the SHA-256 fingerprint recorded in the report and provenance. Scenario 15 asserts the real target labels, sources, and fingerprint; glossary gains "Targets file". |
| P1-3 | Provenance was unfalsifiable and the proposal named the wrong seed. | Closed | Proposal seed path corrected to `givn/ideation/repository-orientation/changes/benchmark-run.md`. `design.md` "Provenance record" defines the committed `benchmark/evidence/real-run/provenance.json` contents; scenarios 7/11/15 cross-check it against the manifest, setup lock, cassettes, and report. |
| P1-4 | The digest migration would redden a permanent scenario; S9's order was wrong; result-cassette keys could not be regenerated. | Closed | `design.md` "Fixture migration" plans the one-shot `migrate-cassettes` result-cassette key rewrite, model-cassette regeneration, and fingerprint update; "S9 replay recipe" fixes the order (copy, edit query, recompute gold fingerprint, replay, assert new-key miss and old key absent); a `@givn.modified @wip` delta scenario updates the permanent replay-fingerprint assertion to the manifest-declared, recomputed digest plus the digest-key shape. |
| P1-5 | Preparation scripts were excluded from the inventory by assertion while tested as actions. | Closed | Chosen option (b): `fetch-repos.sh`, `setup.sh`, and `run-real.sh` are project setup preconditions, not consumer actions; scenarios 3/4 are reworded to benchmark validation ("The real manifest and setup lock agree on every pin"; "A real fixed input that differs from its pin stops before scoring"). Chapter 3 separates the setup interface row from the consumer CLI; both usecase copies keep the same two inventory entries and no new `@e2e`. |
| P1-6 | The endpoint was an unrecorded operator dependency with unbounded candidate-side cost. | Closed | Hybrid endpoint decision applied across `design.md` (D1, "Model endpoints", version table, CLI flags, provenance, report), delta usecase preconditions/R15, chapter 2/3/6/7/8, glossary, and scenarios 7/15. Candidate-internal embeddings stay on loopback and are bounded only by the index deadline. |
| P1-7 | Carried D4 and the fourth-candidate rule were unresolved; D4 was double-claimed. | Closed | `design.md` records D4 once: ocbi results are not post-filtered, the typo no-match rate records its actual fuzzy behavior, and fuzziness is a reported limitation, never a normalization. It also records "no fourth candidate" with the coverage reason. The proposal crosswalk now maps gate D4 to the single run D4 entry. |
| P2-1 | The ADR verdict bundled candidates and chapter 4 contradicted chapter 9. | Closed | Five candidates were judged independently; candidate setup/pins, gold minimums, record gate/floor, and adapter/session route to `design.md`; the query-digest cassette-key format qualifies and is registered as ADR-0001 with a full MADR body and index. Each design entry now names its rejected alternatives and reason. Chapter 4 states one qualified ADR and routes the rest to `design.md`. |
| P2-2 | Arc42 chapter content was stale or self-contradictory. | Closed | Chapter 1 now points at `givn/specs/benchmark-comparison/` and separates the predecessor contract from the real path; chapter 9's ADR index claim is true because `docs/arc42/adr/README.md` now exists; glossary gains query digest and setup lock (plus power label, provenance record, publication floor, targets file); QS-018 fixed; all diagrams remain Mermaid. |
| P2-3 | `gold_fingerprints` collapsed multi-repository identity. | Closed | `reproduction.gold_fingerprints` is keyed `<repository>/<metric>`; `SCHEMA_VERSION` becomes 2; `Report::validate` checks every `inputs.gold` entry has one matching key; renderer and fixture step use the same key. |
| P2-4 | Real deadlines, setup-lock probes, `run-real.sh`, and the cell audit were unasserted; S11 was not executable. | Closed | Scenario 1 asserts 14,400/1,800-second deadlines; scenario 3 asserts probe records and the no-pin-origin rule; scenario 7 asserts the provenance command; scenario 11 audits candidate × repository × metric over the committed result cassette and `inputs.*` and replays each installed candidate's slice. |
| P2-5 | The durable usecase violated the scope criteria and diverged from the delta. | Closed | Delta `usecase.md` is the edited copy and the design states archive replaces the durable document; Goal/Trigger/guarantees carry no change ID in either copy; both interaction inventories use identical wording. |
| P2-6 | Adapter naming deviated from the predecessor without rationale. | Closed | `design.md` "Architecture impact" records the deliberate rename (`adapters/ocbi.rs`, `adapters/session.rs`) and that no predecessor interface was dropped. |

## Branch checklist

| Branch | Verdict | Evidence |
|---|---|---|
| Scope | **PASS** | `targets.yaml` is now designed, not dropped; "done" is unambiguous because the per-family floor and the gold-minimum check gate publication. |
| Tech choices | **PASS** | Manifest-authored pins, `CandidateSession`/`FakeSession`, record-once, hybrid endpoints, and the digest key fit the scenarios; rejected alternatives and reasons are recorded. |
| Missing scenarios | **PASS** | Floor, below-minimum gold, unreviewed referenced gold, deadlines, setup-lock probes, targets labels, provenance, and the replay probe all have scenarios. |
| Testability | **PASS** | Every scenario has a named driver and a concrete assertion; the strict runner, `filter_run_and_exit()`, `--name` bypass, and typed `NotImplemented` stub remain; the permanent modified scenario fails RED (keys lack the digest). |
| E2E fidelity | **PASS** | The inventory remains two CLI actions with two `@e2e` scenarios; no new `@e2e` and no tag removal. The real path is regular scenarios plus manual evidence, not a duplicate consumer action. |
| Interaction Coverage | **PASS** | Matrix rows and both usecase inventories match; setup scripts are documented as project setup preconditions rather than hidden actions. |
| Risk | **PASS** | The most likely failure (manual evidence chain stalls or fabricates) is mitigated by the provenance record and its cross-checking assertions; the synthetic run is bounded. |
| Use-case/persona context | **PASS** | Use-case identity, capability ownership, and confirmed/conditional personas are recorded; change IDs removed from durable Goal/Trigger/guarantees. |
| Decision ledger | **PASS** | D1, D2, D3, and D4 each appear once with owner/decision; the fourth-candidate rule is recorded; the seed path is corrected. |
| Visual design contract | **PASS** | Terminal interface, 80/160-column widths, no HTML/CSS/JS, `Interface: terminal`; real report adds rows, not a screen. |
| Deferrals / operator decisions | **PASS** | Four operator decisions are recorded; no mandatory check is deferred. |
| Ubiquitous language | **PASS** | query digest, setup lock, power label, provenance record, publication floor, and targets file are in the glossary with contrasts. |
| Feasibility / evidence honesty | **PASS** | Pins are authored, the synthetic run is bounded, the explanation budget is bounded, and the provenance record makes fabrication fail a scenario. |
| ADR qualification | **PASS** | Per-candidate verdicts in the marker and chapter 9; ADR-0001 minted with a real MADR body and directory index. |
| Arc42 | **PASS** | 12-row table agrees; content mismatches in ch01/ch04/ch09/ch10/ch12 fixed; diagrams are Mermaid. |

## Independent arc42 read

I re-derived the expected chapter set from the proposal, specs, and design
before opening `arc42.md`. My call was **Yes** for all twelve rows:

| # | Chapter | Mine | Marker | Reason |
|---|---|---|---|---|
| 1 | introduction-and-goals | Yes | Yes | New real-run goal/quality goal and stakeholder expectation. |
| 2 | architecture-constraints | Yes | Yes | Manifest pins, setup lock, hybrid endpoints, per-power gold, 1,800 s deadline, floor. |
| 3 | context-and-scope | Yes | Yes | Candidates, repositories, synthetic input, two endpoints, scripts, provenance. |
| 4 | solution-strategy | Yes | Yes | Adapter session, pins, gold power, floor, digest, provenance. |
| 5 | building-block-view | Yes | Yes | Provenance writer, targets loader, setup validation, migration, floor. |
| 6 | runtime-view | Yes | Yes | Real record/replay, hybrid endpoints, floor, provenance. |
| 7 | deployment-view | Yes | Yes | Work trees, tools, lock, endpoints, evidence paths. |
| 8 | crosscutting-concepts | Yes | Yes | Power, floor, provenance, endpoints, digest, review semantics. |
| 9 | architecture-decisions | Yes | Yes | Per-candidate verdicts and ADR-0001 register row. |
| 10 | quality-requirements | Yes | Yes | QS-018 fixed; QS-022–QS-024 added. |
| 11 | risks-and-technical-debt | Yes | Yes | R-009/R-012–R-016 updated; R-017 added. |
| 12 | glossary | Yes | Yes | query digest, setup lock, power label, provenance, floor, targets. |

All twelve chapter files exist, contain decision-specific content, and contain
no ASCII-art diagrams (box-drawing scan and pseudo-diagram scan both clean).

## Independent ADR check

| Candidate | Verdict | Route |
|---|---|---|
| Query digest in the model-cassette key | `QUALIFIED` — real alternatives, durable on-disk format/migration boundary, reversal requires cassette migration, not ownable by Gherkin, no prior ADR | `NEW_ADR` → `docs/arc42/adr/ADR-0001-query-digest-model-cassette-key.md` |
| Candidate setup and manifest pins | `NOT_QUALIFIED` — reversible within this change; rationale belongs to design | `CANONICAL_ARTIFACT: design.md` |
| Per-power gold minimums and review | `NOT_QUALIFIED` — authoring/workflow behavior owned by usecase+Gherkin | `CANONICAL_ARTIFACT: design.md` |
| Record gate and per-family floor | `NOT_QUALIFIED` — workflow rule owned by usecase+Gherkin | `CANONICAL_ARTIFACT: design.md` |
| Adapter/session boundary | `NOT_QUALIFIED` — component boundary, but reversible within one change and design owns the interface | `CANONICAL_ARTIFACT: design.md` |

`docs/arc42/09-architecture-decisions.md` now registers ADR-0001 and keeps the
per-candidate verdicts; `docs/arc42/adr/README.md` indexes the record; chapter
11 carries the migration consequence (R-015).

## Hardening change list

- `givn/changes/benchmark-run/proposal.md` — corrected the seed path; narrowed
  the carried-D4 crosswalk to the single D4 ledger entry.
- `givn/changes/benchmark-run/design.md` — rewrote D1–D3 with the operator
  decisions and rejected alternatives; added D4 and fourth-candidate decisions;
  manifest-authored pins and setup-lock separation; hybrid endpoints; per-power
  gold minimums with fixture opt-out; unreviewed-gold and below-minimum typed
  failures; per-family floor; scenario→driver table; targets loading/precedence
  and fingerprint; provenance record; fixture/result-cassette migration and S9
  recipe; report schema changes (`<repository>/<metric>`, version 2, endpoints,
  power, targets/provenance fingerprints); adapter rename rationale; updated
  interaction matrix, local runnability, traceability, and risks.
- `givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature`
  — 18 scenarios: 17 `@givn.added @wip` (15 existing scenarios reworked plus
  floor and below-minimum gold) and one `@givn.modified @wip` for the permanent
  replay fingerprint. No `@e2e` touched.
- `givn/changes/benchmark-run/specs/benchmark-comparison/usecase.md` — hybrid
  preconditions, synthetic deadline, floor flow/extension 24, below-minimum
  extension 25, unreviewed-set extension 21, R2/R3/R11 updates, R14/R15, and
  examples 6–8.
- `givn/specs/benchmark-comparison/usecase.md` — removed change IDs from
  Goal/Main flow/Success/R9/out-of-scope and aligned the inventory wording.
- `givn/changes/benchmark-run/arc42.md` — updated the 12-row impact table,
  replaced the bundled ADR verdict with five per-candidate verdicts, and
  recorded `NEW_ADR` for ADR-0001.
- `docs/arc42/01-introduction-and-goals.md` — corrected the stale change path,
  separated the benchmark-gate contract from the real path, and updated the
  quality goals.
- `docs/arc42/02-architecture-constraints.md` — manifest pins, setup-lock
  scope, hybrid endpoints, per-power gold, 1,800-second deadline, floor.
- `docs/arc42/03-context-and-scope.md` — project setup separated from the
  consumer CLI; both endpoints and the provenance record added.
- `docs/arc42/04-solution-strategy.md` — strategy rows, technology choices,
  and the reconciled ADR sentence.
- `docs/arc42/05-building-block-view.md` — provenance writer, migration,
  targets loader, floor gate, setup validation.
- `docs/arc42/06-runtime-view.md` — hybrid real record/replay flow, floor, and
  provenance steps.
- `docs/arc42/07-deployment-view.md` — endpoint deployments, setup-lock scope,
  and provenance path.
- `docs/arc42/08-crosscutting-concepts.md` — hybrid boundary, keys/fingerprints,
  gold power/minimums, publication floor, targets file, provenance, exit codes.
- `docs/arc42/09-architecture-decisions.md` — ADR-0001 register row, later
  registration note, per-candidate verdicts.
- `docs/arc42/10-quality-requirements.md` — quality tree, QS-018 fix,
  QS-019–QS-024.
- `docs/arc42/11-risks-and-technical-debt.md` — R-009, R-012–R-017.
- `docs/arc42/12-glossary.md` — six new terms, updated gold-set entry, four new
  invariants, lifecycle events.
- `docs/arc42/adr/ADR-0001-query-digest-model-cassette-key.md` — new MADR
  record.
- `docs/arc42/adr/README.md` — new ADR directory index.
- `givn/changes/benchmark-run/design-review.md` — this file.

## Verification

- `givn lint --change benchmark-run` → exit 2, 20 findings: 18 expected `@wip`
  warnings (all 18 delta scenarios) and 2 persona-reference advisories on the
  delta usecase. No parse, duplicate, or structural finding; the
  `@givn.modified` scenario's permanent title is accepted.
- `givn check specs --change benchmark-run` → exit 0.
- `givn check design --change benchmark-run` → exit 0.
- `givn check arc42-docs --change benchmark-run` → exit 0.
- `givn status --change benchmark-run` → stage remains `arc42-docs-done`;
  design-review present as this artifact; tasks and review pending.

## Remaining dispositions

- All 18 scenarios remain `@wip` by design; the task list and RED/GREEN loop are
  the next artifact, not this review.
- The real record run, the gold authoring, the fixture cassette migration, the
  synthetic attempt, and the committed provenance/cassettes/report are tasks and
  evidence; none was fabricated here.
- The orchestrator owns the final sign-off on this hardened plan; the
  `DESIGN-REVIEW` sign-off marker is intentionally not written here, so
  `givn status` reports design-review as pending until that review completes.

---

## Status

STATUS: DONE

Hardened; tasks next. The orchestrator writes the final sign-off marker after
its own review.

## Final sign-off

DESIGN-REVIEW: PASS
