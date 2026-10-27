# Tasks: benchmark-run

## Shared design constraints

- Harness: Rust workspace member `benchmark/`; binary `benchmark`; edition 2024; `Cargo.lock` committed. No product code.
- Toolchain: `RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu`; `env.sh` resolves `libz.so.1` and LLVM 23.
- Runner: `benchmark/tests/benchmark_features.rs`; cucumber-rs; `.fail_on_skipped()` plus `filter_run_and_exit()`; default filter `not @wip`; `--name`/`--tags` supplied by the caller bypass that default.
- Feature paths: `GIVN_FEATURES` newline-separated; `env.sh`'s `givn_resolve_features` supplies the absolute list; runner fails when empty.
- Single scenario: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "<exact title>"`.
- E2E: `./run-tests.sh --e2e` → `cargo test --test benchmark_features -- --tags @e2e`; distinct from `./run-tests.sh`. No new `@e2e` scenario in this change; the two permanent CLI scenarios remain.
- Strictness: undefined, pending, skipped, and failed steps must exit non-zero; every new step starts with `unimplemented!("benchmark-comparison: <step>")`.
- Manifest pins: `benchmark/manifest.yaml` is the only origin of candidate and repository pins (resolved at implementation with evidence); `benchmark/setup.lock.json` records toolchains and probe results, never the origin of pins; `--refresh` is a new record run.
- Fixed inputs: `typescript-reference` (primary), `python-reference` and `rust-reference` (secondary), each ≥200,000 reference-language code lines; `synthetic-monorepo` (secondary) 10,000,000 code lines, seed `20261020`, generator `1`. Public index deadline 14,400 s; synthetic 1,800 s.
- Gold: primary minimum 50 typo / 30 natural-language / 20 explanation / 50 concept / 20 concept-equivalence; secondary minimum 25 / 15 / 10 / 25 / 10. `gold_minimums.mode: fixture` in the fixture manifest; `published` in the real manifest. Manifest-referenced gold requires `reviewed_by: maintainer--ortsinn`; unreferenced drafts are never loaded.
- Publication floor: at least one measured candidate cell for each of typo, natural-language, explanation, and concept, and at least one measured cell for every manifest repository; otherwise exit 2 before either report file is written.
- Recorded model keys include the SHA-256 query digest: `model:explanation:<candidate>:<task>:sha256:<digest>` and `model:embedding:<candidate>:<model>:sha256:<digest>`. A changed query is a missing response, never reuse.
- Hybrid endpoints: loopback embedding endpoint for benchmark embeddings and candidate-internal embeddings; external OpenAI-compatible explanation endpoint only for the at-most-80 reviewed gold explanation queries. Record both localities, model names, and the explanation budget in setup lock, provenance, and report.
- Targets: `benchmark/targets.yaml`; precedence `--targets` > manifest `targets_file` > inline; SHA-256 fingerprint in report and provenance. AC-01 0.90 is `proposed`; AC-03/12/21/22 are `unstated`.
- Provenance: `benchmark/evidence/real-run/provenance.json` records command, UTC start/end, host/toolchain, endpoint localities/models, candidate pins, per-candidate/repository index outcomes, and cassette digests; scenarios cross-check it.
- Report schema: version 2; `reproduction.gold_fingerprints` keyed `<repository>/<metric>`; `inputs.repositories[].power`; `reproduction.targets_fingerprint`, `reproduction.provenance_fingerprint`; candidate setup deviations.
- Fixture migration: one-shot result-cassette key rewrite, model-cassette regeneration, manifest fingerprint update; the `@givn.modified` scenario updates the permanent replay-fingerprint assertion.
- Evidence run input: `BENCHMARK_EXPLANATION_ENDPOINT` plus its key, and a loopback `BENCHMARK_EMBEDDING_ENDPOINT` serving the embedding model. Do not fabricate cassettes or reports if either is unavailable; record the task as blocked.

## Setup — runner, strict mode, and baseline

- [x] Prove strict mode and record the baseline.
  - Command: `source ./env.sh && ./run-tests.sh`
  - Baseline output: `1 feature / 28 scenarios (28 passed) / 171 steps (171 passed)`, exit 0.
  - Strict probe: temporary `/tmp/benchmark-strict-probe.feature` with one undefined step, run with `GIVN_FEATURES=/tmp/benchmark-strict-probe.feature ./run-tests.sh`. Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
  - Removed the probe and reran `./run-tests.sh`: `28 scenarios (28 passed)`, `171 steps (171 passed)`, exit 0.
  - Confirmed `givn/commands.yaml` declares `verify.command: ./run-tests.sh` and `verify.e2e_command: ./run-tests.sh --e2e`.
  - Production files: none; no repair was needed.
  - COMMIT: cddeb5a

- [x] E2E setup check (no new e2e scenario).
  - Command: `./run-tests.sh` → full count 28; `./run-tests.sh --e2e` → 2 scenarios; e2e count is strictly less than full.
  - Output: full `1 feature / 28 scenarios (28 passed) / 171 steps (171 passed)`, exit 0; e2e `1 feature / 2 scenarios (2 passed) / 14 steps (14 passed)`, exit 0. `2 < 28`.
  - Confirmed the e2e step file remains `benchmark/tests/steps/e2e/benchmark_cli.rs`; `FakeOpenAiEndpoint::start` binds `127.0.0.1:0` and the steps assert every peer is loopback.
  - COMMIT: 18a569a

## Evidence preparation (prerequisites for S7/S11/S12/S15)

- [x] Resolve and commit the real fixed inputs and candidate pins.
  - Authored `benchmark/manifest.yaml` with the three reference repositories and commits (microsoft/TypeScript v7.0.2 `1e4744d6…` measured 731,537 TypeScript lines; python/cpython v3.15.0 `d220409a…` measured 918,945 Python lines; rust-lang/rust 1.99.1 `58cf8f9c…` measured 2,919,114 Rust lines), the synthetic seed/version/fingerprint (`20261020`/`1`/`sha256:799817ea…`), candidate pins (ocbi v0.37.0 `573b71c3…`, CKB v9.3.1 `5ff375db…`, Serena v2.1.0 `5e4f5cd1…` with evidence URLs), power labels, and 14,400/1,800-second deadlines. Cassette paths are referenced; their fingerprints and the reviewed gold entries are added by the record/gold tasks. `targets.yaml` carries AC-01 `proposed` and the rest `unstated`.
  - Authored `benchmark/setup.lock.json` with probes, toolchains, copied pins (`pinned_by: benchmark/manifest.yaml`), and both endpoint records. The explanation endpoint is honestly `configured: false` with model `unconfigured`; the record run is blocked until the operator supplies `BENCHMARK_EXPLANATION_ENDPOINT` plus its model/key.
  - `benchmark/fetch-repos.sh`: `fetch-repos: fixed inputs match the manifest pins` (three pinned shallow checkouts; synthetic content fingerprint matched; per-candidate copies created).
  - `benchmark/setup.sh`: probes `ok` for ocbi/CKB/Serena; `setup validation passed: benchmark/manifest.yaml`; `candidate pins, probes, toolchains, and fixed inputs verified`.
  - Production files: `benchmark/manifest.yaml`, `benchmark/targets.yaml`, `benchmark/setup.lock.json`, `benchmark/fetch-repos.sh`, `benchmark/setup.sh`, `benchmark/.gitignore`, `benchmark/src/setup.rs`, `benchmark/src/manifest.rs` (power/synthetic/evidence/targets_file/gold_minimums), `benchmark/src/lib.rs`, `benchmark/src/bin/setup-check.rs`, `benchmark/src/bin/generate-synthetic.rs`.
  - COMMIT: 555ac04

- [ ] Author and review the real gold sets.
  - Primary repository: 50 typo, 30 natural-language, 20 explanation, 50 concept, 20 concept-equivalence reviewed tasks. Each secondary: 25/15/10/25/10. Synthetic gold generated and reviewed with the generator.
  - Drafts live under `benchmark/gold/drafts/`; reviewed files under `benchmark/gold/<repository>/<metric>.jsonl`; manifest entries carry `version`, `tasks`, `fingerprint`, `reviewed_by`.
  - Run `benchmark/src/gold/propose.rs` and the review checks; output the counts and fingerprints. Output: _(paste)_.
  - COMMIT: _(hash)_

- [ ] Record the real run and commit provenance, cassettes, and report.
  - Requires the hybrid endpoints named above; if unavailable, mark blocked and do not fabricate evidence.
  - Command: `./benchmark/run-real.sh` (fetch/setup/record/replay/provenance).
  - Assert the per-family floor and every manifest cell; commit `benchmark/evidence/real-run/provenance.json`, `benchmark/cassettes/results/results.jsonl`, `benchmark/cassettes/model/model.jsonl`, and `benchmark/reports/report.json`/`report.md`; update manifest fingerprints.
  - Output: _(paste provenance summary and report path)_.
  - COMMIT: _(hash)_

## S1 — Real manifest pins every reference repository and the synthetic monorepo

- [x] RED — remove `@wip` from S1 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^Real manifest pins every reference repository and the synthetic monorepo$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement the real manifest/repository identity assertions; list production files; targeted run; paste zero output.
  - Production files: `benchmark/manifest.yaml`, `benchmark/targets.yaml`, `benchmark/src/manifest.rs` (power/synthetic/evidence/targets_file/gold_minimums fields), `benchmark/src/setup.rs`, `benchmark/src/lib.rs`, `benchmark/src/bin/setup-check.rs`, `benchmark/src/bin/generate-synthetic.rs`, `benchmark/fetch-repos.sh`, `benchmark/setup.sh`, `benchmark/setup.lock.json`, `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] COMMIT: 09f2ed7
