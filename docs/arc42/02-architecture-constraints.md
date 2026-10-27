# 2. Architecture Constraints

## Technical constraints

| Constraint | Motivation |
|---|---|
| Rust gnu toolchain `stable-x86_64-unknown-linux-gnu` (cargo/rustc 1.99.1) | The workstation's default musl toolchain is not runnable; all project scripts source `env.sh`, which sets `RUSTUP_TOOLCHAIN`. |
| `libz.so.1` on the loader path | The gnu `rustc` needs zlib; `env.sh` uses a system zlib or the Nix `zlib` path from `nix build --no-link --print-out-paths nixpkgs#zlib`. |
| LLVM 23 coverage tools | rustc 1.99.1 uses LLVM 23.1.1 and writes raw profile format version 11; `env.sh` resolves `nixpkgs#llvmPackages_23.llvm` and exports `LLVM_COV`/`LLVM_PROFDATA`. `nixpkgs#llvmPackages.llvm` (21.1.8) rejects that profile. |
| No container runtime | Isolation is the local fixture checkout, the caller's report directory, and a loopback recorder. The fixture suite starts no candidate process; the real evidence run installs candidates under `benchmark/work/` and starts them outside the automated test suite. |
| Fixture repository and candidate identities pinned in the manifest | The fixture revision `fixture-1`, the three candidate identities, and the gold/cassette fingerprints are recorded in the manifest and report. Replay verifies the recorded candidate version/commit, repository commit, gold and committed-cassette digests, gold path/line ranges, and model dependencies. |
| Fixed checkout path in the manifest | Every gold path and line range is validated against `checkout` before any score; a missing file or out-of-range line fails the run. |
| Hybrid model boundary | Replay serves recorded responses and makes no external request; the `@e2e` record scenario passes a deterministic fake endpoint on loopback. Benchmark embeddings use a loopback endpoint, and only the at-most-80 reviewed gold explanation queries reach the external explanation endpoint under a recorded operator decision. |
| One `benchmark run` command | The consumer surface is the proposed benchmark command; there is no separate `verify` or `report` subcommand. |
| `cucumber` 0.23 runner with `harness = false` and `filter_run_and_exit()` | Libtest would consume `--name`/`--tags`; the strict runner fails the process on any undefined, pending, skipped, or failed step. |
| Two Cobertura reports from separate instrumentation runs | The regular scope excludes `@e2e`; each scope has its own `CARGO_TARGET_DIR`, and the merge refuses byte-identical reports. |
| Package versions pinned in `Cargo.lock` | Reproducible builds. `reqwest` is pinned at 0.13.5; `rmcp` is the latest stable release at implementation, pinned in `Cargo.lock`. |

## Organisational constraints

| Constraint | Motivation |
|---|---|
| The split is an operator decision | On 2026-10-20 the user chose to split the real-candidate measurement into `benchmark-run`; `benchmark-gate` delivers the fixture harness. The review records the exact `OPERATOR-DECISION` line and the deferral. |
| The benchmark pre-registers no build threshold | The product-path decision is made after the report, in `docs/decisions/benchmark-path.md`, and the report contains no chosen path. |
| AC-01's 0.90 is recorded as `proposed`, never `confirmed` | The acceptance thresholds in the proposal attachments are not confirmed; presenting 0.90 as accepted would overstate the evidence. |
| Target numbers for AC-03, AC-12, AC-21, and AC-22 stay `unstated` | No confirmed numeric target exists in the repository; inventing one would be fabricated acceptance. |
| The maintainer owns the real gold sets | Gold sets use per-power published minimums and `reviewed_by`; a manifest-referenced set without a reviewer fails before scoring, and an unreferenced draft is never loaded. |
| The real record run is the benchmark's evidence gate | The record run is a manual evidence task with installed candidates and the hybrid endpoints. A missing dependency is recorded, never fabricated; the synthetic input is attempted within 1,800 seconds; a report publishes only above the per-family floor; the provenance record and the fixture suite keep the boundary honest. |

## Conventions

| Convention | Motivation |
|---|---|
| Gherkin scenarios assert concrete fixture values, not section presence | A report that prints headings with arbitrary numbers must not pass. |
| Every metric reports value, numerator, denominator, and coverage; missing cells are `unavailable` or `not_applicable` | Prevents a small or missing sample from reading as a strong result. |
| "Metric/task coverage" always means attempted tasks; "code coverage" always means instrumented lines | The two concepts must not collide in the report or the coverage scripts. |
| Diagrams are Mermaid fenced blocks | ASCII-art diagrams are not permitted in the architecture documentation. |
| The report states the exact replay command and every fingerprint | The result must be independently rerunnable. |
| The fixture candidate is named as such, never just "candidate" | A fixture identity is a test double, not an external tool; the glossary and report text keep the distinction. |

## Real comparison constraints

| Constraint | Motivation |
|---|---|
| Candidate pins are authored in `benchmark/manifest.yaml` with their evidence and committed; `benchmark/setup.sh` installs exactly those pins and exits non-zero on drift | A remembered or setup-resolved version is not reproducible and makes the comparison unauditable; the manifest is the single origin. |
| `benchmark/setup.lock.json` records probe results and resolved toolchain versions, never a candidate pin | The lock proves what was installed and probed without becoming a second, circular source of truth. `--refresh` is an explicit new pin and a new record run. |
| Public reference repositories are pinned to authored commits with at least 200,000 reference-language code lines | The acceptance criteria name sizes and languages; an unpinned repository makes two runs incomparable. |
| The synthetic monorepo is generated from seed `20261020` and generator version `1`, producing 10,000,000 non-blank, non-comment code lines, and is attempted with a 1,800-second index deadline | The large-input path must be reproducible without committing the generated tree and tractable in one record run; a breach is `index_timeout` with elapsed time. |
| Real gold minimums are per repository power: 50/30/20/50/20 on the primary public repository and 25/15/10/25/10 on each secondary repository, every task reviewed by `maintainer--ortsinn` | The comparison needs enough tasks per family; per-power counts keep authoring bounded while preserving a defensible headline comparison. Secondary repositories are labelled lower-power and excluded from cross-repository aggregates. |
| The model boundary is hybrid: a loopback embedding endpoint plus an external explanation endpoint used only for the at-most-80 reviewed gold explanation queries | Explanations need a stronger model, but candidate-internal embeddings must not create unbounded external cost or code egress. Both localities, models, and the budget are recorded. |
| A real report publishes only when at least one measured candidate cell exists for typo, natural-language, explanation, and concept, and at least one measured cell exists for every manifest repository | A "measured comparison" must not be satisfiable by an all-unavailable report; the runner exits 2 before writing either report file. |
| Candidate installs and SCIP indexers/language servers live under `benchmark/work/` and are recorded in the setup lock | The evidence run must be repeatable on the workstation; a missing dependency becomes a visible setup deviation or unavailable cell. |
