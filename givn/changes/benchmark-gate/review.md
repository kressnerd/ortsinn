# Review: benchmark-gate

README-IMPACT: updated - Quick start, Usage

The review was run against the completed task set (`givn status`: 100/100) and
the committed tree through the hardening commits. The scope split is the
operator decision recorded below. The change delivers the fixture-backed
harness and report contract; `benchmark-run` owns the real candidate adapters,
public fixed inputs, real gold sets, candidate installs, and the real record
run.

## Fabrication Audit (do this first)

### Step definition scan

Command: a scan of `benchmark/tests/steps/**/*.rs` for step attributes with an
empty body (`{}`), `unimplemented!`, or `todo!`:

```text
benchmark/tests/steps/mod.rs                          steps=  0 empty=0 unimplemented=0
benchmark/tests/steps/benchmark_comparison.rs         steps= 88 empty=0 unimplemented=0
benchmark/tests/steps/e2e/mod.rs                      steps=  0 empty=0 unimplemented=0
benchmark/tests/steps/e2e/benchmark_cli.rs            steps= 13 empty=0 unimplemented=0
```

**0 empty/no-op step bodies found across 2 step definition files** (101 step
attributes). Every body references a domain object (`Manifest`, `Report`,
`ResultCassette`, `ModelCassette`, `RunFailure`, `GoldValidationError`,
`BenchmarkWorld`) or an assertion.

### Commit cross-check

Every scenario-level task records a commit. Setup is covered by `e5e77f7`
(workspace, runner, strict mode) and `32f32da` (e2e fixture and loopback
endpoint). Production source means a file under `benchmark/src/`, a workspace
manifest, or a production script, not `tests/` or the spec.

| Scenario | Commit hash | Touches production code? |
|---|---|---|
| Setup: workspace, runner, strict mode | `e5e77f7` | Yes (`Cargo.toml`, `benchmark/src/lib.rs`, `benchmark/src/main.rs`) |
| Setup: e2e fixture and endpoint | `32f32da` | Yes (`benchmark/src/record/proxy.rs`, `benchmark/setup.sh`) |
| S1 | `ec411830` | Yes |
| S2 | `5261c265` | Yes |
| S3 | `fd6d4702` | Yes |
| S4 | `23d11827` | No in the original task; fixed by follow-up `de400364` (`metrics/common.rs`, `metrics/concept.rs`) |
| S5 | `a0b0ef68` | Yes |
| S6 | `a9385214` | Yes |
| S7 | `08f7b8d5` | No in the original task; fixed by follow-up `948fdf7` (`runner.rs` typed replay miss) |
| S8 | `b8632dcc` | Yes |
| S9 | `e874ddf9` | Yes |
| S10 | `54013e22` | Yes |
| S11 | `50ae634e` | Yes |
| S12 | `180fbf8b` | Yes |
| S13 | `e1166927` | Yes |
| S14 | `9c4b724b` | Yes |
| S15 | `8109028a` | Yes |
| S16 | `685a3a91` | Yes |
| S17 | `bdd41251` | Yes |
| S18 | `b10d2420` | No in the original task; fixed by follow-up `2d36756` (`metrics/gaps.rs`, `report/model.rs`, `report/render.rs`) |
| S19 | `0568c8f2` | Yes |
| S20 | `bc042c15` | Yes |
| S21 | `9fd8a5fe` | Yes |
| S22 gold path | `35fd2b0` | Yes (`gold/mod.rs`, `manifest.rs`, fixture manifest) |
| S23 gold line range | `9203c40` | Yes (`GoldValidationError` in `gold/mod.rs`) |
| S11 hardening: concept-equivalence gold identity | `7e75dce` | Yes (`gold/mod.rs`, fixture gold/manifest) |
| S16 hardening: applied index deadline | `2dbd702` | Yes (`manifest.rs`, `runner.rs`, `render.rs`) |
| S24 multi-repository values | `acca925` | Yes (`metrics/*`, `report/model.rs`, `report/render.rs`) |
| S25 gold fingerprint mismatch | `6f29f06` | Yes (`manifest.rs`, `gold/mod.rs`) |
| S26 cassette fingerprint mismatch | `41368d4` | Yes (`runner.rs`) |
| S27 limitations and invalid-report rejection | `acca925` / `e961244` | Yes (`report/model.rs`, `report/render.rs`, `runner.rs`) |
| S28 candidate identity mismatch | `59f8bb6` | Yes (`record/cassette.rs`, `runner.rs`, fixture cassette) |
| S15 extension: uninstallable vs unstartable | `68e4533` | Coverage extension; the closed status model in `metrics/common.rs` already supported both reasons |