- Design constraints: manifest IDs and languages; primary/secondary; 200k minimum; synthetic seed/generator/10M; checkouts under `benchmark/work`; 14,400/1,800-second deadlines; read the committed manifest and setup lock, not a live network call.

## S2 — Real manifest pins every candidate version and commit

- [x] RED — remove `@wip` from S2 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title (step bindings temporarily absent).
  - Output: `1 scenario (1 failed)`, `2 steps (1 passed, 1 failed)`, `Step doesn't match any function`, `EXIT=101`.
- [x] GREEN — assert open-codebase-index, CKB, and Serena version plus 40-hex commit from the manifest; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/manifest.rs` (`Manifest::validate_candidate_pins`, used by both S2 steps), `benchmark/tests/steps/benchmark_comparison.rs`, committed `benchmark/manifest.yaml` (`ocbi`/`ckb`/`serena` pins and evidence URLs).
  - Regression test: `manifest::tests::s2_candidate_pins_require_evidence_and_40_hex_commits` (valid pin, short commit, missing evidence, mismatched evidence version/commit, mismatched pin) → `1 passed`.
  - Output: `1 scenario (1 passed)`, `3 steps (3 passed)`, `EXIT=0` (re-run after moving the assertions into production code).
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `3 steps (3 passed)`, `EXIT=0`.
- [x] COMMIT: 6c7d70a
- Design constraints: manifest is the only pin origin; setup lock must not write pins; candidate adapters `ocbi`/`ckb`/`serena`; 40-character commits.

## S3 — The real manifest and setup lock agree on every pin

- [x] RED — remove `@wip` from S3 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title (step bindings temporarily absent).
  - Output: `1 scenario (1 failed)`, `1 step (1 failed)`, `Step doesn't match any function`, `EXIT=101`.
