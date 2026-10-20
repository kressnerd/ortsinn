# Design review: benchmark-gate

Phase 2 hardening of the `givn` design review. Phase 1 (fresh-context
grilling) produced 20 ranked findings; this document records the decision and
the applied resolution for each. No `tasks.md` was written and no
implementation was started. The proposal was not changed.

## Findings and resolutions

| # | Finding | Decision | Resolution in the artifacts |
|---|---|---|---|
| Q1 | The runner could not honour `--name`/`--tags` or a multi-file `GIVN_FEATURES` list, and `run()` could exit 0 on failures. | Add a `harness = false` test target; stage the multi-file list and run once with `filter_run_and_exit()`; fail on an empty list. | `design.md` "Executable-spec runner" names the `[[test]] name = "benchmark_features" harness = false` target, the staging directory, empty-list failure, and `filter_run_and_exit()`; `design.md` "Strict mode and the RED stub pattern" repeats the exit-code requirement; the tree records the Cargo target. |
| Q2 | The Then-steps asserted section presence, not measured values; no fixture contract existed. | Add a concrete fixture contract and assert every metric and arm value; name the RED stub pattern. | `design.md` "Fixture contract" fixes repository/revision, candidate identities, gold task counts, fingerprints, and exact values for typo @1/@5/no-match, NL recall/MRR, explanation correctness/grounding, concept default/no-expansion, and concept-equivalence default/no-expansion. Every scenario Then-step in `benchmark-comparison.feature` now asserts those values. The runner section names `unimplemented!(\"benchmark-comparison: <step text>\")` and the typed `NotImplemented` error as the RED pattern. |
| Q3 | AC-01's 0.90 was presented as a confirmed target. | User chose option 3: keep 0.90 but record it as `proposed`. | `design.md` Metrics, report contract, manifest model, report model, and risks record `status: proposed` with source "proposal attachments 1/7 (AC-01); not confirmed" and label the gap "gap to proposed target". AC-03/12/21/22 remain `unstated`. The feature asserts the "proposed, never confirmed" label. No artifact calls the target confirmed. |
| Q4 | Serena's NL cell was measured with regex search; CKB's Rust support is Tier 3. | Mark Serena NL `unavailable` with `no_natural_language_search`; optional labelled proxy excluded from the NL aggregate; record CKB Rust Tier 3. | `design.md` candidate table, the following explanation, the report model (`proxy` with `excluded_from_aggregate`), limitations, and risks. The feature adds a scenario asserting NL unavailability, the unchanged 0.50 NL aggregate, and the proxy label; the setup scenario asserts CKB Rust Tier 3. |
| Q5 | Missing scenarios: no-match rate, coverage, gold identity, reproduction command, setup record, mismatch, uninstallable candidate, index deadline, query error, record→replay, all-unavailable gaps. | Add the scenarios with concrete fixture values, tagged `@givn.added @wip`. | `benchmark-comparison.feature` now has 21 scenarios, including each listed behavior. `design.md` "Fixture contract" supplies the mismatch/timeout/unstartable/query-error variants, and the black-box table maps each new scenario. |
| Q6 | The arc42 impact assessment was not independently defensible; chapters were scaffolds with placeholder diagrams. | Independently walk all 12 rows, update every affected chapter, fill diagrams, record the ADR verdict and trailing status. | All chapters 01–08 and 10–12 now contain decision-specific content; 03/05/06/07/10 have real Mermaid diagrams; 09 records `ADR action: NONE`, the routing to `design.md`, and the existing-ADR check. `arc42.md` has the full table with `#`, Chapter, Affected?, Reason, and Summary columns and ends with `## Status` / `STATUS: DONE`. |
| Q7 | Benchmark terms and the confirmed "Concept equivalence" term were missing from the glossary; "coverage" was ambiguous. | Add the terms with contrasts; distinguish metric/task coverage from code coverage. | `docs/arc42/12-glossary.md` adds Benchmark, Candidate, Capability gap, Cassette, Concept equivalence, Fixed input, Gold set, Measured gap, Metric/task coverage, No-expansion arm, Recording boundary, and Replay. "Metric/task coverage" is used consistently in the design, constraints, crosscutting concepts, and glossary. |
| Q8 | No approved visual direction and no width mechanism. | Record the user's go-ahead for the single terminal baseline; add `--width` and an 80/160 E2E assertion; state why Web UI does not apply. | `design.md` "Visual Design Contract" records approval on 2026-10-20, the `--width` mechanism, and the plain-text/no-HTML sentence. The E2E infrastructure and the E2E scenario assert one-line label/value pairs at both widths. |
| Q9 | Record mode was absent from the inventory and the CLI exposed un-specced `verify`/`report` subcommands. | Add a record-mode inventory entry and `@e2e`; keep only `run`. | `usecase.md` Interactions and `design.md` Interaction Coverage Matrix add the record→replay row; the feature adds an `@e2e` using a local fake endpoint. The CLI is `benchmark run` only; `verify`/`report` are removed from the tree, the matrix, and the local run command. |
| Q10 | Candidate setup normalization missed CKB SCIP indexers, Serena's Python 3.13 environment and language servers, ocbi's custom embedding config, and CKB's Git metadata. | Install and pin all of them; state `.git` presence; record every version/deviation; surface degradation in affected cells. | `design.md` candidate table and "Local toolchain setup" list the indexers, language servers, `embeddingProvider: "custom"` keys, `ckb doctor`, `.git` presence, and the failure rule. `usecase.md` preconditions and the setup scenario assert the recorded facts. |
| Q11 | Coverage scripts lacked a shared environment, the "non-E2E" scope still ran E2E, the child export was in a Rust step, and the merge was undefined. | Add `env.sh`, `--tags 'not @e2e'`, move `show-env` into `measure-coverage.sh`, define profile paths and the `llvm-profdata` + `cargo llvm-cov` merge, and assert non-zero probes. | `design.md` "Coverage process boundaries" gives the exact commands, disjoint profile paths, the two-stage merge, and the four production probes. `env.sh` is in the tree and the runner section states all four scripts source it. `givn/commands.yaml` already names `measure-coverage.sh`/`merge-coverages.sh`, so it needs no change. |
| Q12 | The report model lacked concept-equivalence and gap statuses; coverage arithmetic was undefined. | Add `concept_equivalence`, gap `status`/reason and the all-unavailable case, and per-metric coverage formulas. | `design.md` report model, report contract, and data model include `concept_equivalence`, `to_target`/`to_best` status with `no_measured_candidate`, and coverage `attempted / (attempted + unavailable)` with the no-match denominator. The renderer description adds the equivalence table; scenarios assert the values and the boundary. |
| Q13 | Layout drift (missing `gold/propose.rs`, fixture paths), an ungated production fixture adapter, un-specced CLI surface, and an overstated workspace claim. | Add the paths, gate the fixture adapter, remove the subcommands, reword the workspace claim. | The tree includes `gold/{mod,propose}.rs` and `tests/fixtures/e2e/*`; the fixture adapter is `cfg(test)`/`fixture` only; the CLI is `run` only; the scope says this change creates the workspace with `benchmark` as its only member. |
| Q14 | The proposal claimed a user decision that the ideation record still showed open. | Close Q5 with option 4 (user/2026-10-20); annotate D2–D4 as resolved; keep the seed text; do not reopen Q5. | `givn/ideation/repository-orientation/questions.md` Q5 is closed with the option-4 resolution and source, and notes that it does not confirm AC-01. The seed keeps its text and adds resolution notes for Q5 and D2–D4. |
| Q15 | Duplicate extension numbers, a replay-contradicting precondition, an unmarked conditional persona, and no extension traceability. | Renumber, split preconditions by mode, mark the persona, add extension rows. | `usecase.md` extensions are 1–10 uniquely; preconditions are split into record and replay (replay needs no install); the conditional upstream persona is marked. `design.md` traceability maps extensions 1–9 and the all-unavailable extension 10. |
| Q16 | The suite could be GREEN while no real adapter or record path worked. | Add the record E2E and a manual record-run acceptance gate; make it the top risk and success evidence. | `design.md` risks lists it first; "Local toolchain setup" defines the committed record-run evidence file and replay probe; `usecase.md` success guarantee names the gate; the record `@e2e` proves the recorder→replay contract. |
| Q17 | Interface classification did not say why Web UI does not apply. | Add the sentence. | `design.md` Visual Design Contract: plain text to stdout, no HTML/CSS/JS, no browser driver. |
| Q18 | No structured ADR verdict or existing-ADR check. | Record `ADR action: NONE`, routing to `design.md`, and the register search. | `docs/arc42/09-architecture-decisions.md` "Decision register status" and `givn/changes/benchmark-gate/arc42.md` "ADR qualification" record the five-gate verdict and the empty-register check. |
| Q19 | `rmcp` features and ocbi recording configuration were unnamed. | Name the features and config keys. | `design.md` technology table names `client` + `transport-child-process`; the candidate table names `embeddingProvider`, `baseUrl`, model, and dimensions. |
| Q20 | Proposal container wording and archive-hardcoded script/evidence paths. | Leave the proposal unchanged (not a scope mismatch); remove hardcoded change paths from permanent scripts and evidence. | `design.md` runner discovery only includes the change specs when the directory exists; transcripts default to `benchmark/evidence/visual` via `GIVN_EVIDENCE_DIR`. The proposal's D3 option wording is untouched and the design's no-container decision is authoritative; recorded here rather than silently rewritten. |