The S4/S7/S18 finding from the previous review is remediated: each now has a
follow-up production commit for its behavior (no-expansion arm availability,
typed missing-recorded-response failure, gap status and rendering). S22/S23
add the gold path/line validation production change and its scenarios; S24–S28
add the multi-repository, fingerprint, candidate-identity, limitations, and
write-validation coverage.

### Promised-architecture cross-check

| Component (from design.md) | Exists in tree? | Path |
|---|---|---|
| CLI | Yes | `benchmark/src/cli.rs` |
| Manifest / checkout / fixed inputs | Yes | `benchmark/src/manifest.rs` |
| Runner | Yes | `benchmark/src/runner.rs` |
| Normalized models | Yes | `benchmark/src/adapters/mod.rs` |
| Recorder/replayer | Yes | `benchmark/src/record/{cassette,proxy}.rs` |
| Metrics | Yes | `benchmark/src/metrics/*.rs` |
| Report model and renderer | Yes | `benchmark/src/report/{model,render}.rs` |
| Synthetic generator | Yes | `benchmark/src/gen/synthetic_repo.rs` |
| Gold loader and validator | Yes | `benchmark/src/gold/mod.rs` |
| Gherkin runner | Yes | `benchmark/tests/benchmark_features.rs` |
| E2E steps | Yes | `benchmark/tests/steps/e2e/benchmark_cli.rs` |
| Fixture manifest/checkout/gold/cassettes | Yes | `benchmark/tests/fixtures/e2e/` |
| Setup script | Yes | `benchmark/setup.sh` |
| Cobertura merge/probe tool | Yes | `benchmark/tools/cobertura.py` |
| Real candidate adapters, `rmcp` transport, gold proposer, public manifest/targets/fetch/gold/cassettes, real record run | Out of this change; named as `benchmark-run` interfaces in `design.md` | `benchmark-run` |

The previous "promised components missing" finding is resolved by the scope
split: `design.md` no longer promises the real candidate layer as built here,
and names each such component as `benchmark-run`'s interface. The unused
`FixtureAdapter` was deleted (`d7e04e1`); the scenarios and CLI read the
committed cassettes through the runner.

### Strict-mode proof

Present in `tasks.md` (Setup — "Prove strict mode"):

```text
Non-zero probe: GIVN_FEATURES=/tmp/strict-probe.feature ./run-tests.sh
→ STRICT_EXIT=101; 1 scenario (1 failed); cucumber.rs:1243: 1 step failed
```

The current runner calls `filter_run_and_exit(&staged, |_, _, _| true)` with
`.fail_on_skipped()`; CLI `--name`/`--tags` are applied by `filter_run`, and
the untargeted default is `CUCUMBER_FILTER_TAGS=not @wip`. The design now
states that exact mechanism and the `target/givn-features` staging path.

### Downgraded e2e scenario scan

| @e2e Scenario | Real-interface assertion present? | Driven via real browser (if UI)? | Downgraded? |
|---|---|---|---|
| Maintainer runs the benchmark and reads the measured report | Yes — subprocess `env!("CARGO_BIN_EXE_benchmark")`; asserts exit 0, four terminal sections, printed report path, report JSON values 0.60/0.50/0.50/0.40, and 80/160-column rows | N/A — terminal, no HTML/CSS/JS | No |
| Maintainer records a run and replays it without an external call | Yes — CLI subprocess with `--mode record` against a loopback `axum` endpoint; asserts loopback base URL and peers, endpoint request count, model/result cassette fingerprints, replay report equality, and unchanged request count | N/A — terminal, no HTML/CSS/JS | No |