- [x] GREEN — implement `setup.rs` pin/lock/probe agreement; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/setup.rs` (`validate_pins`: pin equality, 40-hex, probes/toolchains, manifest-only pin origin, endpoints), `benchmark/src/manifest.rs`, `benchmark/tests/steps/benchmark_comparison.rs`, committed `benchmark/setup.lock.json`.
  - Regression test: `setup::tests::s3_setup_lock_agrees_with_pins_and_records_endpoints` (valid agreement, candidate pin mismatch, self pin origin, missing explanation model) → `1 passed`.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0` (re-run after adding the regression test).
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0`.
- [x] COMMIT: 8c7c5af
- Design constraints: setup lock records probe/toolchain records only; no pin origin; checkout and install paths resolve; probe records exist for every candidate.

## S4 — A real fixed input that differs from its pin stops before scoring

- [x] RED — remove `@wip` from S4 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title (step bindings temporarily absent).
  - Output: `1 scenario (1 failed)`, `1 step (1 failed)`, `Step doesn't match any function`, `EXIT=101`.
- [x] GREEN — drive `setup.rs` validation against a temporary manifest/checkout with a wrong commit or missing installation; exit 2, named input, no report; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/setup.rs` (`validate_fixed_inputs`: checkout `HEAD` vs pinned commit, candidate install path, synthetic checkout), `benchmark/tests/steps/benchmark_comparison.rs` (temporary wrong-commit manifest).
  - Regression test: `setup::tests::s4_fixed_input_drift_is_named_and_stops_before_report` (wrong checkout names repository and expected/actual commits, missing install names candidate, exit code 2, no report files, matching inputs pass) → `1 passed`.
  - Output: `1 scenario (1 passed)`, `4 steps (4 passed)`, `EXIT=0` (re-run after adding the regression test).
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `4 steps (4 passed)`, `EXIT=0`.
- [x] COMMIT: be497dd
- Design constraints: no network; temp copy; exit 2; error names the mismatched fixed input; no report files; the run never starts when preparation is invalid.

## S5 — A documented setup deviation is recorded on every affected cell

- [x] RED — remove `@wip` from S5 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^A documented setup deviation is recorded on every affected cell$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement setup-deviation recording and cell labelling through `FakeSession`; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/manifest.rs` (`CandidateSetup::setup_deviations`, `SetupDeviation::{label,affects}`), `benchmark/src/report/model.rs` (`Score::setup_deviations`, label validation), `benchmark/src/metrics/common.rs` (`label_setup_deviations`; aggregates merge labels), `benchmark/src/metrics/{typo,natural_language,explanation,concept}.rs` (every affected candidate cell is labelled without changing its value), `benchmark/src/runner.rs` (deviation limitation), `benchmark/src/report/render.rs` (candidate and score lines), `benchmark/src/adapters/session.rs` (`FakeSession::with_setup_deviation`), `benchmark/tests/benchmark_features.rs` (deviation/endpoint world state), `benchmark/tests/steps/benchmark_comparison.rs` (S5 record-mode steps over a temporary manifest and the loopback fixture endpoint).
  - Regression test: `manifest::tests::s5_setup_deviation_labels_only_the_affected_scope` → `1 passed`; `cargo test -p benchmark --lib` → `8 passed`.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] COMMIT: 59582dc
