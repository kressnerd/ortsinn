# Tasks: benchmark-gate

## Shared design constraints

- Harness: Rust workspace member `benchmark/`; binary `benchmark`; edition
  2024; `Cargo.lock` committed.
- Toolchain: `RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu`; `env.sh`
  resolves `LD_LIBRARY_PATH` for `libz.so.1` (system or Nix).
- Runner: `benchmark/tests/benchmark_features.rs`; `[[test]] name =
  "benchmark_features" harness = false`; `filter_run_and_exit()` or
  `run_and_exit()`; `.fail_on_skipped()`.
- Feature paths: `GIVN_FEATURES` newline-separated; stage or iterate the
  files; fail if empty. `run-tests.sh` supplies the paths.
- Single scenario: `GIVN_FEATURES=<file> cargo test --test benchmark_features
  -- --name "<title>"`.
- E2E: `./run-tests.sh --e2e` → `cargo test --test benchmark_features --
  --tags @e2e`; distinct from `verify.command`.
- Strictness: undefined/pending/skipped steps must fail; steps use
  `unimplemented!()` until GREEN.
- Fixture: `fixture-repo` revision `fixture-1`, commit
  `1111111111111111111111111111111111111111`, 120 code lines; candidates
  `fixture-ocbi`/`fixture-ckb`/`fixture-serena` version `fixture-1`, commits
  `aaaa…`, `bbbb…`, `cccc…`; 5 typo, 4 NL, 4 explanation, 5 concept, 4
  concept-equivalence tasks; expected values in the scenarios.
- Reports: `report.json` + `report.md`; target status `confirmed | proposed |
  unstated`; AC-01 0.90 is `proposed`; metric/task coverage = attempted /
  (attempted + unavailable), excluding not-applicable.
- Candidate setup: CKB SCIP indexers + Tier 3 Rust note; Serena Python 3.13
  + language servers; ocbi `embeddingProvider: "custom"` + recorder URL;
  `.git` present in candidate copies.
- No `tasks.md` scenario is complete without pasted targeted-run output, the
  production files list, and a commit hash.

## Setup — runner, strict mode, and workspace

- [x] Create the Cargo workspace and `benchmark` crate skeleton.
  - Command: `source ./env.sh && cargo test --test benchmark_features --no-run`
    → `Finished test profile`; executable produced.
  - Production files: `Cargo.toml`, `Cargo.lock`, `benchmark/Cargo.toml`,
    `benchmark/src/lib.rs`, `benchmark/src/main.rs`,
    `benchmark/tests/benchmark_features.rs`.
- [x] Create `env.sh`, `run-tests.sh`, `measure-coverage.sh`,
  `merge-coverages.sh`, `benchmark/setup.sh`.
  - `env.sh` sets `RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu` and
    resolves `LD_LIBRARY_PATH` for zlib.
  - `givn/commands.yaml` keeps `verify.command: ./run-tests.sh` and
    `verify.e2e_command: ./run-tests.sh --e2e`.
  - Production files: `env.sh`, `run-tests.sh`, `measure-coverage.sh`,
    `merge-coverages.sh`, `benchmark/setup.sh`, `benchmark/.gitignore`.
- [x] Create the cucumber runner and step skeleton.
  - `benchmark/tests/benchmark_features.rs` reads `GIVN_FEATURES`, stages the
    feature files, builds `cucumber::Cucumber`, calls `.fail_on_skipped()`,
    and uses `run_and_exit()` with the cucumber CLI filter.
  - Step files: `benchmark/tests/steps/benchmark_comparison.rs`,
    `benchmark/tests/steps/e2e/benchmark_cli.rs`,
    `benchmark/tests/steps/mod.rs`, `benchmark/tests/steps/e2e/mod.rs`.
- [x] Prove strict mode.
  - Non-zero probe: `GIVN_FEATURES=/tmp/strict-probe.feature ./run-tests.sh`
    → `STRICT_EXIT=101`; `1 scenario (1 failed)`; `cucumber.rs:1243: 1 step failed`.
  - Baseline: `./run-tests.sh` → `BASELINE_EXIT=0`; `0 features`,
    `0 scenarios` because all real scenarios are `@wip`.