The record-mode step exercises the recorder→replay contract on the fixture;
the real candidate measurement is `benchmark-run`'s use-case guarantee and its
own record-run gate. Both Then steps assert CLI/endpoint observations.

**Web UI classification:** the opposite classification — a rendered page is a
Web UI — does not hold. The CLI prints plain text to stdout and renders no
HTML, CSS, or JavaScript; no browser surface or driver applies.

### E2e scope check

| Capability | @e2e scenario count | Distinct actions? | Over-produced? |
|---|---|---|---|
| benchmark-comparison | 2 | Yes — (1) run in replay mode and read the report; (2) record against an endpoint and replay | No. `--width` is an output variant asserted inside the replay scenario. |

### Duplicate-implementation check

| Capability | File(s) verify.e2e_command invokes | Other implementation found? (incl. untracked) | Stronger one used instead? |
|---|---|---|---|
| benchmark-comparison | `./run-tests.sh --e2e` → `cargo test --test benchmark_features -- --tags @e2e` → `benchmark/tests/benchmark_features.rs`, which loads `benchmark/tests/steps/e2e/benchmark_cli.rs` (13 step attributes) and the regular file; the driven binary is `benchmark/src/main.rs` → `benchmark/src/cli.rs` | No second e2e implementation. `benchmark/tests/e2e_fixture.rs` was removed (`d7e04e1`); its loopback request/peer assertion is covered by S21. `target/givn-features/*.feature` is a generated staging copy. `git status --ignored` shows no untracked Rust step file. | N/A — the only implementation was reviewed. |

### Local environment check

Configured command, run once during this review:

```text
. ./env.sh && cargo run -q -p benchmark -- run \
  --manifest benchmark/tests/fixtures/e2e/manifest.yaml \
  --mode replay --out reports/benchmark
```

It exited 0 and printed the run identity, candidate list, the four metric
sections, and `Report: reports/benchmark/report.json`; both report files were
written. The fixture checkout, gold, cassettes, and loopback endpoint are the
digital twins; no real candidate process is part of the runnable stack.

### E2e isolation check

| `verify.command` value | `verify.e2e_command` value | Identical? | Regular scope scenario count | E2e scope scenario count | E2e count strictly smaller? |
|---|---|---|---|---|---|
| `./run-tests.sh` | `./run-tests.sh --e2e` | No | 28 (26 non-`@e2e` + 2 `@e2e`) | 2 | Yes (2 < 28) |

The regular command executes all 28 scenarios; the coverage regular scope
excludes `@e2e` and runs 26. The counts are from the targeted runs recorded in
`tasks.md` and the static feature count. No archive verification receipt exists yet; the archive records
the per-scope counts.

### Design.md conformance check

| Decision (from design.md) | design.md says | Actually built | Matches? |
|---|---|---|---|
| `verify.e2e_command` | `./run-tests.sh --e2e` → `cargo test --test benchmark_features -- --tags @e2e` | Same | Yes |
| E2E step file | `benchmark/tests/steps/e2e/benchmark_cli.rs` | Same | Yes |
| E2E framework/driver | cucumber-rs + real CLI subprocess | Same | Yes |
| CLI surface | `run` only with `--manifest`, `--mode`, `--out`, `--width`, `--model-endpoint` | Same | Yes |
| Runner call | `filter_run_and_exit(&staged, \|_, _, _\| true)`; CLI filters in `filter_run` | Same | Yes |
| Staging directory | `target/givn-features/` | Same | Yes |
| Feature discovery | `GIVN_FEATURES` else `givn/specs` plus every active `givn/changes/*/specs`, no hard-coded id | `env.sh` `givn_resolve_features` | Yes |
| Real adapters / MCP / gold proposer / public inputs / fetch | `benchmark-run` interface | Not built; absent by design | Yes |
| Gold validation | Declared SHA-256, path containment, file exists, ordered range, end line inside file, explanation evidence ranges | `Gold::load` + `validate_location` | Yes |
| Gold/cassette identity | Every gold set and committed cassette has a computed digest checked before scoring; mismatches exit 2 with no report | `Gold::load`, `cassette_identity` | Yes |
| Candidate identity | Each recorded result carries candidate version/commit, checked against the manifest | `RecordedResult`, `verify_fixed_inputs` | Yes |
| Index deadlines | `IndexRecord.deadline_seconds` records the applied deadline; availability is per candidate and repository | `manifest.rs`, `runner.rs`, `metrics/common.rs` | Yes |
| Per-repository values | Every metric carries `per_repository`; aggregates combine repository cells | `metrics/*`, `report/model.rs`, `report/render.rs` | Yes |
| Limitations and write validation | `limitations` list and `Report::validate()` before `report.json`/`report.md` | `report/model.rs`, `report/render.rs`, `runner.rs` | Yes |
| Coverage merge | Two `CARGO_TARGET_DIR` runs, two Cobertura files merged by `benchmark/tools/cobertura.py`, per-scope probes | Same | Yes |
| `reqwest` version | 0.13.5 | 0.13.5 in manifest and lock | Yes |
| Local run command | Fixture manifest under `--mode replay` | Same | Yes |
| Manual record-run gate | `benchmark-run`'s gate; deferral dispositioned by the operator decision | Not checked in this change; recorded as the `DEFERRAL` below | Yes |