- Design constraints: setup deviation names dependency, reason, affected capabilities/repositories; affected measured cells keep their value; no affected cell is zero; candidate remains in the report.

## S6 — A candidate that fails its setup probe is unavailable, not zero

- [x] RED — remove `@wip` from S6 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^A candidate that fails its setup probe is unavailable, not zero$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement probe failure → candidate unavailable, other candidates measured, report lists it; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/adapters/session.rs` (`FakeSession::failing_probe` consumers; `apply_session` records the probe outcome as the candidate-level `unavailable` reason), `benchmark/tests/steps/benchmark_comparison.rs` (S6 steps drive a temporary manifest and replay the committed fixture cassettes), `benchmark/tests/benchmark_features.rs` (focused candidate).
  - Regression test: `adapters::session::tests::s6_probe_failure_marks_candidate_unavailable` → `1 passed`.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] COMMIT: 3ccbe6c
- Design constraints: candidate-level `unavailable` with probe reason; other installed candidates measured; run continues; never zero.

## S7 — The committed real run records its recording mode, cassette fingerprints, and model dependencies

- [ ] RED — remove `@wip` from S7 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — assert the committed provenance, cassette fingerprints, recording mode, and model dependencies; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/provenance.rs`, `benchmark/src/runner.rs`, `benchmark/src/report/model.rs`, `benchmark/tests/steps/benchmark_comparison.rs`, committed evidence.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: read committed provenance/cassettes/report; mode `record`; model and result fingerprints; every measured cell's model dependency present; provenance command/UTC/pins/digests/index outcomes cross-check. Do not run a live record in this scenario.