## Independent arc42 read

I walked the 12-row selection table against proposal, specs, and design before
opening `givn/changes/benchmark-gate/arc42.md`. My call was **Yes** for rows 1,
2, 3, 4, 5, 6, 7, 8, 9, 10, 11, and 12: the harness has goals, constraints,
context, strategy, components, flows, deployment, crosscutting concepts, an ADR
verdict to record, quality scenarios, risks, and new terms. `arc42.md` marks
every row Yes with a reason and a summary, so there is no omission or
contradiction. Chapter files 01–08 and 10–12 contain decision-specific content;
09 contains the register plus the structured verdict; no chapter is a bare
scaffold, and no ASCII-art diagram exists.

## Checks

- `givn check proposal --change benchmark-gate` → exit 0.
- `givn check specs --change benchmark-gate` → exit 0.
- `givn check design --change benchmark-gate` → exit 0.
- `givn check arc42-docs --change benchmark-gate` → exit 0.
- `givn lint --change benchmark-gate` → exit 2 with 21 findings, all expected
  `@wip` warnings for the 21 in-scope scenarios; no parse, duplicate, or
  structural finding.
- `givn check design-review --change benchmark-gate` → exit 0 after this file
  was written.

## Remaining dispositions

- All 21 scenarios are `@wip` by design; the task list and RED/GREEN loop are
  the next artifact, not this review.