No design deviation remains in the built tree. The earlier deviations
(staging path, hard-coded change id, `run_and_exit` wording, `reqwest` lock,
coverage mechanism) were resolved by code and the updated design. The
hardening pass changed the report, fixture, and fixed-input contracts; a fresh
design-review is required for those changes and is not claimed by this review.

### Interaction coverage verification

| Inventory entry | Matrix row exists? | @e2e scenario exists? (title) | Promised driving mechanism | Actual driving mechanism | Match? |
|---|---|---|---|---|---|
| Run the benchmark in replay mode on the fixed inputs and read the measured report | Yes | Maintainer runs the benchmark and reads the measured report | `std::process::Command` runs `env!("CARGO_BIN_EXE_benchmark")` with `run --manifest ... --mode replay`; captures stdout/stderr and reads the printed report path | Same; reads `Report: ` path and report JSON | Yes |
| Record a benchmark run against a local model endpoint and replay the recorded run | Yes | Maintainer records a run and replays it without an external call | Same subprocess with `--mode record --model-endpoint http://127.0.0.1:<port>/v1`, then `--mode replay`, comparing report models | Same; also asserts loopback peers and unchanged request count | Yes |

The use-case `## Interactions` table has exactly these two entries; the design
matrix has exactly these two rows; the feature has exactly these two `@e2e`
scenarios; the step file uses the promised subprocess driver. No inventory
entry is unmapped and no matrix row is extra.

## Hardening pass findings (F-05, F-06, F-10)

The fresh-context design grill raised the following findings after the
split. Each is resolved or, where the grill overreached, the artifact was
narrowed instead of relabelled.

| Finding | Disposition | Evidence |
|---|---|---|
| F-05 — concept-equivalence had no gold identity; the concept count excluded the variants | Resolved: the gold is split into `concept.jsonl` (5) and `concept_equivalence.jsonl` (4), each with version, count, and computed digest; `Gold::load` validates the count; S11 asserts the identity | `7e75dce`; S11 output in `tasks.md` |
| F-06 — the report omitted the applied index deadline | Resolved: every index record carries `deadline_seconds`; S16 asserts the applied 1-second deadline alongside elapsed time and `index_timeout` | `2dbd702`; S16 output in `tasks.md` |
| F-10.1 — candidate identity mismatch had no scenario | Resolved: recorded results carry candidate version/commit; `verify_fixed_inputs` compares them; a mismatch exits 2 with no report | `59f8bb6`; S28 |
| F-10.2 — gold fingerprint mismatch had no scenario; fingerprints were labels | Resolved: fixture gold digests are computed SHA-256 values and checked before parsing; S25 mutates the file and keeps the declared digest | `1722dae`; S25 |
| F-10.3 — cassette fingerprint mismatch/corruption had no scenario | Resolved: committed cassette digests are checked; S26 mutates the result cassette and keeps the declared digest | `1722dae`; S26 |
| F-10.4 — multi-repository values had no fixture or model field | Resolved: every metric has `per_repository`; S24 adds `fixture-extra` with its own gold and asserts per-repository and aggregate values | `acca925`; S24 |
| F-10.5 — limitations and invalid-report validation had no scenario | Resolved: the report has a closed `limitations` list and `Report::validate()` runs before write; S27 asserts both | `acca925`; S27 |
| F-10.6 — only `candidate_unstartable` was exercised | Resolved: S15 records `fixture-uninstallable` and `fixture-unstartable` with distinct reasons and a continuing run | `1722dae`; S15 |
| F-10.7 — concept-equivalence gold identity had no scenario | Resolved with F-05 (S11) | `7e75dce`; S11 |