- [x] Verify `run-tests.sh` excludes `@wip` by default and includes a named
  scenario when `--name` is supplied.
  - Full feature set: 21 scenarios, all `@wip`.
  - `@wip`-excluded baseline: 0 scenarios.
  - Targeted S1: `GIVN_FEATURES=<feature> cargo test --test benchmark_features
    -- --name "^Report identifies every candidate and fixed repository revision$"`
    → `S1_RED_EXIT=101`; `1 scenario (1 failed)`; `not implemented`.
  - Commit: e5e77f7d962cbfc2cf2526fa0f3264f33d0b6b24
  - Follow-up fixes: `7887da99014955aa274f96793a54b3de9347464d` — the S1–S5
    `Given` loads the fixture manifest; the `When` runs replay.

## S1 — Report identifies every candidate and fixed repository revision

- [x] RED — remove `@wip` from S1 only; run targeted; paste non-zero output.
  - Command: `GIVN_FEATURES="$(realpath givn/changes/benchmark-gate/specs/benchmark-comparison/benchmark-comparison.feature)" cargo test --test benchmark_features -- --name "^Report identifies every candidate and fixed repository revision$"`
  - Output:
    ```text
    Running tests/benchmark_features.rs (target/debug/deps/benchmark_features-fdb44a46bc060f2b)
    Feature: Benchmark comparison
      Scenario: Report identifies every candidate and fixed repository revision
       ✘  Given the fixture benchmark inputs
          Step panicked. Captured output: not implemented
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    error: test failed, to rerun pass `--test benchmark_features`
    EXIT=101
    ```