- No deferral of a mandatory check: coverage, E2E, arc42, and the ADR verdict
  are all resolved in the plan.
- The manual record-run gate is a task/acceptance gate, not an untracked
  promise; it is named in the design, risks, and use-case success evidence.

DESIGN-REVIEW: PASS

## Post-split re-review (2026-10-20)

The design was re-grilled in a fresh context after the split and after the
implementation audit. The following findings were raised and resolved in the
artifacts; no finding remains open.

| # | Finding | Resolution |
|---|---|---|
| R1 | The report-contract prose promised Run metadata, harness/dependency/hardware fields, capability sets, per-repository metrics, synthetic inputs, a Limitations section, and write validation that the implemented model does not carry. | `design.md` "Report contract" now describes the implemented model: `run{id, mode, external_requests}`, inputs repositories/gold, candidate setup/status, per-candidate metrics, gaps, path decision, reproduction. The Data model section already matches. |
| R2 | `report.md` omitted the concept and concept-equivalence no-expansion values and the "proposed target" label. | `render.rs` now prints both no-expansion metric rows; a proposed target renders as "to proposed target"; the S18 step asserts the rendered unavailable line. |
| R3 | Cassette keys were described as SHA-256 request keys; the implementation indexes results by `(candidate, metric, task, arm)` and model records by deterministic keys. | `design.md` "Recording and replay" now names the tuple index and the model key shape; the SHA-256 is the file fingerprint. |
| R4 | Durable `benchmark/evidence/visual` transcripts were gitignored. | `benchmark/.gitignore` no longer ignores `evidence/`; both transcripts are committed. |
| R5 | `benchmark/setup.sh` did not verify `cargo-llvm-cov` or the LLVM 23 tools. | `setup.sh` now fails when `cargo-llvm-cov`, `LLVM_COV`, or `LLVM_PROFDATA` is missing and prints the resolved LLVM version. |
| R6 | The E2E section claimed a `BENCHMARK_HOME`, every-row width checks, and terminal-size detection. | `design.md` now says the child writes under `target/benchmark-e2e/`, the width step checks the metric label/value rows, and `COLUMNS` is the fallback. |
| R7 | The reproduction command was not runnable as printed. | The recorded command is `cargo run -p benchmark -- run ...`; the renderer prints `. ./env.sh && <command>` plus the toolchain prefix field. |
| R8 | Arc42 claims overstated the generator subprocess, manifest validation, and the ADR README. | Chapters 01, 03, 05, and 09 now state the in-process generator scenario, manifest-vs-gold validation split, and that `docs/arc42/adr/` holds only the template. |
| R9 | The design traceability/black-box tables lacked the index-deadline and gold-validation scenarios; review counts mixed 23 total with 21 non-e2e. | Rows added for S16/S22/S23; the review states "23 via `run-tests.sh` (21 non-e2e + 2 e2e)". |
| R10 | `benchmark-run` reuses D1–D3 without mapping the gate's carried D2–D4. | `benchmark-run/proposal.md` records the crosswalk: gate D2→run D2, gate D3→run D1, gate D4→run D1/D3. |