The grill's split boundary is unchanged: no real candidate adapter, public
fixed input, real gold set, fetch, or real record run is added. The harness
now verifies every fixed input replay can check (candidate identity,
repository commit, gold digest, cassette digest, gold path/range), and the
report carries per-repository values, applied deadlines, and limitations.

## Use-Case And Persona Conformance

| Contract | Evidence | Pass? |
|---|---|---|
| Owning use-case guarantees and rules preserved | The report names candidates/revisions/values/coverage/gaps, replay is external-call free, R10 holds, and the success guarantee is scoped to the fixture-backed harness and report contract. The real measurement is explicitly `benchmark-run`'s guarantee. | Yes |
| Capability ownership and typed relationships preserved | One capability (`benchmark-comparison`), no Includes, no Extends. | Yes |
| Confirmed Persona-relevant outcomes preserved | `maintainer--ortsinn` runs the CLI and reads the fixture report; `upstream--maintainer` is conditional and not exercised. | Yes |
| Actors remain distinct from Persona biographies | Actors are the Ortsinn maintainer (primary) and upstream maintainer (conditional); Personas are review lenses. | Yes |
| Migration ledger and coverage evidence complete, if applicable | No migration is part of this change; the archive receipt carries coverage. | N/A |

**Use-case scope rejection criteria:** none holds. The Goal is one outcome;
the primary actor is named and triggered; the capability passes the placement
test; the scenarios observe report content without mutating state;
preconditions are split by mode; the Goal/Trigger do not name a mechanism; the
capability name is goal-shaped; the main flow is one goal; no Extends/Includes
misuse exists. `## Extensions` now lists extensions 1–17, each with its
main-flow step reference; extension 10 covers the all-unavailable gap, 11–12
gold path/line validation, 13–14 gold/cassette digest verification, 15
candidate identity, 16 per-repository values, and 17 limitations/write
validation.

PERSONAS: maintainer--ortsinn

`upstream--maintainer` is conditional on a contribution path and is not
exercised by this change; no invented Persona is recorded.

## Deferred mandatory checks

OPERATOR-DECISION: 2026-10-20 — the user chose to split the real-candidate measurement into benchmark-run; benchmark-gate delivers the fixture harness.

DEFERRAL: the manual real record-run acceptance gate (one committed `benchmark/evidence/record-runs/<candidate>-<repository>.json` from a real `--mode record` run plus a replay probe per installed candidate) — dispositioned by the operator decision above; `benchmark-run` owns the gate, its committed evidence, and the real measured report.

The deferral is not evidence that a real measurement was made. This change's
acceptance evidence is the committed fixture, 28 scenarios (26 regular and 2
`@e2e` CLI), and the measured two-scope coverage.

## Visual Review

| @e2e Scenario | Viewport | Evidence file | Present and non-empty? |
|---|---|---|---|
| Maintainer runs the benchmark and reads the measured report | terminal, 80 and 160 columns | `givn/changes/benchmark-gate/evidence/visual/maintainer-runs-the-benchmark-and-reads-the-measured-report/transcript.txt` | Yes |
| Maintainer records a run and replays it without an external call | terminal, 80 columns | `givn/changes/benchmark-gate/evidence/visual/maintainer-records-a-run-and-replays-it-without-an-external-call/transcript.txt` | Yes |

