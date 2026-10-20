# 2. Architecture Constraints

## Technical constraints

| Constraint | Motivation |
|---|---|
| Rust gnu toolchain `stable-x86_64-unknown-linux-gnu` (cargo/rustc 1.99.1) | The workstation's default musl toolchain is not runnable; all project scripts source `env.sh`, which sets `RUSTUP_TOOLCHAIN`. |
| `libz.so.1` on the loader path | The gnu `rustc` needs zlib; `env.sh` uses a system zlib or the Nix `zlib` path from `nix build --no-link --print-out-paths nixpkgs#zlib`. |
| LLVM 23 coverage tools | rustc 1.99.1 uses LLVM 23.1.1 and writes raw profile format version 11; `env.sh` resolves `nixpkgs#llvmPackages_23.llvm` and exports `LLVM_COV`/`LLVM_PROFDATA`. `nixpkgs#llvmPackages.llvm` (21.1.8) rejects that profile. |
| No container runtime | Isolation is the local fixture checkout, the caller's report directory, and a loopback recorder. This change starts no candidate process. |
| Fixture repository and candidate identities pinned in the manifest | The fixture revision `fixture-1`, the three candidate identities, and the gold/cassette fingerprints are recorded in the manifest and report. Replay verifies the recorded candidate version/commit, repository commit, gold and committed-cassette digests, gold path/line ranges, and model dependencies. |
| Fixed checkout path in the manifest | Every gold path and line range is validated against `checkout` before any score; a missing file or out-of-range line fails the run. |
| Loopback-only model boundary | Replay serves recorded responses and makes no external request; the `@e2e` record scenario passes a deterministic fake endpoint on loopback. |
| One `benchmark run` command | The consumer surface is the proposed benchmark command; there is no separate `verify` or `report` subcommand. |
| `cucumber` 0.23 runner with `harness = false` and `filter_run_and_exit()` | Libtest would consume `--name`/`--tags`; the strict runner fails the process on any undefined, pending, skipped, or failed step. |
| Two Cobertura reports from separate instrumentation runs | The regular scope excludes `@e2e`; each scope has its own `CARGO_TARGET_DIR`, and the merge refuses byte-identical reports. |
| Package versions pinned in `Cargo.lock` | Reproducible builds. `reqwest` is pinned at 0.13.5; the real candidate SDK (`rmcp`) is `benchmark-run`'s. |

## Organisational constraints

| Constraint | Motivation |
|---|---|
| The split is an operator decision | On 2026-10-20 the user chose to split the real-candidate measurement into `benchmark-run`; `benchmark-gate` delivers the fixture harness. The review records the exact `OPERATOR-DECISION` line and the deferral. |
| The benchmark pre-registers no build threshold | The product-path decision is made after the report, in `docs/decisions/benchmark-path.md`, and the report contains no chosen path. |
| AC-01's 0.90 is recorded as `proposed`, never `confirmed` | The acceptance thresholds in the proposal attachments are not confirmed; presenting 0.90 as accepted would overstate the evidence. |
| Target numbers for AC-03, AC-12, AC-21, and AC-22 stay `unstated` | No confirmed numeric target exists in the repository; inventing one would be fabricated acceptance. |
| The maintainer owns the real gold sets | `benchmark-run` adds the reviewed public gold sets; this change's fixture gold is committed and fixed. |
| A real record run is `benchmark-run`'s acceptance gate | This change cannot install or start a real candidate on this host; it proves the contract with the committed fixture and hands the real record gate to the follow-up change. |

## Conventions

| Convention | Motivation |
|---|---|
| Gherkin scenarios assert concrete fixture values, not section presence | A report that prints headings with arbitrary numbers must not pass. |
| Every metric reports value, numerator, denominator, and coverage; missing cells are `unavailable` or `not_applicable` | Prevents a small or missing sample from reading as a strong result. |
| "Metric/task coverage" always means attempted tasks; "code coverage" always means instrumented lines | The two concepts must not collide in the report or the coverage scripts. |
| Diagrams are Mermaid fenced blocks | ASCII-art diagrams are not permitted in the architecture documentation. |
| The report states the exact replay command and every fingerprint | The result must be independently rerunnable. |
| The fixture candidate is named as such, never just "candidate" | A fixture identity is a test double, not an external tool; the glossary and report text keep the distinction. |