- [x] GREEN — implement manifest identity and report identity; list production
  files; run targeted; paste zero output.
  - Production files: `benchmark/src/manifest.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/src/report/mod.rs`, `benchmark/src/runner.rs`,
    `benchmark/src/cli.rs`, `benchmark/src/lib.rs`, `benchmark/src/main.rs`,
    `benchmark/tests/benchmark_features.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`,
    `benchmark/tests/fixtures/e2e/manifest.yaml`,
    `benchmark/tests/fixtures/e2e/repo/src/math_utils.py`,
    `benchmark/tests/fixtures/e2e/repo/src/text_utils.py`.
  - Output:
    ```text
    Feature: Benchmark comparison
      Scenario: Report identifies every candidate and fixed repository revision
       ✔  Given the fixture benchmark inputs
       ✔  When the maintainer runs the benchmark in replay mode
       ✔  Then the report names fixture-ocbi at version "fixture-1" and commit "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
       ✔  And the report names fixture-ckb at version "fixture-1" and commit "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
       ✔  And the report names fixture-serena at version "fixture-1" and commit "cccccccccccccccccccccccccccccccccccccccc"
       ✔  And the report names fixture-repo at revision "fixture-1", commit "1111111111111111111111111111111111111111", and 120 code lines
    [Summary]
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `ec41183052166945f88513092b723f1877c9d9bc`
- Design constraints: manifest/report identity fields; fixture values from the
  shared block; no product code.

## S2 — Report contains the four acceptance measurements

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report contains the four acceptance measurements
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✘  Then the typo recall at rank one is 0.60 with 3 of 5 tasks
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    3 steps (2 passed, 1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/adapters/mod.rs`,
    `benchmark/src/adapters/fixture.rs`, `benchmark/src/record/mod.rs`,
    `benchmark/src/record/cassette.rs`, `benchmark/src/gold/mod.rs`,
    `benchmark/src/metrics/mod.rs`, `benchmark/src/metrics/common.rs`,
    `benchmark/src/metrics/typo.rs`,
    `benchmark/src/metrics/natural_language.rs`,
    `benchmark/src/metrics/explanation.rs`,
    `benchmark/src/metrics/concept.rs`, `benchmark/src/report/model.rs`,
    `benchmark/src/report/render.rs`, `benchmark/src/runner.rs`,
    `benchmark/src/lib.rs`, `benchmark/Cargo.toml`,
    `benchmark/tests/steps/benchmark_comparison.rs`,
    `benchmark/tests/fixtures/e2e/gold/*.jsonl`,
    `benchmark/tests/fixtures/e2e/cassettes/results/results.jsonl`,
    `benchmark/tests/fixtures/e2e/cassettes/model/model.jsonl`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    10 steps (10 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    10 steps (10 passed)
    EXIT=0
    ```
  - Commit: `5261c265dd165b1419bf01315275f0e2e22c73c6`
- Design constraints: typo @1 0.60 (3/5), @5 0.80 (4/5); NL recall @5 0.50
  (2/4), MRR 0.30; explanation 0.50 (2/4), grounding 1.00 (4/4); concept
  0.40 (2/5), concept-equivalence 0.50 (2/4); `concept_equivalence` is in the
  report model.

## S3 — Report states measured gaps without a pre-registered build threshold

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report states measured gaps without a pre-registered build threshold
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✘  Then the typo recall at rank one shows a gap of -0.30 to the target 0.90
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    3 steps (2 passed, 1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/metrics/gaps.rs`,
    `benchmark/src/metrics/mod.rs`, `benchmark/src/metrics/typo.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/src/runner.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    8 steps (8 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    8 steps (8 passed)
    EXIT=0
    ```
  - Commit: `fd6d4702ba09b8c018b751cf5891cf6cc5667227`
- Design constraints: typo @1 gap -0.30 to a **proposed** 0.90; never
  "confirmed"; NL/concept show measured and best values; `path_decision`
  deferred to `maintainer--ortsinn` and `docs/decisions/benchmark-path.md`.

## S4 — Report compares concept recall with and without query expansion

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report compares concept recall with and without query expansion
     ✘  Given a candidate that supports disabling query expansion
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: none beyond the shared arm scoring added with
    the S2 metric model; the S4 step definitions are in
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    7 steps (7 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    7 steps (7 passed)
    EXIT=0
    ```
  - Commit: `23d118270b10308ef1621bd68ae9d0696c938cf1`
- Design constraints: default concept 0.40 (2/5), no-expansion 0.20 (1/5);
  concept-equivalence 0.50 (2/4) and 0.25 (1/4); fixture-serena no-expansion
  `not_applicable` reason `no_expansion_switch`.

## S5 — Report distinguishes an unavailable capability from a zero score

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report distinguishes an unavailable capability from a zero score
     ✘  Given a candidate that cannot explain code
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/report/model.rs`,
    `benchmark/src/metrics/gaps.rs`, `benchmark/src/runner.rs`,
    `benchmark/src/report/render.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - Commit: `a0b0ef6884e09b5da0f47397c2a101127dbc4852`
- Design constraints: fixture-ocbi explanation `unavailable` reason
  `no_explanation_operation`; coverage 0/4 with value 0.0; aggregate remains
  0.50 (2/4); capability gap listed.

## S6 — Replay rerun reproduces the report without an external model call

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Replay rerun reproduces the report without an external model call
     ✘  Given a completed benchmark run with recorded model responses
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/report/model.rs`,
    `benchmark/src/record/cassette.rs`, `benchmark/src/runner.rs`,
    `benchmark/src/report/render.rs`,
    `benchmark/tests/benchmark_features.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`,
    `benchmark/tests/fixtures/e2e/cassettes/results/results.jsonl`,
    `benchmark/tests/fixtures/e2e/cassettes/model/model.jsonl`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Commit: `a9385214870cc6d624354f90448cff505ec9dcba`
- Design constraints: normalized result cassettes; model cassette fingerprint
  `sha256:fixture-model-v1`; no candidate start; no external request.

## S7 — A missing recorded response stops the run

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: A missing recorded response stops the run
     ✘  Given a benchmark run that needs a model response absent from the recording
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/tests/benchmark_features.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`; the replay boundary and
    model-cassette failure path are in `benchmark/src/runner.rs` and
    `benchmark/src/record/cassette.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Commit: `08f7b8d5f53cde6777f7083a8d4f24305588ba63`
- Design constraints: exit code 2; error names the missing input and candidate;
  no `report.json`/`report.md`.

## S8 — The synthetic monorepo is regenerated to the same content fingerprint

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: The synthetic monorepo is regenerated to the same content fingerprint
     ✘  Given the fixed synthetic seed and generator version
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/gen/synthetic_repo.rs`,
    `benchmark/src/gen/mod.rs`, `benchmark/src/lib.rs`, `Cargo.toml`,
    `benchmark/tests/benchmark_features.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
    (two full 10M-line generations; ~42 s of test time)
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
    (two full 10M-line generations; ~35 s of test time)
  - Commit: `b8632dcc6e509b05f0618b9308ae7554d0e7d407`
- Design constraints: `benchmark/src/gen/synthetic_repo.rs`; 10,000,000 code
  lines; fingerprint `sha256:synthetic-demo-v1`; gold locations match.
  The scenario's `sha256:synthetic-demo-v1` string is the recorded fixed-input
  identifier (like the fixture gold/cassette identifiers), not a hex digest.
  The generator also computes `content_digest`, the SHA-256 over the sorted
  `(path, file content digest)` list; the Then-step proves two independent full
  regenerations produce the same digest and the same gold locations. The
  digest dependency is optimized in dev/test builds in the workspace
  `Cargo.toml` so the full generation stays a reasonable test-time operation.

## S9 — Report separates wrong rank from no match in typo recall

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report separates wrong rank from no match in typo recall
     ✘  Given the fixture typo gold set of five tasks
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/report/model.rs`,
    `benchmark/src/metrics/typo.rs`, `benchmark/src/report/render.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Commit: `e874ddf93f3ddb9413129ee3de538551ffaacf45`
- Design constraints: no-match 1/5 = 0.20; @5 0.80 (4/5); no-match rate
  denominator is the typo task count.

## S10 — Report states metric and task coverage with denominators

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report states metric and task coverage with denominators
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✘  Then the typo coverage is 5 of 5 with a value of 1.0
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    3 steps (2 passed, 1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/report/render.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`. The per-candidate
    `attempted`, `unavailable`, and `coverage` fields already existed on
    `Score` from the S2/S5 model; this step exercises every cell.
  - GREEN output:
    ```text
    1 scenario (1 passed)
    9 steps (9 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    9 steps (9 passed)
    EXIT=0
    ```
  - Commit: `54013e22c382c2edcf78e197c248f6c71dd74c73`
- Design constraints: typo 5/5 = 1.0; NL 4/8 = 0.5; explanation 4/8 = 0.5;
  concept 5/5 = 1.0; concept-equivalence 4/4 = 1.0; ckb explanation 4/4,
  ocbi 0/4; formula attempted / (attempted + unavailable), excluding
  not-applicable.

## S11 — Report names each gold set's version, task count, and fingerprint

- [x] RED — remove `@wip` from S11 only; run targeted; paste non-zero output.
  - Output:
    ```text
    Scenario: Report names each gold set's version, task count, and fingerprint
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✘  Then the typo gold set is version 1 with 5 tasks and fingerprint "sha256:fixture-typo-v1"
        Step panicked. Captured output: not implemented: benchmark-comparison: gold set identity
    [Summary]
    1 feature
    1 scenario (1 failed)
    3 steps (2 passed, 1 failed)
    EXIT=101
    ```
- [x] GREEN — add the gold-set identity record to the report model and runner;
  implement the step; list production files; run targeted; paste zero output.
  - Production files: `benchmark/src/report/model.rs`,
    `benchmark/src/report/render.rs`, `benchmark/src/runner.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output:
    ```text
    Scenario: Report names each gold set's version, task count, and fingerprint
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the typo gold set is version 1 with 5 tasks and fingerprint "sha256:fixture-typo-v1"
     ✔  And the natural-language gold set is version 1 with 4 tasks and fingerprint "sha256:fixture-nl-v1"
     ✔  And the explanation gold set is version 1 with 4 tasks and fingerprint "sha256:fixture-explanation-v1"
     ✔  And the concept gold set is version 1 with 5 tasks and fingerprint "sha256:fixture-concept-v1"
    [Summary]
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `50ae634ebf9fd6643b9d9eb7bf5d76b2bcf6d0d4`
- Design constraints: typo v1 5 tasks `sha256:fixture-typo-v1`; NL v1 4
  `fixture-nl-v1`; explanation v1 4 `fixture-explanation-v1`; concept v1 5
  `fixture-concept-v1`.

## S12 — Report states the exact command that reproduces the run

- [x] RED — remove `@wip` from S12 only; run targeted; paste non-zero output.
  - Output:
    ```text
    Scenario: Report states the exact command that reproduces the run
     ✔  Given a completed benchmark run with recorded model responses
     ✘  When the maintainer reads the report
        Step panicked. Captured output: not implemented: benchmark-comparison: read the report
    [Summary]
    1 feature
    1 scenario (1 failed)
    2 steps (1 passed, 1 failed)
    EXIT=101
    ```
- [x] GREEN — add the reproduction record (command, portable toolchain prefix,
  manifest/gold/model/result fingerprints) and implement the steps; list
  production files; run targeted; paste zero output.
  - Production files: `benchmark/src/report/model.rs`,
    `benchmark/src/report/render.rs`, `benchmark/src/runner.rs`, `env.sh`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output:
    ```text
    Scenario: Report states the exact command that reproduces the run
     ✔  Given a completed benchmark run with recorded model responses
     ✔  When the maintainer reads the report
     ✔  Then the report states the replay command for the fixture manifest with mode replay and its output path
     ✔  And the report states the toolchain environment prefix used by the command
     ✔  And the report states the manifest, gold, model-cassette, and result-cassette fingerprints
    [Summary]
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `180fbf8b2aae6726facf6d14db086a61ee376a7c`
- Design constraints: replay command with fixture manifest, mode, output path;
  toolchain environment prefix; manifest/gold/model/result fingerprints.
  The prefix references zlib through `ZLIB_LIBRARY_PATH` (exported by
  `env.sh`) so no absolute host path enters the report.

## S13 — Report records candidate setup, index status, and timeouts

- [x] RED — remove `@wip` from S13 only; run targeted; paste non-zero output.
  - Output:
    ```text
    Scenario: Report records candidate setup, index status, and timeouts
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✘  Then every candidate records its install command, version, commit, and adapter version
        Step panicked. Captured output: not implemented: benchmark-comparison: candidate identity record
    [Summary]
    1 feature
    1 scenario (1 failed)
    3 steps (2 passed, 1 failed)
    EXIT=101
    ```
- [x] GREEN — add the candidate setup record (indexers, doctor output,
  environment, language support, embedding boundary, git metadata, index
  status/duration) to the manifest and report; implement the steps; list
  production files; run targeted; paste zero output.
  - Production files: `benchmark/src/manifest.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/src/runner.rs`,
    `benchmark/tests/fixtures/e2e/manifest.yaml`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output:
    ```text
    Scenario: Report records candidate setup, index status, and timeouts
     ✔  Given the fixture benchmark inputs
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then every candidate records its install command, version, commit, and adapter version
     ✔  And CKB records the pinned scip-typescript, scip-python, and rust-analyzer indexer versions and its doctor output
     ✔  And CKB records Rust language support as Tier 3 in its capability note
     ✔  And Serena records its Python 3.13 environment and the pyright, typescript-language-server, and rust-analyzer versions
     ✔  And open-codebase-index records embeddingProvider "custom" and the recorder base URL
     ✔  And every candidate copy records git_metadata "present"
     ✔  And every candidate records a per-repository index status and duration
    [Summary]
    1 feature
    1 scenario (1 passed)
    9 steps (9 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    9 steps (9 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `e11669276f0c20ebf9c0cf597cf2b64be8d54bee`
- Design constraints: install/version/commit/adapter; CKB indexers + doctor
  output + Rust Tier 3; Serena Python 3.13 + language servers; ocbi custom
  embedding URL; `git_metadata: present`; per-repository index status/duration.

## S14 — A fixed-input mismatch stops the run

- [x] RED — remove `@wip` from S14 only; run targeted; paste non-zero output.
  - Output:
    ```text
    Scenario: A fixed-input mismatch stops the run
     ✘  Given a fixture manifest whose fixture-repo commit is "2222222222222222222222222222222222222222"
        Step panicked. Captured output: not implemented: benchmark-comparison: fixture manifest with a mismatched commit
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
- [x] GREEN — record the commit each normalized result was recorded against
  and verify it against the manifest before scoring; implement the given and
  the error step; list production files; run targeted; paste zero output.
  - Production files: `benchmark/src/record/cassette.rs`,
    `benchmark/src/runner.rs`,
    `benchmark/tests/fixtures/e2e/cassettes/results/results.jsonl`,
    `benchmark/tests/benchmark_features.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output:
    ```text
    Scenario: A fixed-input mismatch stops the run
     ✔  Given a fixture manifest whose fixture-repo commit is "2222222222222222222222222222222222222222"
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the run exits with code 2
     ✔  And the error names fixture-repo and the mismatched expected and actual commits
     ✔  And no report.json or report.md is published
    [Summary]
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `9c4b724bece14e4f2ba8d4ea11538d798bbafe6d`
- Design constraints: expected `1111…`, actual `2222…`; exit 2; error names
  the repository and commits; no report files.

## S15 — An uninstallable or unstartable candidate is recorded and the run continues

- [x] RED — remove `@wip` from S15 only; run targeted; paste non-zero output.
  - Output:
    ```text
    Scenario: An uninstallable or unstartable candidate is recorded and the run continues
     ✘  Given fixture-unstartable whose MCP handshake fails
        Step panicked. Captured output: not implemented: benchmark-comparison: fixture-unstartable handshake
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
- [x] GREEN — record candidate-level unavailability in the report, make every
  metric cell for such a candidate `unavailable` with the reason, list it as a
  capability gap, and continue with the other candidates; implement the steps;
  list production files; run targeted; paste zero output.
  - Production files: `benchmark/src/metrics/common.rs`,
    `benchmark/src/metrics/typo.rs`,
    `benchmark/src/metrics/natural_language.rs`,
    `benchmark/src/metrics/explanation.rs`,
    `benchmark/src/metrics/concept.rs`, `benchmark/src/metrics/gaps.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/src/runner.rs`, `benchmark/tests/steps/benchmark_comparison.rs`.
  - Output:
    ```text
    Scenario: An uninstallable or unstartable candidate is recorded and the run continues
     ✔  Given fixture-unstartable whose MCP handshake fails
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the report marks fixture-unstartable unavailable with the reason "candidate_unstartable"
     ✔  And the run completes and reports measured values for fixture-ocbi and fixture-ckb
     ✔  And the report lists fixture-unstartable in the capability gaps
    [Summary]
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
- [x] REFACTOR — run targeted; paste zero output.
  - Output:
    ```text
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Combined active set (S1–S15):
    ```text
    1 feature
    15 scenarios (15 passed)
    96 steps (96 passed)
    EXIT=0
    ```
- [x] COMMIT — hash: `8109028aee80e48f6ce5ff57b951597b7b27fe24`
- Design constraints: `fixture-unstartable` MCP handshake fails;
  `candidate_unstartable`; other candidates measured; capability gap.

## S16 — A candidate that exceeds its index deadline is unavailable, not zero

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: A candidate that exceeds its index deadline is unavailable, not zero
     ✘  Given fixture-timeout with an index deadline of 1 second on fixture-repo
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/manifest.rs`,
    `benchmark/src/metrics/common.rs`, `benchmark/src/metrics/typo.rs`,
    `benchmark/src/metrics/natural_language.rs`,
    `benchmark/src/metrics/explanation.rs`,
    `benchmark/src/metrics/concept.rs`, `benchmark/src/runner.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    Scenario: A candidate that exceeds its index deadline is unavailable, not zero
     ✔  Given fixture-timeout with an index deadline of 1 second on fixture-repo
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the report marks fixture-timeout unavailable on fixture-repo with the reason "index_timeout"
     ✔  And the report records the elapsed index time
     ✔  And every metric for fixture-timeout on fixture-repo is unavailable and never zero
     ✔  And the run continues with the other candidates
    [Summary]
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - Commit: `685a3a91eebd3a391364ad11cc62cf70e41cefb1`
- Design constraints: `fixture-timeout`, 1 s deadline, `index_timeout`;
  elapsed time; metrics unavailable, never zero; run continues.

## S17 — A query error counts as a miss and is reported separately

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: A query error counts as a miss and is reported separately
     ✘  Given a natural-language gold set of four tasks where 1 query returns an error
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/metrics/natural_language.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    Scenario: A query error counts as a miss and is reported separately
     ✔  Given a natural-language gold set of four tasks where 1 query returns an error
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the natural-language recall at rank five is 0.50 with 2 of 4 tasks
     ✔  And the report states a query error total of 1 separately from the recall value
     ✔  And the errored query is counted as a miss and not as an unavailable task
    [Summary]
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Commit: `bdd41251ba442433d8e70e7e3e8a9eb891944734`
- Design constraints: NL recall 0.50 (2/4); query error total 1 separate;
  errored query is a miss, not unavailable.

## S18 — A metric with no measured candidate reports an unavailable gap

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: A metric with no measured candidate reports an unavailable gap
     ✘  Given a fixture run where every typo candidate is unavailable
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: none; the S3/S5 gap model already carries an
    unavailable gap without a numeric value. Step definitions only:
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    Scenario: A metric with no measured candidate reports an unavailable gap
     ✔  Given a fixture run where every typo candidate is unavailable
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then the typo to-best gap has status "unavailable" with the reason "no_measured_candidate"
     ✔  And the typo to-target gap has status "unavailable" with the reason "no_measured_candidate"
     ✔  And neither gap value is rendered as zero
     ✔  And the report is still published as complete
    [Summary]
    1 feature
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    6 steps (6 passed)
    EXIT=0
    ```
  - Commit: `b10d24201af038465c880aab0dce6e78eacaaf6d`
- Design constraints: typo `to_best` and `to_target` status `unavailable`,
  reason `no_measured_candidate`; no zero; report complete.

## S19 — Report does not score a regex interface mismatch as natural-language recall

- [x] RED / GREEN / REFACTOR / COMMIT
  - RED output:
    ```text
    Scenario: Report does not score a regex interface mismatch as natural-language recall
     ✘  Given fixture-serena exposes no natural-language search operation
        Step doesn't match any function
    [Summary]
    1 feature
    1 scenario (1 failed)
    1 step (1 failed)
    EXIT=101
    ```
  - GREEN production files: `benchmark/src/metrics/natural_language.rs`,
    `benchmark/src/report/model.rs`, `benchmark/src/report/render.rs`,
    `benchmark/src/metrics/common.rs`, `benchmark/src/manifest.rs`,
    `benchmark/tests/fixtures/e2e/manifest.yaml`,
    `benchmark/tests/fixtures/e2e/cassettes/results/results.jsonl`,
    `benchmark/tests/steps/benchmark_comparison.rs`.
  - GREEN output:
    ```text
    Scenario: Report does not score a regex interface mismatch as natural-language recall
     ✔  Given fixture-serena exposes no natural-language search operation
     ✔  When the maintainer runs the benchmark in replay mode
     ✔  Then fixture-serena's natural-language cells are unavailable with the reason "no_natural_language_search"
     ✔  And the natural-language recall at rank five remains 0.50 with 2 of 4 tasks, excluding fixture-serena
     ✔  And a measured regex arm is labelled "regex_pattern_search_proxy" and excluded from natural-language recall and mean reciprocal rank
    [Summary]
    1 feature
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - REFACTOR output:
    ```text
    1 scenario (1 passed)
    5 steps (5 passed)
    EXIT=0
    ```
  - Combined active set (S1–S19):
    ```text
    1 feature
    19 scenarios (19 passed)
    118 steps (118 passed)
    EXIT=0
    ```
  - Commit:
- Design constraints: fixture-serena NL `unavailable` reason
  `no_natural_language_search`; aggregate NL 0.50 (2/4) excluding it; regex
  proxy labelled `regex_pattern_search_proxy`, excluded from NL/MRR.

## E2E setup — local environment and e2e runner

- [ ] Bring up the local environment: `benchmark/setup.sh` installs Rust
  toolchain env, zlib, candidates, CKB indexers, Serena 3.13 + language
  servers, and the fake OpenAI-compatible endpoint fixture.
  - Output:
- [ ] Configure and prove `verify.e2e_command` strict subset.
  - `verify.command` count:
  - `verify.e2e_command` count:
  - Strict proof output:
- [ ] Prove the e2e runner reaches the local fixture endpoint.
  - Output:

## S20 — Maintainer runs the benchmark and reads the measured report

- [ ] RED / GREEN / REFACTOR / COMMIT (e2e runner)
  - RED output:
  - GREEN production files:
  - GREEN output:
  - REFACTOR output:
  - Commit:
- Design constraints: `env!("CARGO_BIN_EXE_benchmark")`; `run --manifest
  benchmark/tests/fixtures/e2e/manifest.yaml --mode replay`; exit 0; four
  sections and report path; values 0.60/0.50/0.50/0.40; width 80 and 160
  no label/value wrap; transcript committed.

## S21 — Maintainer records a run and replays it without an external call

- [ ] RED / GREEN / REFACTOR / COMMIT (e2e runner)
  - RED output:
  - GREEN production files:
  - GREEN output:
  - REFACTOR output:
  - Commit:
- Design constraints: local fake OpenAI-compatible endpoint; record writes
  model and result cassettes; replay matches measurements and order; no
  external request; model cassette fingerprint written.

## Done when

- [ ] Every scenario has targeted RED, GREEN, and REFACTOR output pasted, a
  production-files list for GREEN, and a commit hash.
- [ ] `verify.command` and `verify.e2e_command` both exit 0 on the full suite.
- [ ] The e2e scenario count is strictly less than the full count, or every
  in-scope scenario is `@e2e` (not the case here).
- [ ] The manual record-run gate evidence is committed or the task names the
  exact condition for it.