The transcripts show the run identity, candidate list, four metric sections,
report path, and the deferred path decision. Every metric label/value pair
stays on one line at both widths; at width 80 the path decision wraps, which
the contract permits.

| confirmed persona | Goal | Finding | Disposition |
|---|---|---|---|
| `maintainer--ortsinn` | Run the benchmark and read the measured report | The summary presents the four families, values, coverage, candidate identities, and report path; the report path points at a real report file. No contract rule is broken. | No finding. |

**Baseline:** headings and labels are textual domain terms; no internal
identifier is the only carrier of state; there is no duplicated visible
content; the error state is an error line rendered by the CLI; the interface is
English plain text; no motion exists. The terminal contract declares no page
title and no reduced-motion surface; those Web UI checks do not apply.

VISUAL-REVIEW: PASS

## Retirements

N/A — this change retires no capability.

## Ubiquitous language conformance

The durable glossary now holds Benchmark, Candidate, Capability gap, Cassette,
Code coverage, Concept equivalence, Fixed input, Fixture candidate, Gold set,
Index deadline, Measured gap, Metric/task coverage, No-expansion arm, Path
decision, Query error, Recording boundary, Replay, Target, and the other
existing terms. Findings from the previous review are resolved:

| Term or usage | Finding | Disposition |
|---|---|---|
| `Candidate` / fixture identities | The report and spec use `fixture-ocbi`, `fixture-ckb`, and `fixture-serena` for test doubles, which the old glossary would have called candidates. | The glossary now separates `Candidate` (external tool) from `Fixture candidate` (committed test double), and the report/spec text calls them fixture candidates. |
| `Fixture contract`, `fixture candidate` | Absent from the glossary. | `Fixture candidate` added; the fixture contract is named in the design and use case. |
| `Index deadline`, `Query error`, `Path decision`, `Target` | Absent from the glossary. | Added with contrast terms. |
| `Code coverage` versus `Metric/task coverage` | Code coverage was used without a glossary entry. | Added `Code coverage` with the contrast. |

No inconsistent reuse remains.

## Arc42 implementation conformance

Independently derived affected chapters: all twelve. The harness has goals,
constraints, context, strategy, building blocks, runtime flows, deployment,
crosscutting concepts, an ADR verdict, quality scenarios, risks, and new terms.

| Arc42 chapter or architectural fact | Durable-doc source | `arc42.md` claim | `design.md` treatment | `tasks.md` treatment | Implementation evidence | Match? |
|---|---|---|---|---|---|---|
| Harness goal and quality goals | `docs/arc42/01-introduction-and-goals.md` | Yes — harness goal, quality goals, split | Scope and traceability | S1–S28 | Fixture report, CLI, e2e, coverage | Yes |
| Operator split and `benchmark-run` boundary | `docs/arc42/02-architecture-constraints.md` | Yes — split constraint | Scope, decisions | Split/operator section | Review `OPERATOR-DECISION` and `DEFERRAL`; `benchmark-run` proposal | Yes |
| Fixture checkout, cassettes, fake endpoint | `docs/arc42/03-context-and-scope.md` | Yes — context diagrams and interface table | Scope, recording, fixture contract | S1–S28 | Fixture tree, S20/S21 | Yes |
| Harness strategy and gold validation | `docs/arc42/04-solution-strategy.md` | Yes — strategy | Technology decisions, gold sets | S22/S23/S24–S28 | `gold/mod.rs`, scenarios | Yes |
| Crate building blocks | `docs/arc42/05-building-block-view.md` | Yes — level-1/2 views | Harness tree, internal primitives | No task creates absent components | Tree matches; real adapters marked `benchmark-run` | Yes |
| Replay/record/timeout/mismatch flows | `docs/arc42/06-runtime-view.md` | Yes — four flows | Run sequence | S1–S28 | `runner.rs`, S6/S7/S14/S22/S23/S24–S28 | Yes |
| Fixture deployment, LLVM 23, coverage targets | `docs/arc42/07-deployment-view.md` | Yes — deployment diagram | Local runnability, coverage | Coverage gate task | `env.sh`, scripts, measured reports | Yes |
| Recording boundary, gold validation, closed statuses | `docs/arc42/08-crosscutting-concepts.md` | Yes — crosscutting concepts | Recording, metrics, gold, gaps | S2–S28 | Implemented and covered | Yes |
| ADR verdict | `docs/arc42/09-architecture-decisions.md` | Yes — `ADR action: NONE` | Decisions D1–D4 | No ADR task | Register and chapter agree | Yes |
| Quality scenarios QS-001–QS-015 | `docs/arc42/10-quality-requirements.md` | Yes — quality tree/scenarios | Risks, coverage | S1–S28 | Scenarios and measured coverage | Yes |
| R-001–R-011, TD-001–TD-005 | `docs/arc42/11-risks-and-technical-debt.md` | Yes — risks/debt | Risks | Split section | Matches the split and coverage mechanism | Yes |
| Glossary terms | `docs/arc42/12-glossary.md` | Yes — terms added | Metrics/recording/gaps | Feature uses them | Fixture candidate and missing terms added | Yes |