## S8 — A real replay reproduces the report without an external call

- [ ] RED — remove `@wip` from S8 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — `run_replay` over the committed real cassettes; assert measurements, ordering, no candidate process, no external request; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/runner.rs`, `benchmark/src/record/*`, `benchmark/tests/steps/benchmark_comparison.rs`, committed evidence.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: committed manifest/cassettes; no candidate/endpoint; result and model fingerprints; replay probe for installed candidates.

## S9 — A changed query does not reuse a recorded model response

- [x] RED — remove `@wip` from S9 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^A changed query does not reuse a recorded model response$"` (S9 step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement the S9 recipe and typed missing-response behavior; list production files; targeted run; paste zero output.
  - Recipe: copy fixture dir → edit one explanation query → recompute and write its fingerprint in the temp manifest → replay → assert exit 2, missing response for the new digest, old key absent.
  - Production files: `benchmark/src/record/cassette.rs` (normalized query digest, explanation/embedding keys, `ModelRecord::carries_query_digest`, committed-fixture regression test), `benchmark/src/record/keys.rs` (canonical key resolution against the current gold query), `benchmark/src/record/mod.rs`, `benchmark/src/record/proxy.rs` (recorder emits digest keys), `benchmark/src/runner.rs` (replay/record resolve the current gold query and fail with `MissingRecordedResponse`; the query-free limitation is removed), `benchmark/src/bin/migrate-cassettes.rs` (one-shot result-cassette key rewrite), `benchmark/tests/fixtures/e2e/cassettes/{results,model}/*.jsonl` and `benchmark/tests/fixtures/e2e/manifest.yaml` (migrated keys and recomputed fingerprints), `benchmark/tests/benchmark_features.rs` (`ChangedQuery` world state), `benchmark/tests/steps/benchmark_comparison.rs` (S9 steps and the migrated missing-response key), `givn/specs/benchmark-comparison/benchmark-comparison.feature` (permanent fingerprint expectation).
  - Regression tests: `record::keys::tests::s9_changed_query_is_a_distinct_key_and_never_reuses_the_recorded_response`, `record::keys::tests::s9_query_digest_uses_the_normalized_query_text`; `cargo test -p benchmark --lib` → `11 passed`.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0`.
- [x] COMMIT: 614528c
- Design constraints: query digest in the key; no fallback; exit 2; no report; gold fingerprint recomputed before replay.

## S10 — An unreviewed gold set referenced by the manifest stops before scoring

- [x] RED — remove `@wip` from S10 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^An unreviewed gold set referenced by the manifest stops before scoring$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement `reviewed_by` enforcement and typed `unreviewed gold set` failure; unreferenced drafts are not loaded; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/manifest.rs` (`GoldSet::reviewed_by`), `benchmark/src/gold/mod.rs` (`UnreviewedGoldSet`, `ensure_reviewed` called for every manifest-referenced set before its fingerprint or tasks are read), `benchmark/src/gold/propose.rs` (`drafts/` writer; drafts never enter the loader), `benchmark/tests/fixtures/e2e/manifest.yaml` (reviewer on the five committed fixture sets), `benchmark/tests/benchmark_features.rs` (unreviewed set and draft task state), `benchmark/tests/steps/benchmark_comparison.rs` (S10 steps; unreferenced draft under a temporary `gold/drafts/`; the permanent extra-repository fixture's gold entry gains a reviewer).
  - Regression test: `gold::tests::s10_unreviewed_referenced_gold_set_is_typed_and_specifically_named` (missing, empty, and whitespace reviewer are typed and named; a reviewed set loads) → `1 passed`; `cargo test -p benchmark --lib` → `12 passed`.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0`; `./run-tests.sh` → `2 features / 39 scenarios (39 passed) / 228 steps (228 passed)`, exit 0.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `6 steps (6 passed)`, `EXIT=0`.
- [x] COMMIT: e57dc85
- Design constraints: manifest-referenced unreviewed set fails before scoring; no report; drafts under `gold/drafts/` excluded; no change-ID vocabulary.

## S11 — The real record run covers every manifest candidate and repository

- [ ] RED — remove `@wip` from S11 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — audit candidate × repository × metric over the committed result cassette and `inputs.*`; assert floor/provenance/index outcomes; replay installed slices; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/runner.rs`, `benchmark/src/report/model.rs`, `benchmark/src/provenance.rs`, `benchmark/tests/steps/benchmark_comparison.rs`, committed evidence.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: every manifest cell measured/unavailable/not-applicable; synthetic timeout carries elapsed time; replay probe per installed candidate; no omission.

## S12 — The real gold sets declare reviewed task counts

- [ ] RED — remove `@wip` from S12 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — assert per-power minima, reviewers, versions, fingerprints, synthetic generation/review; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/gold/minimums.rs`, `benchmark/src/gold/mod.rs`, `benchmark/src/manifest.rs`, `benchmark/tests/steps/benchmark_comparison.rs`, committed gold.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: primary 170; secondary 85; synthetic generated; lower-power label; `reviewed_by`; typed below-minimum failure.

## S13 — A real candidate's evidence keeps its order and resolution

- [x] RED — remove `@wip` from S13 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^A real candidate\'s evidence keeps its order and resolution$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — drive adapter normalization via `FakeSession` for duplicate and file-only answers; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/adapters/session.rs` (`CandidateSession`, `FakeSession`, `CandidateAnswer`, `normalize_answers`, `dedup_locations` boundary), `benchmark/src/adapters/mod.rs` (module export), `benchmark/tests/benchmark_features.rs` (adapter world state), `benchmark/tests/steps/benchmark_comparison.rs` (S13 steps).
  - Regression test: `adapters::session::tests::s13_normalization_preserves_order_and_file_resolution` → `1 passed`.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] COMMIT: b2ed60f
- Design constraints: preserve candidate order; deduplicate by `(path, start_line, end_line, symbol)` without reordering; file-only remains a file resolution; no capability inference.

## S14 — A real candidate without an operation is unavailable with the capability reason

- [x] RED — remove `@wip` from S14 only; targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^A real candidate without an operation is unavailable with the capability reason$"` (step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    1 step (1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — drive `FakeSession` missing explanation; assert unavailable cell, measured families, capability gap; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/adapters/session.rs` (`capabilities_from_operations`, `Operation::missing_reason`, `apply_session`), `benchmark/tests/steps/benchmark_comparison.rs` (S14 steps and a temporary candidate whose results are duplicated from `fixture-ocbi`), `benchmark/tests/benchmark_features.rs` (focused candidate).
  - Regression test: `adapters::session::tests::s14_missing_operation_is_unavailable_with_capability_reason` → `1 passed`.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `5 steps (5 passed)`, `EXIT=0`.
- [x] COMMIT: 5c87c75
- Design constraints: missing operation → `unavailable` with capability reason; measured families remain; gap listed; never zero.

## S15 — The real report publishes the measured comparison and defers the product path

- [ ] RED — remove `@wip` from S15 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — `run_replay` over committed real cassettes; assert four families/gaps, no-expansion arm, identities/fingerprints, targets labels/sources/fingerprint, provenance, endpoint localities/models/budget, power labels, deferred decision; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`, `benchmark/src/runner.rs`, `benchmark/tests/steps/benchmark_comparison.rs`, committed evidence.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: report schema 2; `<repository>/<metric>` fingerprint keys; primary aggregate excludes secondary; lower-power label; external explanation budget recorded; path decision deferred.

## S16 — A real run below the per-family measurement floor publishes no report

- [ ] RED — remove `@wip` from S16 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — implement the floor check; temp manifest/cassette with one family or repository unmeasured → exit 2, no report files; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/runner.rs`, `benchmark/src/report/model.rs`, `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: per-family measured cell; every repository; exit 2 before report write; unavailable candidates remain in a published report when the floor is met.

## S17 — A real gold set below the published minimum stops before scoring

- [ ] RED — remove `@wip` from S17 only; targeted run; paste non-zero output.
  - Command: single-scenario command with the exact title.
  - Output: _(paste)_
- [ ] GREEN — implement `gold_below_minimum`; temp real manifest/gold below the per-power minimum → exit 2, no report; list production files; targeted run; paste zero output.
  - Production files: `benchmark/src/gold/minimums.rs`, `benchmark/src/gold/mod.rs`, `benchmark/src/manifest.rs`, `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output: _(paste)_
- [ ] REFACTOR — targeted run; paste zero output.
  - Output: _(paste)_
- [ ] COMMIT: _(hash)_
- Design constraints: fixture `mode: fixture`; real `mode: published`; per-power minima; typed failure; no report; no fixture regression.

## S18 — Replay rerun reproduces the report without an external model call (`@givn.modified`)

- [x] RED — remove `@wip` from S18 only (modified scenario); targeted run; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-run/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^Replay rerun reproduces the report without an external model call$"` (S18 step bindings temporarily absent).
  - Output:
    ```
    1 scenario (1 failed)
    5 steps (4 passed, 1 failed)
    Step doesn't match any function
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — migrate the fixture result/model cassettes and manifest fingerprint; assert the report's model fingerprint and every model key's query digest; list production files; targeted run; paste zero output.
  - Migration: the result cassette's `model_requests` keys were rewritten once by `migrate-cassettes` in the S9 commit (the committed fixture must stay replayable as soon as the key format changes); this scenario regenerates the model cassette through the record path (`target/benchmark-e2e/s21-run` from the e2e record scenario), so it now carries digest keys, the recorded request bodies, and the fixture-ocbi embedding record.
  - Production files: `benchmark/src/record/cassette.rs` (S18 committed-fixture regression test), `benchmark/src/record/keys.rs` and `benchmark/src/runner.rs` (canonical digest keys enforced on every replay, delivered with S9), `benchmark/tests/fixtures/e2e/cassettes/model/model.jsonl` (regenerated), `benchmark/tests/fixtures/e2e/manifest.yaml` (recomputed model fingerprint), `benchmark/tests/steps/benchmark_comparison.rs` (S18 steps; the permanent fingerprint step now also checks the declared fingerprint and the recomputed file digest), `givn/specs/benchmark-comparison/benchmark-comparison.feature` (permanent expectation updated to the regenerated digest).
  - Regression test: `record::cassette::tests::s18_committed_fixture_declares_and_carries_query_digests` → `1 passed`; `cargo test -p benchmark --lib` → `11 passed`.
  - Output: `1 scenario (1 passed)`, `7 steps (7 passed)`, `EXIT=0`.
- [x] REFACTOR — targeted run; paste zero output.
  - Output: `1 scenario (1 passed)`, `7 steps (7 passed)`, `EXIT=0`.
- [x] COMMIT: ebef948
- Design constraints: one-shot result-cassette key rewrite; model-cassette regeneration; fingerprint update; permanent replay scenario's literal digest updated; no query-free key remains.

## Closeout

- [ ] Every delta step phrase is bound in the permanent `benchmark/tests/steps/benchmark_comparison.rs`; no delta step copy remains.
- [ ] `./run-tests.sh` exits 0 with all 18 delta scenarios green and the 28 permanent scenarios green; `./run-tests.sh --e2e` exits 0 with 2 scenarios.
- [ ] `givn lint --change benchmark-run` has no findings beyond expected `@wip`/advisory while work is open; before archive, no `@wip` remains.
- [ ] The committed real evidence (provenance, cassettes, report, gold review records) is present and cross-checked; the evidence run was not fabricated.