Design checks: `givn check design --change benchmark-gate` and
`givn check design-review --change benchmark-gate` pass; the scenario/step,
gold-validation, coverage, and split checks in `review.md` are the
implementation evidence.

## Post-hardening re-review (2026-10-20, fresh context)

The hardened plan was re-checked against the full grill report in a fresh
context (`DesignReReview`). Current counts are **28 scenarios (26 regular + 2
`@e2e`)**; the earlier 23/21 counts in the historical sections are superseded.

| ID | Status | Evidence |
|---|---|---|
| F-01 | Resolved | Report model/contract match; `Report::validate` and the generated fixture report carry the declared fields. |
| F-02 | Resolved | Candidate/repository identities, gold digests, and cassette digests are verified; S25/S26/S28 exercise the failures. |
| F-03 | Resolved | This post-hardening section is the current gate artifact; the historical 23/21 counts are marked superseded. |
| F-04 | Resolved | `cargo run -p benchmark -- run …` is emitted with `. ./env.sh && …`; S12 checks it. |
| F-05 | Resolved | Five gold identities, including `concept_equivalence` with 4 tasks and its digest; S11 asserts them. |
| F-06 | Resolved | `IndexRecord.deadline_seconds` is recorded and rendered; S16 asserts the applied 1-second deadline. |
| F-07 | Resolved | Both durable transcripts are tracked; `benchmark/.gitignore` ignores only generated directories. |
| F-08 | Resolved | `Report::validate` checks the closed status, coverage, fingerprints, and limitations; S27 rejects an invalid report before writing. |
| F-09 | Resolved | The child writes under `target/benchmark-e2e/`; replay makes no network request; no `BENCHMARK_HOME` remains. |
| F-10 | Resolved | S24 multi-repository, S25 gold mismatch, S26 cassette mismatch, S27 limitations/validation, S28 candidate identity, and S15 uninstallable/unstartable are implemented. |
| F-11 | Documented limitation | The fixture model key is `(candidate, task or model)` and excludes the query; the report limitation names it and `benchmark-run` must add a query digest before reuse across gold versions. |
| F-12 | Resolved | `design.md` and arc42 ch.05 place the synthetic generator under the executable-spec verification path, not the runner. |

No blocker remains. `givn check design-review --change benchmark-gate` and
`givn check design` pass; `review.md` carries the implementation evidence.

DESIGN-REVIEW: PASS