- [x] Every chapter marked affected by either the independent review or
      `arc42.md` was opened and checked for stale, contradictory, or placeholder
      content.
- [x] Every Arc42 fact or architecture decision relevant to this change is
      represented in `design.md` and mapped to one or more tasks in `tasks.md`.
- [x] The completed implementation follows those facts and decisions; evidence
      names the built file, command, test, or observable result.
- [x] Architecture changes discovered in the completed implementation are
      recorded in the durable Arc42 chapter and the change-level assessment.
- [x] No mismatch remains. The previous mismatches (real adapters, public
      inputs, manual gate) were fixed by the scope split and the updated
      chapters; the design change triggered design-review again.

ARC42 CONFORMANCE: CLEAN

## Coverage

**Coverage result:** measured during this review with the committed scripts.

- Regular scope (`GIVN_COVERAGE_NON_E2E_PATH`): probes
  `benchmark/src/metrics/typo.rs` covered_lines=70 hits=2005 and
  `benchmark/src/report/render.rs` covered_lines=259 hits=8609.
- E2e scope (`GIVN_COVERAGE_E2E_PATH`): probe `benchmark/src/cli.rs`
  covered_lines=26 hits=101.
- Merged (`GIVN_COVERAGE_OUTPUT_PATH`): line-rate 0.939898, 2033 of 2163
  lines; probes `typo.rs` 70/2610, `render.rs` 405/12538, `cli.rs` 26/101.
- The two reports come from distinct `CARGO_TARGET_DIR`s and are not
  byte-identical; `merge-coverages.sh` refuses a missing or copied input.

## Coverage gap classification

| Location | Lines | Classification | Action taken |
|---|---|---|---|
| `benchmark/src/adapters/fixture.rs` `FixtureAdapter` | 20–55 | 1. Dead code (YAGNI/KISS) | Deleted in `d7e04e1`; no caller existed. |
| `benchmark/tests/e2e_fixture.rs` | 77 lines | 1. Dead code / redundant infrastructure test | Deleted in `d7e04e1`; S21 asserts the loopback endpoint, request count, and peer. |
| Gold path/line validation | `benchmark/src/gold/mod.rs` | 2. Missing test coverage | Production validation added (`35fd2b0`, `9203c40`) and scenarios S22/S23 added (`e90db47`). |
| S4 no-expansion arm | `benchmark/src/metrics/common.rs` | 2. Missing production change behind the scenario | `no_expansion_arm_cell` added (`de400364`). |
| S7 missing recorded response | `benchmark/src/runner.rs` | 2. Missing production change behind the scenario | Typed `MissingRecordedResponse` added (`948fdf7`). |
| S18 gap status | `benchmark/src/metrics/gaps.rs` | 2. Missing production change behind the scenario | `Gap::unavailable`/`not_applicable` and rendering added (`2d36756`). |
| F-05 concept-equivalence gold identity | `benchmark/src/gold/mod.rs`, fixture gold/manifest | 2. Missing contract identity | Separate `concept_equivalence` set, version, count, and digest added (`7e75dce`); S11 asserts it. |
| F-06 applied index deadline | `benchmark/src/manifest.rs`, `runner.rs`, `render.rs` | 2. Missing report field | `IndexRecord.deadline_seconds` added (`2dbd702`); S16 asserts the 1-second deadline. |
| Multi-repository values | `benchmark/src/metrics/*`, `report/model.rs` | 1. Missing production behavior | `per_repository` scores added (`acca925`); S24 asserts `3/5`, `2/5`, and `5/10`. |
| Gold/cassette identity verification | `benchmark/src/gold/mod.rs`, `runner.rs` | 1. Missing production behavior | Digests checked with exit 2 and no report (`1722dae`); S25/S26. |
| Candidate identity verification | `benchmark/src/record/cassette.rs`, `runner.rs` | 1. Missing production behavior | Recorded version/commit checked against the manifest (`59f8bb6`); S28. |
| Limitations and report validation | `report/model.rs`, `report/render.rs`, `runner.rs` | 1. Missing production behavior | `limitations` and `Report::validate()` added (`acca925`); S27. |
| Uninstallable vs unstartable | `metrics/common.rs`, fixture | 3. Coverage-only gap | Both reasons were already supported; S15 now records both candidates (`1722dae`). |

No region is unclassified. The real candidate adapters and public fixed inputs
are a different change's deliverable (`benchmark-run`); they are not an
uncovered region of this change's use-case guarantee.

No bucket-3 exception was needed. There is no fourth bucket and no excusing
language.

## Scenario execution evidence

Recorded evidence in `tasks.md`: 1 feature, 28 scenarios, 171 steps for the
regular scope and 1 feature, 2 scenarios, 14 steps for the e2e scope, both
exit 0. The review ran the configured local command once. `givn lint --change
benchmark-gate` exits 0 and reports one clean file.

`givn check review --change benchmark-gate` passes integrity and the run
declaration, net delta, overlap dispositions, and the operator-decision /
deferral check.

## Overlap dispositions

`givn lint --change benchmark-gate` reports no shape-match, duplicate, subset,
or long-scenario findings. `givn check review` reports "overlap dispositions
passed"; there are no removed+added pairs to disposition.

| Scenario A | Scenario B | Disposition |
|---|---|---|

## Split-or-keep

No scenario exceeds the lint threshold; `givn lint` reports no `LONG` finding.

| Scenario | Decision |
|---|---|

## Sign-off

- [x] Fabrication audit: CLEAN — the S4/S7/S18 production-touch finding is
      remediated by follow-up commits; the split removes the real-adapter
      components from this change's promises; the unused adapter and duplicate
      test are deleted.
- [x] Every checked task has a verified commit touching production code, or a
      production follow-up commit for the three earlier gaps.
- [x] Every promised component exists, or is named as `benchmark-run`'s
      interface.
- [x] Strict-mode proof present and passing.
- [x] Scenario execution evidence is recorded per scope; the archive receipt
      will carry the final counts.
- [x] Coverage is measured with the committed two-scope mechanism; the archive
      receipt will carry the merged result.
- [x] Every coverage gap is classified and resolved.
- [x] Dead code deleted. Missing tests added. No hard-to-test gap was needed.
- [x] Redundant unit tests removed (`e2e_fixture.rs`).
- [x] No `@wip` tags remain. No implementation-layer detail in the spec.
- [x] Canonical E2E policy read; every normalized inventory action has one
      `@e2e` scenario and variants are regular scenarios.
- [x] Every E2E scenario has a real-interface primary assertion and uses the
      reviewed driver.
- [x] The local run command starts the fixture stack cleanly, verified during
      this review.
- [x] The exact files `verify.e2e_command` invokes were identified and read;
      the tree was searched for a parallel e2e implementation.
- [x] `verify.e2e_command` is not identical to `verify.command`; 2 of 28
      scenarios prove the e2e scope is strictly smaller.
- [x] Implementation (commands, file layout, framework/driver) matches
      `design.md`; the previous deviations were fixed and design-review was
      re-run.
- [x] Interaction coverage verified: every inventory entry maps to a matrix
      row, an `@e2e` scenario, and the promised driving mechanism.
- [x] No finding was excused with a classification outside the three buckets.

REVIEW: PASS
